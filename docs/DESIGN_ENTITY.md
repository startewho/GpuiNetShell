# 设计方案：C# 侧 View / Entity、通知与关注、父子数据传递

> 状态：**P1–P6 已实现**。本文档已按实现回填。
> 目标：在 C# 侧复刻 GPUI 的 `Entity`/`Context`/`observe`/`subscribe`/`emit` 心智模型，
> 使多组件、可复用状态、父子通信有统一、可组合的写法，同时不破坏现有 `View`/`Render` ABI。

---

## 1. GPUI 的模型（参照）

GPUI 的核心（见 `gpui-pre-0.3.4/src/_ownership_and_data_flow.rs`）：

| 概念 | GPUI | 语义 |
|---|---|---|
| `Entity<T>` | `cx.new(\|cx\| T { .. })` | 由 `App` 持有所有权；句柄是「带类型标签的引用计数句柄」；只有拿到 `App`/`Context` 才能读写状态 |
| `read` | `entity.read(cx)` | 只读访问状态 |
| `update` | `entity.update(cx, \|this, cx\| ..)` | 可变访问；通过 `Context<T>` 触发通知/事件 |
| `downgrade` / `upgrade` | `entity.downgrade()` | 弱引用，避免循环持有 |
| `Context<T>` | 回调第二个参数 | 包裹 `App` + 绑定到某实体；提供 `notify`/`emit`/`observe`/`subscribe`/`spawn` |
| `cx.notify()` | | 通知「观察者」：re-render 订阅者 + `observe` 订阅者 |
| `cx.observe(&other, cb)` | | 当 `other` notify 时回调，回调拿到 `Entity<other>` 句柄 |
| `cx.subscribe(&other, cb)` | | 订阅 `other` 发出的**类型化事件**（需实现 `EventEmitter<E>`） |
| `cx.emit(event)` | | 向所有 subscribe 该类型的订阅者派发事件 |
| `Subscription` | `.detach()` 或持有并 drop | 订阅的生命周期由 `Subscription` 决定；drop 即取消 |
| `View` | `impl Render for X` | 可渲染；实体化后由 `Entity<X>` 提供身份（`entity_id`），notify 只重渲染该子树 |

关键点：**notify 与 emit 是两套解耦机制**——notify 表示「我的状态变了，重新渲染/同步我」；emit 表示「发生了一个语义事件，附带 payload」。observe 监听前者，subscribe 监听后者。

---

## 2. 现状（C# 侧约束）

- `View`（`View.cs`）：抽象类，`Render(ref RenderContext)`，`Invalidate()`，`OnInput`；由 `GpuiApplication.Session` 持有唯一根实例，每次 render 重建整棵树。
- 没有实体身份：所有状态都在「页面/视图对象」里，整页一变更即整树重渲染。
- 回调通过 `EventRegistry` 用 token 注册，快照 generation 退休时释放；**无订阅/观察关系**。
- 子组件目前是**无状态元素**（`ui.Button(...).OnClick(...)`），数据靠闭包捕获；列表/表格用 P7 `render_item` 回调把数据推给原生。
- 原生侧已有 `use_keyed_state`/`update_entity`（按 key 保留原生实体），是「身份」的现成载体。

---

## 3. 设计目标

1. **Entity**：C# 侧有可寻址、带类型、可选弱引用的状态单元，生命周期与原生 keyed entity 对齐。
2. **通知**：实体状态变化能只触发其所属子树重渲染（而非整页），并与 GPUI 的 notify 语义一致。
3. **关注**：`Observe`（监听 notify）与 `Subscribe`/`Emit`（类型化事件）两套机制。
4. **父子数据传递**：
   - 父 → 子：显式 `Props` + 子视图工厂（或直接传 Entity 句柄）；
   - 子 → 父：`Emit` 事件 + 父 `Subscribe`，或 `Observe` 同步，或回调委托；
5. 与现有 `View`/ABI **向后兼容**：旧写法继续可用。

---

## 4. 提议的 C# API

### 4.1 `Entity<T>` / `WeakEntity<T>` / `AppContext`

```csharp
public sealed class Entity<T> where T : class
{
    public EntityId Id { get; }
    public T Read();                       // 只读（无锁语义见 §6）
    public R Update<R>(Func<T, Context<T>, R> update);
    public void Update(Action<T, Context<T>> update);
    public WeakEntity<T> Downgrade();
    public bool IsAlive { get; }
}

public readonly struct WeakEntity<T> where T : class
{
    public Entity<T>? Upgrade();
}

public sealed class EntityId { public ulong Value { get; } }   // 全局单调
```

创建：

```csharp
AppContext.New<T>(Func<Context<T>, T> init) -> Entity<T>
```

`Context<T>`：

```csharp
public sealed class Context<T> where T : class
{
    public Entity<T> Entity { get; }
    public void Notify();                              // 状态变了
    public void Emit<TEvent>(TEvent e);                // 类型化事件
    public Subscription Observe<T2>(Entity<T2> target, Action<T, Entity<T2>, Context<T>> onNotify);
    public Subscription Subscribe<T2, TEvent>(Entity<T2> target, Action<T, Entity<T2>, TEvent, Context<T>> onEvent);
    public TGlobal? TryGlobal<TGlobal>();              // 进程级全局状态
    public void SetGlobal<TGlobal>(TGlobal value);      // 设置并触发重绘
    public void Spawn(Action<T, Context<T>> action);    // 任意线程 -> UI 线程
    public void Spawn<TResult>(                         // 后台计算，结果回 UI 线程
        Func<CancellationToken, Task<TResult>> work,
        Action<T, TResult, Context<T>> apply,
        Action<Exception>? onError = null);
}
```

全局状态另见 `GlobalStore`：按类型唯一，`GpuiApplication.SetGlobal<T>` / `Global<T>` 为便捷入口；设置时所有活动窗口重绘。

`Subscription`：

```csharp
public sealed class Subscription : IDisposable
{
    public static Subscription Empty { get; }
    public void Dispose();     // 取消订阅（幂等）
}
```

> `Detach()` 本轮未实现：订阅在 `Dispose()` 或**持有它的实体 `Release()`** 时取消
> （`EntityRegistry.Release` 会移除该实体所有出边）。C# 无确定性析构，故不依赖 GC。

### 4.2 `View` 升级：实体化视图

保留现有 `View`（无身份），新增**实体化视图**约定：

```csharp
public abstract class View : IView
{
    // 兼容：无身份。整页/整视图由 Session 持有。
    protected abstract Element Render(ref RenderContext ui);
    public void Invalidate();
    public void OnInput(Action<InputEvent>);
}

// 有身份的视图：状态放在实体里，View 只负责描述
public interface IEntityView<T> where T : class
{
    Element Render(T state, ref RenderContext ui, Context<T> cx);
}
```

用法（父页面渲染子组件）：

```csharp
var counter = app.New<CounterState>(cx => new CounterState());

ui.Child(counter, (state, ui, cx) =>        // 渲染实体化子视图
    ui.HStack(
        ui.Label($"count={state.Count}"),
        ui.Button("inc").Label("+1").OnClick(() =>
            counter.Update((s, c) => { s.Count++; c.Notify(); }))
    ));
```

> `ui.Child(entity, render)` 会把该实体注册为原生 keyed entity，
> 于是 `cx.Notify()` 只重渲染这个子树。`render` 委托每次（原生）重绘时重新执行，
> 因此它应只读取实体当前状态与其参数。

### 4.3 通知（notify）语义

- `Context<T>.Notify()`：
  1. 标记该实体的原生 keyed state 为 dirty；
  2. 触发所有 `Observe` 订阅者（同步、按注册顺序）；
  3. 请求该子树重渲染（若该实体当前被渲染在某窗口里）。
- 映射到 ABI：复用现有 `invalidate` 但**带实体定位**（见 §5 ABI 变更）。
- **实体子树内不自动重绘**：原生 `EntityHost` 的 element callback 以
  `without_invalidate` 构建（`HostContext.invalidate` 设为 no-op），因此实体子树里的
  控件回调不会强制整窗重建；只有显式 `Context.Notify()` 会通过 `notify_entity`
  重绘该子树。这是「手动触发、只重绘局部」的关键。非实体（旧 `View`）路径保持
  回调后整窗 `refresh` 的兼容行为。

### 4.4 关注（observe / subscribe / emit）

```csharp
// 观察：对方 notify 时同步
childCtx.Observe(parentEntity, (child, parent, cx) =>
{
    child.Label = parent.Read().Title;   // 同步派生状态
    cx.Notify();
});

// 订阅：对方 emit 类型化事件
childCtx.Subscribe<ParentState, ItemPicked>(parentEntity, (child, parent, ev, cx) =>
{
    child.SelectedId = ev.Id;
    cx.Notify();
});

// 发射
parentCtx.Emit(new ItemPicked(id));      // 或 cx.Entity 内部 emit
```

事件类型无约束（任意 `class`/`record`）；`Subscribe<T2, TEvent>` 靠泛型类型做多路分发（实现见 §5.2）。

### 4.5 父子数据传递（三种推荐模式）

1. **Props（父→子，单向）**：父在 Render 时把数据作为参数传给子的 render 委托；子不持有父。
   ```csharp
   ui.Child(childEntity, (state, ui, cx) => RenderWith(state, parentTitle, ui, cx));
   ```
2. **共享实体（父↔子，双向）**：父把自己的 `Entity<ParentState>` 传给子；子 `Observe`/`Read`/`Emit`。
3. **回调（子→父，命令式）**：父传委托给子（现有写法，保留）。

三者可混用：Props 传不可变数据，Entity 传可变共享状态，委托传一次性动作。

---

## 5. 与原生/ABI 的衔接

### 5.1 身份：EntityId ↔ 原生 keyed entity

- C# `EntityId.Value` 直接作为原生 keyed state 的 key。
- 渲染实体化子视图时，托管用新组件 `EntityHost`（`COMPONENT_ENTITY_HOST = 102`）表达：
  - 节点 data = 十进制 entity id；
  - 方法 `render_entity(callbackToken)` = 托管侧注册的「持久」element renderer（不随 snapshot generation 退休，见 §5.2）。
- 原生 `EntityHostMaterializer` 在 render 期间 `use_keyed_state("shell-entity:{id}")`
  建立/复用 `Entity<EntityHostView>`，设置 renderer，并把一个 notifier 注册进
  **窗口级** `EntityHosts` 表（`ShellView` 持有，key = entity id）。
- `Notify` → ABI `notify_entity(session, entity_id)`（any-thread）打成命令；
  GPUI 线程上 `Root::notify_entity` → `ShellView::notify_entity` 从 `EntityHosts`
  取出 notifier，`Entity::update` + `cx.notify()`，只标记该子树 dirty。

> 关键修正：**不能**在 render 之外调用 `window.use_keyed_state`——GPUI 会断言
> 调用处于 layout/prepaint。因此 notify 走的不是再次 `use_keyed_state` 查表，而是
> 渲染期间记下的 `EntityHosts` notifier。`ShellView::content` 每帧先清空该表，
> 由本帧的 `EntityHost` materialize 重新登记，于是离开树的实体会被释放。

### 5.2 通知/事件的托管实现

- `Context<T>` 由 `EntityRegistry`（托管，按 `EntityId`）持有：
  - `observers[entityId] = List<(targetEntityId, handler)>`
  - `listeners[entityId][eventType] = List<(targetEntityId, handler)>`
- `Notify()`：遍历 observers 调 handler（同步）；handler 里通常会再 `Notify()` 自己，形成级联——需**防环**（记录正在 notify 的实体集合，或限制链深度）。
- `Emit(e)`：按 `typeof(e)` 查 listeners 派发。
- 生命周期：`Subscription.Dispose` 从表里移除；实体被 GC/显式释放时移除其所有入/出边。**弱引用**避免父→子→父 的强引用环。

### 5.3 ABI 变更（已实现）

| 新增 | 签名 | 用途 |
|---|---|---|
| `notify_entity` | `(u64 session, u64 entity_id) -> i32` | 实体级重渲染 |
| 组件 | `COMPONENT_ENTITY_HOST = 102` | 承载实体子树的原生组件（data = entity id，方法 `render_entity`） |

> 未新增 op：复用现有 `OpMethod` / `OpCallback`。已 **ABI 7→8**、schema hash
> `…6C55 → …6C56`。托管 `GpuiNetShellApi` 已同步新增 `NotifyEntity` 表项。
> 附带新增 `EventRegistry.RegisterPersistentElement(entityId, renderer)`：实体
> 子树的 renderer 不随 generation 退休，按 entity id 原地替换。

---

## 6. 线程与生命周期

- 现有：GPUI 线程 STA；托管回调都在该线程；`Invalidate()` 可跨线程（ingress）。
- 实体状态读写：**限定在 GPUI 线程**（与 GPUI 一致，避免锁）。`Context` 不跨线程。
- 后台更新：`Context.Spawn` 把完成动作投到 `UiDispatcher`（进程级并发队列），
  队列在每帧 `Session.RenderInto` 开头、构建元素树之前于 UI 线程 `Drain`；
  `Post` 的唤醒回调是 `InvalidateAll`（复用现有 ingress，无需新 ABI）。
  `Spawn` 在应用动作前检查实体是否仍存活。
- 全局状态：`GlobalStore` 按类型唯一、加锁读写；`Set`/`Remove` 触发 `Changed`，
  应用据此 `InvalidateAll` 重绘所有窗口。
- 实体所有权：`EntityRegistry`（进程级）持有到显式 `Release`；`EntityHosts`
  notifier 表随窗口存活，且每帧重建。
- 订阅生命周期：`Subscription.Dispose()` 取消；持有它的实体 `Release()` 会移除其
  所有出边。子视图实体不被父实体自动绑定（无 `Detach`）。

---

## 7. 兼容与迁移

- 旧 `View` + `Render` + `Invalidate()` 不变，未实体化的视图照常工作。
- 新能力**增量**：`App.New<T>` / `ui.Child(...)` / `Context<T>`。
- 源生成器（`GpuiCallbacks`）可后续扩展，为实体视图生成 token 属性——本轮不强制。
- **Sample 迁移**：所有带状态的 gallery 页改为 `GalleryPage<TState>`（状态存实体、
  内容经 `ui.Child`、事件用 `Update((s, c) => { ...; c.Notify(); })`）。纯静态页没有
  可触发的更新，不实体化以免多一层 `EntityHost`。`SecondaryWindowView` 亦同理实体化。

---

## 8. 分阶段执行计划（审阅后）

| 阶段 | 内容 | 交付 | 状态 |
|---|---|---|---|
| P1 | `Entity<T>`/`WeakEntity<T>`/`EntityId`/`EntityRegistry`（纯托管，无 ABI）+ 单元测试 | 可 `New/Read/Update/Downgrade` | ✅ |
| P2 | `Context<T>` + `Notify` + `Observe`（托管内级联，无原生） | 观察者回调可跑 | ✅ |
| P3 | `Subscribe`/`Emit` 类型化事件 + `Subscription` 生命周期 | 事件派发可跑 | ✅ |
| P4 | 原生实体子树：`COMPONENT_ENTITY_HOST` + `render_entity` + `notify_entity`（ABI bump） | 实体级局部重渲染 | ✅ |
| P5 | Sample：`EntityPage`（计数器 + 列表选择 + 父子同步）+ 文档 | 可运行示例 | ✅ |
| P6 | （可选）`cx.Spawn` / 全局状态 | 后台结果回 UI 线程；`GlobalStore` | ✅ |

每阶段：`cargo test/fmt/clippy` + `dotnet build/test` + `--check` + 冒烟。

> 实测：`cargo test` 74 通过、`dotnet test` 68 通过（3 个需 manifest 的 native 测试跳过）、
> `dotnet run -- --check` 报 abi 8 / schema `0x6E65747368656C56`。

---

## 9. 决策点（回溯）

> 已按下列选择落地：范围做到 P6（含后台 `Spawn` 与全局状态）；命名用 `New<T>` /
> `Child(entity, render)` / `Context<T>`；父子默认「共享 Entity + 回调」；
> 实体状态严格 GPUI 单线程；接受 ABI 8 / schema `…6C56`；本轮不整合源生成器。

1. **范围**：先做纯托管 P1–P3（不改 ABI），还是直接到 P4（真·实体级局部重渲染）？
2. **API 命名**：`App.New<T>` / `ui.Child(entity, render)` / `Context<T>` 是否符合你的偏好？是否要更贴近 GPUI 的 `cx.new`/`entity.update` 英文命名？
3. **父子传递默认模式**：优先推荐「Props + 回调」，还是「共享 Entity」？
4. **线程模型**：是否严格 GPUI 单线程实体状态（推荐），还是允许托管侧加锁多线程访问？
5. **ABI**：是否接受为此 bump 到 ABI 8 / schema `…6C56`？
6. **与源生成器整合**：本轮是否需要生成实体视图的 token，还是纯手写 API？

---

## 10. 备选方案（若不想动 ABI）

**纯托管通知模型**：实体只用于「状态 + 观察者级联」，最终仍调用现有整页 `Invalidate()`。优点：零 ABI 风险，P1–P3 即可用；缺点：notify 是「整页重渲染」，没有 GPUI 的子树级优化。可作为 P1–P3 的落地形态，把 P4 作为后续增强。
