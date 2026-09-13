# 设计方案：C# 侧 View / Entity、通知与关注、父子数据传递

> 状态：**待审阅**。审阅通过后再进入执行。
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
    public TGlobal? TryGlobal<TGlobal>();              // 预留：全局状态
}
```

`Subscription`：

```csharp
public sealed class Subscription : IDisposable
{
    public void Dispose();     // 取消订阅
    public void Detach();      // 生命周期绑定到当前实体（随实体释放自动取消）
}
```

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
var counter = App.New<CounterState>(cx => new CounterState());

ui.Child(counter, (state, ui, cx) =>        // 渲染实体化子视图
    ui.HStack(
        ui.Label($"count={state.Count}"),
        ui.Button("inc").Label("+1").OnClick(() => cx.Update(s => s.Count++))
    ));
```

> `ui.Child(entity, render)` 会把该实体注册为原生 keyed entity，
> 于是 `cx.Notify()` 只重渲染这个子树。

### 4.3 通知（notify）语义

- `Context<T>.Notify()`：
  1. 标记该实体的原生 keyed state 为 dirty；
  2. 触发所有 `Observe` 订阅者（同步、按注册顺序）；
  3. 请求该子树重渲染（若该实体当前被渲染在某窗口里）。
- 映射到 ABI：复用现有 `invalidate` 但**带实体定位**（见 §5 ABI 变更）。

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

- C# `EntityId.Value` 直接作为原生 `use_keyed_state` 的 key（`SharedString`）。
- 渲染实体化子视图时，托管在 arena 里发一个新的 op：`OP_ENTITY_CHILD { a = entityId, b = childNodeIndex }`（或复用 `OP_SLOT` 的命名槽 + 新组件 `EntityHost`）。
  - 原生为该 key 建立/复用 `Entity<EntityHostView>`，`EntityHostView` 渲染托管给的子节点，并持有 `cx.subscribe` 到自身以响应 notify。
- `Notify` → ABI 新增 `notify_entity(session, entity_id)`（any-thread），原生 `Entity::update` + `cx.notify()`，只标记该实体 dirty 子渲染。

### 5.2 通知/事件的托管实现

- `Context<T>` 由 `EntityRegistry`（托管，按 `EntityId`）持有：
  - `observers[entityId] = List<(targetEntityId, handler)>`
  - `listeners[entityId][eventType] = List<(targetEntityId, handler)>`
- `Notify()`：遍历 observers 调 handler（同步）；handler 里通常会再 `Notify()` 自己，形成级联——需**防环**（记录正在 notify 的实体集合，或限制链深度）。
- `Emit(e)`：按 `typeof(e)` 查 listeners 派发。
- 生命周期：`Subscription.Dispose` 从表里移除；实体被 GC/显式释放时移除其所有入/出边。**弱引用**避免父→子→父 的强引用环。

### 5.3 ABI 变更（草案）

| 新增 | 签名 | 用途 |
|---|---|---|
| `notify_entity` | `(u64 session, u64 entity_id) -> i32` | 实体级重渲染 |
| 渲染 op | `OP_ENTITY_CHILD`（`code=6`?） | arena 表达「实体化子视图」 |
| 组件 | `COMPONENT_ENTITY_HOST = 102` | 承载实体子树的原生组件 |

> ABI 6→7 已在多窗口时用过；本次若加表项需 **ABI 8 + schema bump**，或把 `notify_entity` 复用 `invalidate` 的通道（带 entity 参数，另开表项）。

---

## 6. 线程与生命周期

- 现有：GPUI 线程 STA；托管回调都在该线程；`Invalidate()` 可跨线程（ingress）。
- 实体状态读写：**限定在 GPUI 线程**（与 GPUI 一致，避免锁）。`Context` 不跨线程；需要后台更新时用 `Invalidate`-style 的 ingress 回投（预留 `cx.Spawn`）。
- 实体所有权：`EntityRegistry`（托管静态，按 session? 或全局）持有强引用直到显式 `Dispose` 或窗口关闭；其余用弱引用。
- 子视图实体随其**父实体**释放（`Subscription.Detach` 绑定）。

---

## 7. 兼容与迁移

- 旧 `View` + `Render` + `Invalidate()` 不变，现有 60+ 页无需改。
- 新能力**增量**：`App.New<T>` / `ui.Child(...)` / `Context<T>`。
- 源生成器（`GpuiCallbacks`）可后续扩展，为实体视图生成 token 属性——本轮不强制。

---

## 8. 分阶段执行计划（审阅后）

| 阶段 | 内容 | 交付 |
|---|---|---|
| P1 | `Entity<T>`/`WeakEntity<T>`/`EntityId`/`EntityRegistry`（纯托管，无 ABI）+ 单元测试 | 可 `New/Read/Update/Downgrade` |
| P2 | `Context<T>` + `Notify` + `Observe`（托管内级联，无原生） | 观察者回调可跑 |
| P3 | `Subscribe`/`Emit` 类型化事件 + `Subscription` 生命周期 | 事件派发可跑 |
| P4 | 原生实体子树：`OP_ENTITY_CHILD` + `COMPONENT_ENTITY_HOST` + `notify_entity`（ABI bump） | 实体级局部重渲染 |
| P5 | Sample：`EntityPage`（计数器 + 列表选择 + 父子同步）+ 文档 | 可运行示例 |
| P6 | （可选）`cx.Spawn` / 全局状态 | — |

每阶段：`cargo test/fmt/clippy` + `dotnet build/test` + `--check` + 冒烟。

---

## 9. 待你确认的决策点

1. **范围**：先做纯托管 P1–P3（不改 ABI），还是直接到 P4（真·实体级局部重渲染）？
2. **API 命名**：`App.New<T>` / `ui.Child(entity, render)` / `Context<T>` 是否符合你的偏好？是否要更贴近 GPUI 的 `cx.new`/`entity.update` 英文命名？
3. **父子传递默认模式**：优先推荐「Props + 回调」，还是「共享 Entity」？
4. **线程模型**：是否严格 GPUI 单线程实体状态（推荐），还是允许托管侧加锁多线程访问？
5. **ABI**：是否接受为此 bump 到 ABI 8 / schema `…6C56`？
6. **与源生成器整合**：本轮是否需要生成实体视图的 token，还是纯手写 API？

---

## 10. 备选方案（若不想动 ABI）

**纯托管通知模型**：实体只用于「状态 + 观察者级联」，最终仍调用现有整页 `Invalidate()`。优点：零 ABI 风险，P1–P3 即可用；缺点：notify 是「整页重渲染」，没有 GPUI 的子树级优化。可作为 P1–P3 的落地形态，把 P4 作为后续增强。
