# 设计方案：原生绘制（Canvas）、动画系统、可点击自定义 View 与 prepaint 回调

> 状态：**待审阅**。审阅通过 §12 决策点后再进入执行（分阶段见 §11）。
> 目标：把 GPUI 的原生绘制能力与 gpui-kit 的 motion 动画系统暴露给 C#，
> 并提供「自定义 View + 可控命中区域 + prepaint/paint 阶段回调」的统一写法，
> 且不破坏现有 Element / component / ABI 语义。

---

## 1. 目标与非目标

### 目标
1. **原生绘制**：C# 可用 GPUI 的 `canvas` / `paint_quad` / `paint_path` / 渐变 / 阴影 /
   图片绘制，支持两种模式：
   - **声明式**（默认）：绘制图元随 `Render` 生成、随 snapshot 重放，不产生每帧跨界；
   - **按帧回调**（可选）：托管在 native prepaint 阶段拿到 bounds 后返回绘制指令。
2. **动画**：接入 gpui-kit `base::motion`（`transition` / `spring` / `animate_keyframes` /
   `Presence` / `MotionReveal` / `Stagger` / `Easing`）与语义化 `MotionTokens`。
   托管声明目标值，native 按帧插值，**不做每帧跨界**。
3. **自定义 View + 点击区域**：`Canvas` 上声明命中区域（矩形/路径），native 建立 hitbox
   并路由 hover / press / click / move / scroll；支持重叠、遮挡（block_mouse）、光标样式。
4. **prepaint 等阶段回调**：托管在 layout / prepaint / paint 阶段被调用（携带 bounds），
   返回绘制指令与命中区域；可选自定义测量（measure）。
5. **向后兼容**：旧 `View` / `Element` / 组件 / 回调继续工作；新能力为增量。

### 非目标（本轮）
- 任意元素树的 CSS `transform: scale/rotate`（GPUI 无通用元素变换）；缩放/旋转动画仅在
  Canvas 自绘内容内可行（见 §7.6）。
- 把每帧动态绘制纳入 retained snapshot（按帧回调是逃生舱，不参与快照复用）。
- 完整 LSP/文本编辑类自定义视图。

---

## 2. 现状与可复用点

| 现有能力 | 位置 | 对本设计的用途 |
|---|---|---|
| 扁平 arena：nodes / ops / children / utf8 | `crates/gpui-net-shell/src/{abi,snapshot}.rs` | 绘制图元与命中区域直接作为 node + child 表达，**无需新 buffer** |
| `Snapshot::decode` 校验（索引/环/字符串） | `snapshot.rs` | 图元命令复用同一校验 |
| 组件注册与 materializer | `registry.rs` / `components/` | 新增 `Canvas`、`Paint*`、`HitRegion`、`Motion` |
| typed children 被父组件「消费为指令」 | `components/table.rs`（先例） | Canvas 把 children 消费为绘制/命中指令 |
| `render_element` 回调（托管返回 arena） | `abi.rs` / `registry.rs::ElementCallback` | Canvas 按帧 paint 回调直接复用 |
| `click(token)` 与 `invoke(token,kind,number,data)` | `abi.rs` | 命中区域事件路由直接复用 |
| 实体子树 / `scope` 回调生命周期 | `entity_host.rs` / `EventRegistry` | Canvas 子树的回调同样按回调作用域回收 |
| gpui `canvas(prepaint,paint)`、`Window::paint_quad/paint_path/paint_layer` | `gpui` | Canvas 的 native 实现基座 |
| gpui-kit `base::motion` | `external/gpui-kit/crates/base/src/motion.rs` | 动画基座 |

**关键结论**：新增组件即可，**不需要新的 arena buffer，也不需要新的 ABI 表项**；
只在组件 id / schema hash 上做一次 bump（见 §10）。

---

## 3. 边界规则与性能预算

沿用 ARCHITECTURE 的规则：**跨界只为状态迁移或事件，绝不为每次 builder 调用/每帧/每属性**。

| 能力 | 跨界频率 | 说明 |
|---|---|---|
| 声明式绘制图元 | 仅状态变化 | 随 `Render` 写入 arena；clean repaint 由 native 重放 |
| 动画 | 仅目标变化 | native 按帧插值并 `request_animation_frame`，托管不参与 |
| 静态命中区域 | 仅状态变化 | 随 `Render` 声明；native 在 prepaint 建 hitbox |
| 动态命中区域 / 按帧绘制 | **每帧 prepaint** | 显式开启的逃生舱；限制指令数，需 benchmark |
| 自定义测量 | 每帧 layout | 仅在声明 `Measure` 时；默认不启用 |

按帧回调约束（文档化）：
- 只在 GPUI 线程；托管读取自身状态应廉价、无分配放大；
- 单次返回的绘制指令数量设软上限（例如 4096），超出报错；
- 与滚动/裁剪叠加时以 canvas 的 content mask 为准。

---

## 4. 线格式：复用 node / op / children

### 4.1 结构
- `Canvas` 是容器节点，其 children 为**指令节点**，按声明顺序决定 **z-order（先画在下）**：
  - 绘制图元：`PaintRect`、`PaintLine`、`PaintPath`、`PaintGradient`、`PaintShadow`、`PaintImage`；
  - 命中区域：`HitRegion`；
  - 裁剪/分组：`PaintClip`（可选，映射 `paint_layer` + content mask）。
- 图元参数用 `OpMethod` 承载（数字/字符串/枚举），复杂几何用节点 `data` 字符串的**紧凑路径 DSL**。
- 复用现有 `Length` 语义（`int`=像素、`double (0..1)`=相对 canvas 的比例、`Length.*` 显式单位），
  由 native 结合 bounds 解析。

### 4.2 路径 DSL（示例）
```
M 0 0 L 100 0 Q 120 0 120 20 C 120 40 80 40 80 20 Z
```
- 指令：`M`(move_to) `L`(line_to) `Q`(curve_to) `C`(cubic_bezier_to) `A`(arc_to) `Z`(close)；
- 坐标为 canvas 局部坐标；支持像素或比例（比例以 `f` 后缀，如 `0.5f`）。
- 由 `PaintPathMaterializer` 用 `PathBuilder` 解析、`window.paint_path` 绘制。
- stroke / fill / dash / 宽度 / 颜色由方法指令决定（`.Stroke(1.5).Fill("#f59e0b").Dash(4,2)`）。

### 4.3 颜色与渐变
- 颜色：`#rgb/#rrggbb/#rrggbbaa/rgb()/rgba()`，native 复用 `try_parse_color`。
- 渐变：`PaintGradient` 仅支持 GPUI 的两段线性渐变
  `linear_gradient(angle, from, to)`（`0..360`，0=top）。

---

## 5. 原生绘制：托管 API 草案

### 5.1 声明式
```csharp
ui.Canvas("chart")
   .Clip()                       // 可选：裁剪到 bounds
   .Add(
       ui.PaintRect(0, 0, 1, 0.25).Fill("#0ea5e9").Radius(8),
       ui.PaintLine(0, 0, 1, 1).Stroke(2).Color("#111827"),
       ui.PaintPath("M 0 0 L 1 0 L 1 1 Z").Fill("#f59e0b").Stroke(1.5).Dash(6, 3),
       ui.PaintGradient(90, "#0ea5e9", "#22c55e").Rect(0, 0.75, 1, 0.25),
       ui.PaintShadow(0, 4, 12, "#00000033").Rect(0, 0, 1, 0.5),
       ui.PaintImage("icons/grid.png").Rect(0, 0, 0.2, 0.2).Fit("contain")
   )
   .Full();
```

### 5.2 按帧回调（可选，拆分为 Prepaint / Measure）
```csharp
ui.Canvas("plot")
   .Prepaint(RenderPlotToken)    // prepaint 阶段：bounds -> 绘制指令 + 命中区域
   .Measure(MeasurePlotToken)    // 可选：layout 阶段自定义尺寸
   .Full();

[GpuiCallback("Plot")]
private Element RenderPlot(RenderContext ui, IReadOnlyList<string> bounds)
    => ui.PaintPath(BuildPath(bounds)).Stroke(1.5).Color("#2563eb");

[GpuiCallback("Measure")]
private string MeasurePlot(double availableWidth, double availableHeight)
    => $"{availableWidth}\t{Math.Clamp(availableWidth * 0.5, 80, 320)}";
```
- native 在 **prepaint** 调用 `render_element(PrepaintToken, ["x","y","w","h"])`；
  托管返回的 arena 根节点被解释为**绘制指令 + 命中区域**（而非布局子树）。
- `Measure` 在 layout 阶段经 `request_measured_layout` 调用，返回 `"w\th"`；未标注则不启用。
- `bounds` 四个字符串为 canvas 局部像素坐标。

### 5.3 自定义 View 接口（可选糖）
```csharp
public interface ICanvasView
{
    void OnPaint(CanvasPainter painter, Rect bounds);   // painter 收集指令
    void OnHitTest(CanvasHitRegions regions, Rect bounds); // 可选，动态区域
}

ui.CustomView("gauge", new GaugeView()).Full();
```
- `CanvasPainter` 是托管侧的指令收集器（最终落到图元元素/arena）；
- 便于把复杂自绘封装成可复用类，等价于 §5.1/§5.2 的组合。

### 5.4 native 映射（CanvasMaterializer）
- `CanvasElement : gpui::Element`
  - `request_layout`：`window.request_layout(style, [], cx)`；
  - `prepaint`：
    1. 收集**声明式指令**（materialize 时已从 children 解析成 `Vec<PaintCommand>`）；
    2. 若声明了 paint 回调，调用 `render_element` 解析出动态指令并合并；
    3. 对每个 `HitRegion` 执行 `window.insert_hitbox(bounds, behavior)`，保存 `Hitbox`；
  - `paint`：按 z-order 重放 `PaintCommand`（`fill`/`quad`/`paint_path`/`linear_gradient`/
    `paint_drop_shadows`/`paint_image`），随后 `window.on_mouse_event` 注册区域监听、
    `set_cursor_style`。
- 指令类型与 GPUI 对应：

| PaintCommand | GPUI |
|---|---|
| Rect | `fill` / `quad`（含圆角、边框、渐变背景） |
| Line/Path | `PathBuilder::{stroke,fill}` + `dash_array` → `window.paint_path` |
| Gradient | `linear_gradient` 作为 `Background` |
| Shadow | `window.paint_drop_shadows` / `paint_inset_shadows` |
| Clip | `window.paint_layer` + `ContentMask` |
| Image | `window.paint_image`（需要 asset 加载，见风险） |

---

## 6. 可点击自定义 View（命中区域）

### 6.1 静态区域
```csharp
ui.Canvas("bars")
   .Add(ui.PaintRect(...), ...)
   .Regions(
       ui.HitRegion("bar-0", new Rect(0, 0, 0.1, 1))
           .OnClick(ClickBarToken)     // 复用现有 click(token)
           .OnHoverEnter(HoverToken)   // 复用 invoke(token, "hover_enter", index)
           .OnHoverExit(HoverToken)
           .OnPress(PressToken)
           .OnRelease(ReleaseToken)
           .Cursor(CursorKind.Pointer)
           .BlockMouse(),              // 遮挡下层
       ui.HitRegion("bar-1", new Rect(0.1, 0, 0.1, 1)).OnClick(ClickBarToken)
   );
```

### 6.2 动态区域（prepaint 回调）
- paint 回调除绘制指令外，还可返回 `HitRegion` 指令（bounds 由托管按当前 bounds 计算）。
- native 在同一 prepaint 内插入 hitbox，因此动态区域与动态绘制天然对齐。

### 6.3 事件与路由
- **click**：复用 `callbacks.click(session, token)`。
- **hover / press / release / move / scroll**：复用
  `callbacks.invoke(session, token, kind, number, data, len)`，
  约定 `data` = 事件名（`"hover_enter"`…），`number` = 区域序号；托管侧
  `RegisterCallback(Action<EventValue>)` 包装。
- **z-order / 遮挡**：后插入的 hitbox 在上；`HitboxBehavior::BlockMouse` /
  `BlockMouseExceptScroll` 控遮挡；`cx.stop_propagation()` 防重复。
- **光标**：hover 命中时 `window.set_cursor_style`。
- 事件回调同样纳入 §现有「回调作用域」，避免重绘累积。

### 6.4 与滚动/裁剪
- hitbox 使用 `ContentMask`，canvas 滚动/裁剪后命中测试仍正确；
- 区域坐标相对 canvas 内容坐标，忽略滚动偏移（GPUI 负责）。

---

## 7. 动画系统（gpui-kit motion）

### 7.1 托管 API 草案
```csharp
// 过渡：目标变化时从当前值过渡
ui.Motion("panel", MotionSpec.Transition(180, Easing.OutCubic))
   .Opacity(open ? 1 : 0)
   .TranslateY(open ? 0 : 8)
   .Height(open ? Length.Auto : 0)
   .Add(ui.Label("details"));

// 弹簧
ui.Motion("thumb", MotionSpec.Spring(response: 280, damping: 0.85))
   .X(checked ? 20 : 0)
   .Add(ui.Box());

// 关键帧（循环/一次性）
ui.Motion("pulse", MotionSpec.Keyframes(
        Keyframes.From([(0, 0.6), (0.5, 1.0), (1, 0.6)], Easing.InOut)),
        Timing.Infinite(TimeSpan.FromMilliseconds(1200)))
   .Opacity(Animated)
   .Add(ui.Label("·"));

// 进出场（保持挂载至退出完成）
ui.Presence("notice", visible, MotionSpec.Transition(200, Easing.InOut))
   .Fade(0, 1)
   .SlideY(8, 0)
   .Add(ui.Alert("n", "Saved"));

// 高度揭示（配合 spring 进度）
ui.Reveal("accordion", progress: 0..1)
   .Add(ui.Text("..."));
```

### 7.2 原生实现（`components/motion.rs`）
- `Motion` 为容器节点，托管每帧提供**目标值**（opacity/dx/dy/w/h/…）；native 用
  `use_keyed_state(id)` 保存采样状态，调用 gpui-kit：
  - `motion::transition(id, target, Transition, window, cx) -> f32/Pixels`；
  - `motion::spring(id, target, Spring, window, cx)`；
  - `motion::animate_keyframes(id, &Keyframes, Timing, window, cx)`；
  - 活动时这些函数内部会 `window.request_animation_frame()`。
- 把采样值应用到包裹元素：
  - opacity → `.opacity()`
  - 位移 → `.relative()` 包裹 + 子元素 `.absolute().left(x).top(y)`（纯视觉位移，不影响布局）；
    或 `margin`（影响布局，仅供 reveal 场景）。
  - 尺寸 → `.w()/.h()`
  - 背景色 → 由托管以两段色+进度表达时，用 `LinearColorStop` 插值。
- `Presence`：`Presence::new(id, present).transition(..).sample(window, cx)`，`should_render()`
  决定是否继续挂载，`progress` 驱动 fade/slide。
- `Reveal`：`MotionReveal::new(id, progress, child)`（native 负责测量与裁剪）；
  progress 来自 spring/transition。

### 7.3 语义化策略（主题）
暴露 gpui-component 的 `Theme::motion_tokens()` 语义值：
`duration_fast/normal/slow`、`easing_enter/exit/move`、`spring_control`、`spring_move`、
`distance_short/medium`，托管侧以 `Motion.Normal` / `Easing.Enter` / `Spring.Control` 引用。

### 7.4 减少动效
`cx.reduce_motion()` 为真时，native 直接采用终值且不再请求帧；托管无需感知。

### 7.5 帧驱动与性能
- 动画完全在 native GPUI 线程按帧采样；**不跨界**。
- 目标值仅在托管 `Render`（状态变化）时上传；native 缓存上一个采样值以支持中途反向/重定向
  （spring/transition 自带该语义）。
- 支持 `max_fps`（可选）以降低自绘动画帧率。

### 7.6 限制
- GPUI 无通用元素 `transform`：`scale/rotate` 仅能在 Canvas 自绘中通过
  `PathBuilder::{translate,scale,rotate}` / 四边形几何实现；包裹元素树仅支持
  opacity / 位移 / 尺寸 / 颜色。

---

## 8. 阶段与回调总览

| 阶段 | GPUI | 本设计是否暴露给 C# | 方式 |
|---|---|---|---|
| layout | `request_layout` | 可选 | `Measure` 回调（默认不启用） |
| prepaint | `prepaint` | **是** | `Canvas.Paint` 回调（bounds→指令+区域）；`insert_hitbox` |
| paint | `paint` | 是（经 prepaint 指令） | 重放指令 + 注册事件监听 |
| event | `on_mouse_event` | 是 | click / invoke 回调 |

---

## 9. 源生成器整合

- `[GpuiCallback]` 现有 kind 已覆盖绘制回调：`Element M(RenderContext, IReadOnlyList<string>)`
  → `RegisterElement`。新增**语义别名** `Prepaint`（同一注册路径，文档语义为 prepaint 绘制），
  生成 `<Name>Token`，用法 `ui.Canvas(id).Prepaint(<Name>Token)`。
- 新增 kind `Measure`：签名 `string M(double availableWidth, double availableHeight)`，
  返回 `"w\th"`（复用 Rows 风格的字符串回传），注册为测量回调；默认不生成，除非显式标注。
- `EntityView` 已支持；Canvas 子树内的 paint 回调沿用回调作用域回收。

---

## 10. ABI / schema 变更

- **新增组件 id**（registry 顺序追加，需同步 C#/Rust 两侧常量）：

  | id | 组件 |
  |---|---|
  | 103 | `Canvas` |
  | 104 | `PaintRect` |
  | 105 | `PaintLine` |
  | 106 | `PaintPath` |
  | 107 | `PaintGradient` |
  | 108 | `PaintShadow` |
  | 109 | `PaintImage` |
  | 110 | `HitRegion` |
  | 111 | `Motion` |
  | 112 | `Presence` |
  | 113 | `Reveal` |

- **schema hash bump**：`0x6E65_7473_6865_6C56` → `0x6E65_7473_6865_6C57`（`…6C57`）。
- **ABI 版本**：建议保持 `8`（不新增 `GpuiNetShellApi`/`GpuiNetCallbacks` 表项；
  复用 `render_element` / `click` / `invoke`）。若审阅选择为 canvas 事件新增专用回调，
  则 bump 到 `9`（见 §12 决策点 4）。
- **arena**：无新 buffer，格式不变。
- 两侧常量与 pinned 测试字面量需同步更新（`schema.rs` / `NativeProtocol.cs` /
  `NativeProtocolTests`）。

---

## 11. 分阶段执行计划（审阅后）

| 阶段 | 内容 | 交付 |
|---|---|---|
| P1 | 绘制图元（Rect/Line/Path/Gradient/Shadow）+ `Canvas` 声明式重放 | `CanvasPage` 静态自绘 |
| P2 | `Canvas.Paint` 按帧回调（bounds→指令）+ 静态 `HitRegion` + click | `HitTestPage` 可点矩形/路径 |
| P3 | 动态区域 + hover/press/move/scroll + cursor + block_mouse + clip | 交互式自绘控件 |
| P4 | `Motion`（transition，opacity/translate/size）+ `Presence` | `AnimationPage` |
| P5 | `spring` / `keyframes` / `stagger` / `Reveal` + 主题 motion tokens | 动效示例 |
| P6 | 源生成器 `Paint`/`Measure` kind + 文档 + README | 完整 API |
| P7（可选） | `ICanvasView` 糖 + 图片/SVG 绘制 | 可复用自绘类 |

每阶段：`cargo test/fmt/clippy` + `dotnet build/test` + `--check` + 冒烟 + 内存回归
（沿用 `--cycle-pages`/`--cycle-set` 诊断）。

---

## 12. 已确认的决策点

1. **几何编码**：路径用节点 `data` 的紧凑 **DSL**（零新参数种类）。
2. **长度单位**：沿用现有「`int`=px、`double 0..1`=比例、`Length.*` 显式」。
3. **按帧回调**：**拆分为 `Prepaint` 与 `Measure` 两个独立回调**
   （`Prepaint` 在 prepaint 阶段返回绘制指令 + 命中区域；`Measure` 可选，在 layout 阶段返回尺寸）。
4. **事件通道**：复用 `invoke`（`string`=事件名、`number`=区域序号），**ABI 表不变**。
5. **动画属性范围**：仅 `opacity / translate / size / color`；`scale / rotate` 仅在
   `Canvas` 自绘内（经 `PathBuilder` 变换）可行。
6. **位移实现**：wrapper `relative` + 子 `absolute`（纯视觉位移，不影响布局）。
7. **ABI**：schema bump 到 `…6C57`，**ABI 版本保持 8**（无新表项）。
8. **命名**：`ui.Canvas` / `ui.Motion` / `ui.Presence` / `ui.Reveal` / `ui.HitRegion` /
   `MotionSpec` / `Easing` / `Spring` 确认采用。

详细分步执行计划见 [`PLAN_CANVAS_ANIMATION.md`](PLAN_CANVAS_ANIMATION.md)。

---

## 13. 风险与备选

- **每帧跨界成本**：动态绘制/测量是唯一每帧路径，需 benchmark 并限制指令数；备选是只做
  声明式绘制（零每帧跨界）而放弃动态自绘。
- **图片/字体**：`paint_image` 需要 asset 加载管线（`RenderImage`），P7 单列；本轮可只做
  `paint_svg`（`Icon` 已加载路径）或直接跳过。
- **scale/rotate**：元素树无通用变换，需明确限制。
- **reduce_motion / 无障碍**：native 统一短路，托管无需处理。
- **AOT/trim**：生成器与回调保持 AOT 友好（无反射）。
- **重放一致性**：声明式指令进入 retained snapshot 的语义要与现有元素一致（clean repaint
  不重跑托管）。
