# 执行计划：原生绘制（Canvas）、动画、命中区域、Prepaint/Measure

> 依据：[`DESIGN_CANVAS_ANIMATION.md`](DESIGN_CANVAS_ANIMATION.md)（决策已确认）。
> 每个阶段独立可编译、可测试、可回滚；每阶段结束跑完整验证（§6）。

已确认约束：
- 路径几何：节点 `data` 的紧凑 DSL。
- 长度：`int`=px，`double (0..1)`=比例，`Length.*` 显式。
- 回调拆分：`Canvas.Prepaint`（prepaint，返回指令+区域）与 `Canvas.Measure`（layout，返回尺寸）独立。
- 事件复用 `invoke`（`string`=事件名、`number`=区域序号）；click 复用 `click`。
- 动画属性：opacity/translate/size/color；scale/rotate 仅 Canvas 自绘。
- 位移：wrapper `relative` + 子 `absolute`。
- schema hash `…6C56 → …6C57`，**ABI 版本保持 8**。

---

## 1. 组件 id 与线格式（贯穿各阶段）

组件 id 沿用 registry 顺序追加（当前末位 `EntityHost = 102`）：

| id | 组件 | 说明 |
|---|---|---|
| 103 | `Canvas` | 自绘容器（叶子，children 消费为指令） |
| 104 | `PaintRect` | 矩形（圆角/边框/背景/渐变） |
| 105 | `PaintLine` | 直线 |
| 106 | `PaintPath` | 路径（DSL 在 node data） |
| 107 | `PaintGradient` | 两段线性渐变矩形 |
| 108 | `PaintShadow` | 阴影（drop/inset） |
| 109 | `PaintImage` | 图片/SVG（P7） |
| 110 | `HitRegion` | 命中区域 |
| 111 | `Motion` | 动画容器 |
| 112 | `Presence` | 进出场容器 |
| 113 | `Reveal` | 高度揭示容器 |

通用规则：
- `Canvas` 的 children 是**指令节点**，声明顺序 = z-order（先者在下）。
- 图元参数用 `OpMethod`（数字/字符串/枚举）；路径/点在 `data`。
- 所有坐标/长度经 `Length` 语义：`ArgNumber` 的 `int` 值 = px，`0..1` = 比例；字符串 `"50%"`/`"auto"` 等显式。
- native 侧统一产出 `Vec<PaintCommand>`，在 Canvas 的 prepaint 收集、paint 重放。

---

## 2. P1 — 声明式绘制图元 + Canvas

### 托管
- 新增 `Elements/PaintPrimitives.cs`：
  - `PaintRect(x,y,w,h)`，方法 `.Fill("#..")`、`.Stroke(width)`、`.Color("#..")`、`.Radius(r)`；
  - `PaintLine(x1,y1,x2,y2)` `.Stroke(width).Color("#..").Dash(on,off)`；
  - `PaintPath(dsl)` `.Stroke(width).Fill("#..").Color("#..").Dash(on,off)`；
  - `PaintGradient(angle, from, to)` `.Rect(x,y,w,h).Radius(r)`；
  - `PaintShadow(dx,dy,blur,color)` `.Rect(...).Inset()`；
  - 统一基类 `PaintElement : Element`（可放入 `Canvas.Add(...)`）。
- 新增 `Elements/CanvasElement.cs`：`CanvasElement : Element`，`.Clip()`、`.Add(params Element[])`。
- `Rendering/RenderContext.cs`：工厂 `Canvas(id)`、`PaintRect(...)`、`PaintLine(...)`、`PaintPath(...)`、
  `PaintGradient(...)`、`PaintShadow(...)`。
- `Interop/NativeProtocol.cs`：新增组件常量 103..110（P1 只用到 103..108）。

### native
- `components/paint.rs`：
  - `pub(crate) enum PaintCommand { Rect{..}, Line{..}, Path{segments,style,..}, Gradient{..}, Shadow{..}, Image{..} }`；
  - 每个图元 materializer 解析方法/`data` → `Carrier<PaintCommand>`；
  - 路径 DSL 解析器 `parse_path(&str, scale: Size<Pixels>) -> Result<Vec<Seg>, String>`（`M/L/Q/C/A/Z`，坐标像素或 `f` 比例）；
  - `apply(command, window)`：`fill`/`quad`/`PathBuilder`/`linear_gradient`/`paint_drop_shadows`。
- `components/canvas.rs`：
  - `CanvasMaterializer`：`take_style` → `StyleRefinement`；`take_typed_children::<PaintCommand>` →
    `Vec<PaintCommand>`；返回 `CanvasElement { commands, style, prepaint_token: None, regions: vec![] }`。
  - `CanvasElement : gpui::Element`：`request_layout`（style）；`prepaint`（解析比例坐标）；
    `paint`（重放命令，`.clip` 时用 `paint_layer` + `ContentMask`）。P1 无回调、无区域。
- `components/mod.rs`：注册 `canvas`、`paint`。
- `schema.rs`：常量 + `SCHEMA_HASH = 0x6E65_7473_6865_6C57`；`mod.rs` catalog 测试补名称。
- `registry.rs`：新增 `ElementCallback::decode(&self, arguments) -> Result<Snapshot, String>`
  （供 P2 动态指令复用；P1 可不加）。

### 测试/示例
- native：`paint.rs` 单测（DSL 解析：合法/非法/比例/环）、`canvas.rs`（注册、命令收集）。
- managed：`RenderContextTests` 断言 Canvas 节点 id、子节点顺序、路径 data、方法 op。
- sample：`CanvasPage`（静态自绘：矩形+路径+渐变+阴影），`PageRegistry` 注册。
- 验收：`CanvasPage` 正常绘制；clean repaint 不重跑托管；内存回归无新增每帧滞留。

---

## 3. P2 — Canvas.Prepaint（动态指令） + 静态 HitRegion + click

### 托管
- `CanvasElement`：`.Prepaint(ulong token)`；`HitRegion(id, x,y,w,h)` + `.OnClick(token)`
  （复用 `RenderContext.RegisterCallback`）。
- `RenderContext`：`HitRegion(...)`；（`Prepaint` 用已有 `RegisterElement`/`[GpuiCallback]`）。
- 生成器：暂不改（`Element` kind 已可 `ui.RegisterElement`）。

### native
- `CanvasElement`：
  - `prepaint`：若 `prepaint_token` 存在，调用 `ElementCallback::decode(&["x","y","w","h"])`，
    把返回的快照节点解释为 `PaintCommand`/`HitRegion`，与静态命令合并；对每个区域
    `window.insert_hitbox(bounds, behavior)` 存入 prepaint state。
  - `paint`：重放命令；对每个区域注册 `window.on_mouse_event::<MouseDownEvent/UpEvent>`，
    自己做 down/up 配对（同 hitbox）→ 命中时调用 `callbacks.click(session, token)`。
- `components/canvas.rs` 增加命中测试与 click 合成（keyed state 保存 pending down hitbox id）。

### 测试/示例
- managed：区域节点/回调 token 编码测试。
- sample：`HitTestPage`（矩形命中 + 点击计数，`Update`/`Notify` 局部重绘）。
- 验收：点击区域触发 C# 回调；非命中不触发；滚动/裁剪下命中仍正确。

---

## 4. P3 — 动态区域 + hover/press/move/scroll + cursor + clip

### 托管
- `HitRegion` 增加 `.OnHoverEnter/.OnHoverExit/.OnPress/.OnRelease/.OnMove/.OnScroll`、
  `.Cursor(CursorKind)`、`.BlockMouse()`、`.BlockMouseExceptScroll()`。
- 事件回调经 `RegisterCallback(Action<EventValue>)`；约定 `String`=事件名、`Number`=区域序号。

### native
- `CanvasElement` 用 keyed state 记录 hovered region，发 `hover_enter/exit`；
  `invoke(token, CallbackValueString(event), number=index, ...)`。
- `MouseMoveEvent`/`ScrollWheelEvent` 路由；`window.set_cursor_style`；
  `HitboxBehavior::{Normal,BlockMouse,BlockMouseExceptScroll}`。
- `Clip` 选项：`paint_layer` + `ContentMask`（prepaint/paint 都裁剪）。

### 测试/示例
- sample：`HitTestPage` 扩展 hover 高亮 + cursor + 遮挡。
- 验收：hover 进出各一次；遮挡区域事件不穿透；cursor 切换正确。

---

## 5. P4–P5 — 动画（Motion / Presence / Reveal，spring/keyframes/stagger）

### P4 托管
- `Elements/MotionElement.cs`：
  - `MotionSpec`（`Transition(TimeSpan, Easing)`、`Spring(...)`、`Keyframes(...)`）；
  - `Easing`（`Linear/Ease/In/Out/InOut/CubicBezier/Steps`，映射 gpui-base `Easing`）；
  - `Motion(id, spec)` + `.Opacity(double)`、`.TranslateX/Y(double)`、`.Width/Height(Length)`、
    `.Color(...)`（目标值，每帧上传）；
  - `Presence(id, present, spec)` + `.Fade(from,to)`、`.SlideY(from,to)`；
  - `Reveal(id, progress)`。
- `RenderContext`：`Motion/Presence/Reveal` 工厂。

### P4 native（`components/motion.rs`）
- `MotionMaterializer`：`use_keyed_state(id)` 存采样状态；读目标值；
  用 `gpui_base::motion::{transition, spring, animate_keyframes}` 采样（活动时其内部
  `request_animation_frame`）；把结果套到 wrapper：
  `div().relative()` + 子 `absolute().left/top`（位移）、`.opacity()`、`.w/.h`。
- `PresenceMaterializer`：`Presence::new(id, present).transition(..).sample(window, cx)`，
  `should_render()` 决定挂载；`progress` 驱动 fade/slide。
- `RevealMaterializer`：`MotionReveal::new(id, progress, child)`。
- 读取 gpui-component `Theme::motion_tokens()` 暴露语义：托管侧 `Motion.Normal` /
  `Easing.Enter/Exit/Move` / `Spring.Control/Move`（以字符串 enum 传 native 映射）。
- `cx.reduce_motion()` 为真时直接取终值、不请求帧。

### P5
- `spring`（response/damping/travel/epsilon）与 `animate_keyframes`（`Keyframes`/`Timing`/
  `IterationCount`/`PlaybackDirection`/`Stagger`）暴露。
- sample：`AnimationPage`（开关过渡、弹簧滑块、循环脉冲、Presence 进出、Reveal 展开）。

### 验收
- 目标变化触发过渡；中途反向平滑；无每帧跨界；`reduce_motion` 生效；内存不随动画升。

---

## 6. P6–P7 — 源生成器、图片、`ICanvasView`、文档

- 生成器：`Prepaint` 别名（同 `Element` kind）+ `Measure` kind（`string M(double,double)`）。
- P7 可选：`PaintImage`（`paint_image`/`paint_svg`）与 `ICanvasView` 糖；
  图片需 asset 加载管线，单列。
- 文档：README + design 状态更新 + 示例页。

---

## 7. ABI / schema 变更步骤（每阶段按需）

1. `crates/gpui-net-shell/src/schema.rs`：追加组件常量；需要时 bump `SCHEMA_HASH`。
2. `crates/gpui-net-shell/src/components/mod.rs`：`register()` 追加；catalog 名称测试追加。
3. `src/GpuiNetShell/Interop/NativeProtocol.cs`：同步组件常量与 `SchemaHash`。
4. `tests/GpuiNetShell.Tests/NativeProtocolTests.cs`：更新 pinned 字面量。
5. `ABI_VERSION` 保持 `8`；`GpuiNetShellApi`/`GpuiNetCallbacks` 不改。
6. `dotnet run -- --check` 必须报 abi 8 / schema `0x6E65747368656C57`。

---

## 8. 测试矩阵

| 层 | 内容 |
|---|---|
| native 单测 | DSL 解析、命令构建、Canvas 注册、hitbox 行为、motion 采样 |
| managed 单测 | 节点/op/data 编码、回调 token、Motion/Presence 节点、区域事件参数 |
| 生成器 | 示例项目编译（`Prepaint`/`Measure` token 生成） |
| 契约 | schema/abi pinned 字面量 |
| 冒烟 | `--page=<新页>`、`--check` |
| 内存 | `--cycle-pages` + `--cycle-set`（新页 vs 轻页，确认无每帧累积） |

---

## 9. 验证命令（每阶段结束）

```
cargo fmt -p gpui-net-shell -- --check
cargo clippy -p gpui-net-shell --all-targets -- -D warnings
cargo test -p gpui-net-shell
dotnet build GpuiNetShell.slnx
dotnet test GpuiNetShell.slnx
dotnet run --project samples/GpuiNetShell.Sample -- --check
```

---

## 10. 风险与回滚

- 每帧 prepaint/measure 是唯一每帧跨界路径；设指令数上限并 benchmark。
- Canvas 自定义 `Element` 需手写 click 合成与命中态，注意与滚动/裁剪一致性。
- gpui-base motion 为外部子模块，锁定现有 API；若升级需回归动画。
- 每阶段独立提交，schema bump 只在首次新增组件时发生一次（P1），后续阶段仅加组件常量。

---

## 11. 执行记录

### P1 — 声明式绘制图元 + Canvas（已完成）
- 组件 id：`Canvas=103`、`PaintRect=104`、`PaintLine=105`、`PaintPath=106`、
  `PaintGradient=107`、`PaintShadow=108`；schema `…6C56 → …6C57`，ABI 保持 `8`。
- native：`components/paint.rs`（`PaintCommand`/`Coord`/DSL 解析/`apply`）、
  `components/canvas.rs`（`CanvasElement` 自定义 `Element` + `CanvasMaterializer`）。
- managed：`Elements/PaintPrimitives.cs`（`PaintLength` + 图元）、`Elements/CanvasElement.cs`、
  `RenderContext` 工厂。
- sample：`CanvasPage`（渐变/阴影/矩形/路径/虚线）。
- 测试：native `paint.rs`（长度解析、路径 DSL、错误命令）+ `canvas.rs`；
  managed `RenderContextTests.CanvasRecordsPaintPrimitivesAsChildren`；
  `NativeProtocolTests` 字面量更新。
- 验证：`cargo test` 78 通过；managed 74 通过；`--check` = abi 8 / schema `0x6E65747368656C57`；
  `CanvasPage` 冒烟正常、内存平台（`PaintPath` 触发 GPUI 一次性路径缓存，约 +50MB，非泄漏）。

> 备注：`CanvasPage` 的 `PaintPath` 会触发 GPUI 路径缓存（与虚线分隔线同源）；这是原生
> 路径绘制的固有一次性成本，平坦不增长。若需规避，可只用 quad 类图元。

### P2 — Canvas.Prepaint（动态指令）+ 静态 HitRegion + click（已完成）
- 组件 id 按注册顺序：`HitRegion = 109`（`PaintImage` 延后到 P7，故 P3/P4 的 id 顺延）。
  schema `…6C57 → …6C58`，ABI 保持 `8`。
- native：`paint.rs` 增加 `HitRegionSpec` + `HitRegionMaterializer`；`canvas.rs` 增加
  `prepaint` 回调（prepaint 阶段调 `ElementCallback::build`，把返回的 `CanvasElement`
  的命令/区域并入本帧）、`insert_hitbox`、down/up 合成 click（keyed `ClickState`）→
  `callbacks.click` + `invalidate`。
- managed：`CanvasElement.Prepaint(token)`、`HitRegionElement`（`OnClick`/`BlockMouse`）、
  `RenderContext.HitRegion(...)`。
- sample：`CanvasPage` 改为实体页——`Prepaint(PlotToken)` 每帧按 bounds 绘制，
  两个 `HitRegion` 点击更新实体状态并 `Notify`。
- 测试：managed `CanvasRecordsPrepaintAndHitRegions`；native 目录/注册测试。
- 验证：`cargo test` 78；managed 75；`--check` = abi 8 / schema `0x6E65747368656C58`；
  `CanvasPage` 冒烟无 `prepaint callback must return a Canvas` 报错。

> 说明：动态 prepaint 是本设计唯一的每帧跨界路径；`CanvasPage` 每帧经
> `render_element` 触发一次托管绘制回调，返回指令后由 native 重放。


