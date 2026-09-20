# 移植执行记录（component-shell → GpuiNetShell）

本文件记录把 `external/gpui-kit/crates/component-shell/src/shell` 的组件逐个移植到
`crates/gpui-net-shell` 的进度、ABI 变更与验证方式。**每完成一批更新一次。**

## 约定

- 组件 id = 注册顺序索引。**新组件一律追加**，既有 id 不再移动。
- 每批会 bump `SCHEMA_HASH`（`crates/gpui-net-shell/src/schema.rs` 与
  `src/GpuiNetShell/Interop/NativeProtocol.cs` 同步）。
- `ABI_VERSION` 在回调/记录布局变化时 bump；目前为 **4**。
- 每批验收：`cargo test` / `cargo fmt --check` / `cargo clippy -D warnings` /
  `dotnet build` / `dotnet test` / `--check`，并在 Sample 增加对应页面。

## 框架能力

| 能力 | 位置 | 说明 |
|---|---|---|
| P2 保留状态 | `MaterializeRequest::use_keyed_state` / `update_entity` | 按 key 跨帧复用原生实体；`Scroll`/`Scrollbar` 为示例 |
| Enum 参数 | `ARG_ENUM` / `StyleArg::Enum` | 组件方法的封闭字面量（`size`、`scroll_axis`…） |
| 类型化回调值 | ABI `invoke` 回调 | 回传 bool/number/string（`on_change` 等） |
| 多命名构造函数 | `build_payload` | 按导出名选择（`Separator`/`Alert` 变体） |
| 多参数构造/方法 | `CONSTRUCTOR_ARG_SEPARATOR`、`ARG_STRING_CALLBACK`、`GpuiNetOp.c` | 构造参数打包；方法第 2 参数（如 `item(label, callback)`） |
| **P3 元素参数 + 懒槽** | `ARG_ELEMENT`、`MaterializeRequest::resolve_element`、`take_slot_factory`、`NodeFactory`/`SlotFactory` | 方法可接收子元素（`trigger_element`、`left_content`），命名槽可重建（Popover/HoverCard 的 `content`） |
| **P4 行快照回调** | ABI `resolve_rows`、`MaterializeRequest::resolve_rows`、`EventRegistry.RegisterRows` | native 反向调用 managed 取 tab 分隔行，供 List/Select/DataTable |
| **P5 typed children** | `ChildElement` + `MaterializeRequest::take_children_of` / `take_typed_children` + `typed_child::Carrier<T>` | 父组件按注册名校验子组件，并可取出子组件携带的原生值（`Tab` → `TabBar`） |
| **P7 元素回调** | ABI `render_element`、`MaterializeRequest::resolve_element_callback`、`ElementCallback`、`EventRegistry.RegisterElement` | native 反向调用 managed，managed 在独立 arena 里渲染一个子树并回传，供 List/Select/DataTable 自定义渲染 |
| 双参数方法 | `ARG_STRING_CALLBACK`、`GpuiNetOp.c` | 方法第 2 参数（`item(label, callback)`） |

## 组件 id 表

| id | 名称 | id | 名称 | id | 名称 | id | 名称 |
|---|---|---|---|---|---|---|---|
| 0 | Div | 11 | Resizable | 22 | Pagination | 33 | Tab |
| 1 | Text | 12 | Popover | 23 | Rating | 34 | TabBar |
| 2 | Button | 13 | Spinner | 24 | Clipboard | 35 | List |
| 3 | Label | 14 | Separator | 25 | Breadcrumb | 36 | Select |
| 4 | Badge | 15 | Skeleton | 26 | GroupBox | 37 | DataTable |
| 5 | Progress | 16 | Tag | 27 | StatusBar | 38 | AccordionItem |
| 6 | Combobox | 17 | Link | 28 | Alert | 39 | Accordion |
| 7 | Radio | 18 | Kbd | 29 | Tooltip | 40 | StepperItem |
| 8 | Tabs | 19 | Avatar | 30 | HoverCard | 41 | Stepper |
| 9 | Scroll | 20 | Icon | 31 | DropdownMenu | 42 | DescriptionItem |
| 10 | Scrollbar | 21 | Collapsible | 32 | DropdownButton | 43 | DescriptionList |
| 44 | Field | 45 | Form | 46 | Input | 47 | NumberInput |
| 48 | Textarea | 49 | OtpInput | 50 | Slider | 51 | ColorPicker |
| 52 | Calendar | 53 | DatePicker | 54 | MenuItem | 55 | MenuSeparator |
| 56 | Menu | 57 | MenuBar | 58 | SidebarMenuItem | 59 | SidebarMenu |
| 60 | SidebarHeader | 61 | SidebarFooter | 62 | Sidebar | 63 | SidebarToggleButton |
| 64 | SettingItem | 65 | SettingGroup | 66 | SettingPage | 67 | Settings |
| 68 | TreeItem | 69 | Tree | 70 | TableHeader | 71 | TableBody |
| 72 | TableFooter | 73 | TableRow | 74 | TableHead | 75 | TableCell |
| 76 | TableCaption | 77 | Table | 78 | CommandItem | 79 | CommandGroup |
| 80 | CommandSeparator | 81 | Command | 82 | Attachment | 83 | Bubble |
| 84 | Marker | 85 | Message | 86 | ShimmerText | 87 | MessageScroller |
| 88 | RadioGroup | 89 | Dialog | 90 | AlertDialog | 91 | Sheet |
| 92 | Notification | 93 | Editor | 94 | NativeMenuItem | 95 | NativeMenuSeparator |
| 96 | NativeMenuTrigger | 97 | ContextMenuItem | 98 | ContextMenuSeparator | 99 | ContextMenu |

## 批次记录

| 批次 | 组件 | 新增 id | ABI | SCHEMA_HASH | Sample 页 | 状态 |
|---|---|---|---|---|---|---|
| 前置 | 13 组件移植（含适配 Tabs/Combobox/Resizable/Div） | 复用 0–12 | 2 | `…6C32` | 既有各页 | ✅ |
| P2 | 保留状态框架，改 `Scroll`/`Scrollbar` | — | 2 | `…6C32` | — | ✅ |
| Batch 1 | Spinner, Separator, Skeleton, Tag | 13–16 | 2 | `…6C33` | Feedback | ✅ |
| Batch 2 | Link, Kbd, Avatar, Icon | 17–20 | 2 | `…6C34` | Elements | ✅ |
| Batch 3 | Collapsible, Pagination, Rating, Clipboard | 21–24 | 2 | `…6C35` | Interactive | ✅ |
| 修复 | Popover 改用带样式表面 + 普通子节点承载 content | — | 2 | `…6C35` | Popover | ✅ |
| Batch 4 | Breadcrumb, GroupBox, StatusBar, Alert | 25–28 | 2 | `…6C36` | Display | ✅ |
| P3/P4/P5 | 元素参数+懒槽、行快照回调、typed children（并扩展 op 记录 `c` 字） | — | 4 | `…6C37`→`…6C38` | — | ✅ |
| Batch 5 | Tooltip, HoverCard, DropdownMenu, DropdownButton | 29–32 | 4 | `…6C39` | Menus | ✅ |
| Batch 10 | Tab, TabBar, List, Select, DataTable | 33–37 | 4 | `…6C3B` | Collections | ✅ |
| P7 | 元素回调：`render_element`（managed 渲染子树）；List/Select/DataTable 支持 `render_row`/`render_cell` | — | 5 | `…6C3C` | Collections（DataTable 自定义单元格） | ✅ |
| Batch 6 | AccordionItem, Accordion, StepperItem, Stepper（typed children；省略 `Accordion.on_toggle`，因需 `Send + Sync`） | 38–41 | 5 | `…6C3D` | Disclosure | ✅ |
| Batch 8 | DescriptionItem, DescriptionList, Field, Form（typed children） | 42–45 | 5 | `…6C3E` | Structure | ✅ |
| 输入监控 | ABI `input_event`：鼠标按下/抬起/移动、滚轮、键盘转发到托管层（`InputEvent` + `View.OnInput`） | — | 6 | `…6C3F` | Input Monitor | ✅ |
| 文字输入 | Input（保留 `InputState` + `on_change(string)`，订阅 `InputEvent::Change`） | 46 | 6 | `…6C40` | Input Monitor | ✅ |
| 文字输入 2 | NumberInput, Textarea, OtpInput, Slider（保留 state + `on_change`） | 47–50 | 6 | `…6C41` | Text Input | ✅ |
| 文字输入 3 | ColorPicker, Calendar, DatePicker（保留 state + `on_change`） | 51–53 | 6 | `…6C42` | Date & Color | ✅ |
| Batch 11 | MenuItem, MenuSeparator, Menu, MenuBar（适配为窗口内菜单栏，用 managed 回调替代 action） | 54–57 | 6 | `…6C43` | Menu Bar | ✅ |
| Batch 12 | SidebarMenuItem, SidebarMenu, SidebarHeader, SidebarFooter, Sidebar, SidebarToggleButton | 58–63 | 6 | `…6C44` | Sidebar | ✅ |
| Batch 13 | SettingItem, SettingGroup, SettingPage, Settings（typed children + 懒槽） | 64–67 | 6 | `…6C45` | Settings | ✅ |
| Batch 14 | TreeItem, Tree（保留 `TreeState`，按 id 同步并保留展开/选中） | 68–69 | 6 | `…6C46` | Tree | ✅ |
| Batch 15 | TableHeader/Body/Footer/Row/Head/Cell/Caption/Table（typed parts） | 70–77 | 6 | `…6C47` | Table | ✅ |
| Batch 16 | CommandItem, CommandGroup, CommandSeparator, Command（保留 `CommandState`；路径以 `"section,row"` 回传） | 78–81 | 6 | `…6C48` | Command | ✅ |
| Batch 17 | Attachment, Bubble, Marker, Message, ShimmerText, MessageScroller（保留 `MessageScrollerState`，行渲染走 managed 回调） | 82–87 | 6 | `…6C49` | Chat | ✅ |
| Batch 18 | RadioGroup（`Radio` 改用可渲染 `Part`，组内消费原生 `Radio`；`on_change(index)`） | 88 | 6 | `…6C4A` | Radio Group | ✅ |
| Batch 19 | Dialog, AlertDialog, Sheet, Notification（原生按钮触发 `WindowExt` 效果；`content` 懒槽 + `on_effect_error`） | 89–92 | 6 | `…6C4B` | Window Effects | ✅ |
| 修复 | Window effects 点击无反应：`gpui_component::Root` 只画子视图，sheet/dialog/notification 层需由应用根渲染；`Root::render` 补上 `render_sheet_layer`/`render_dialog_layer`/`render_notification_layer` | — | 6 | `…6C4B` | Window Effects | ✅ |
| Batch 20 | Editor（keyed `EditorState`；`value`/`language` 首次渲染生效，`appearance`/`bordered`/`readonly`/`aria_label`） | 93 | 6 | `…6C4C` | Editor | ✅ |
| Batch 21 | NativeMenuItem, NativeMenuSeparator, NativeMenuTrigger（OS 弹窗菜单；`ManagedMenuAction` 全局 action 监听把选择派发到托管回调） | 94–96 | 6 | `…6C4D` | Native Menu | ✅ |
| 样式 | 长度值完整移植：`int`=px、`double`(0–1)=百分比、`Length`=显式单位（auto/rem/relative）；所有长度样式方法三重载。 | — | 6 | `…6C4D` | 各页 | ✅ |
| 修复 | 声明为布尔的方法被静默丢弃（托管发 Number，Rust 匹配 Boolean）：在 `materialize::record_methods` 按方法参数 schema 做强制转换。修复 Collapsible 内容不显示等 | — | 6 | `…6C4D` | Collapsible | ✅ |
| 修复 | 宿主 `with_assets(())` 导致所有 SVG 图标缺失：改用 `gpui-kit-assets::Assets`。修复 Rating 不显示、侧栏无图标、Clipboard 无按钮 | — | 6 | `…6C4D` | Rating / Clipboard | ✅ |
| 修复 | Chat 页滚动记录未受限：`MessageScroller` 加 `.H(320)` 使其真正虚拟化，不再展开整段日志 | — | 6 | `…6C4D` | Chat | ✅ |
| 删除 | 移除旧的管理式 dialog/sheet/notification：ABI `open_dialog`/`close_dialog`/`open_sheet`/`close_sheet`/`push_notification`、`GpuiApplication` 方法、`Root` 自有层、`OverlaysPage`；改由 Window Effects 控件承担 | — | 6 | `…6C4D` | Window Effects | ✅ |
| Batch 22 | ContextMenuItem, ContextMenuSeparator, ContextMenu（右键菜单，包裹目标元素；条目走托管回调） | 97–99 | 6 | `…6C4E` | Context Menu | ✅ |
| 重设计 | DataTable：托管侧保留行对象，`DataTable(id, rowCount)` + `render_cell` 回调按 `[rowIndex, column]` 取行；`ContextMenuItem` 子项成为行右键菜单，回调收到行号 | — | 6 | `…6C4F` | DataTable | ✅ |
| 热重载 | `MetadataUpdateHandler`：Hot Reload 后清缓存并重绘所有会话（`GpuiApplication.InvalidateAll`） | — | 6 | `…6C4F` | — | ✅ |
| AOT | 确认 NativeAOT 可用（`dotnet publish -p:PublishAot=true`，AOT 二进制正常运行）；库加 `IsAotCompatible`，csproj 按配置选择 debug/release 原生宿主 | — | 6 | `…6C4F` | — | ✅ |
| 焦点 | 输入族 `Input`/`NumberInput`/`Textarea`/`OtpInput`/`Editor` 增加 `on_focus`/`on_blur`（订阅 `InputEvent`/`OtpEvent` 的 Focus/Blur）；C# `FocusExtensions.OnFocus/OnBlur` | — | 6 | `…6C50` | Focus | ✅ |
| 源生成器 | `GpuiNetShell.SourceGen`（Roslyn `IIncrementalGenerator`，analyzer 接入 sample）：`[GpuiCallbacks]`+`[GpuiCallback("Name")]` 生成 token 属性与 `RegisterGeneratedCallbacks(ref RenderContext)`；按签名推断 action/typed/rows/element 回调。DataTablePage 已改用生成的行渲染回调 | — | 6 | `…6C50` | DataTable | ✅ |
| 主题 | 主题模式 Light/Dark/System + 自定义语义色覆盖：ABI `set_theme`/`configure`；Rust `Theme::change` + `Theme::global_mut` 覆盖后 `sync_base`；C# `GpuiApplication.SetTheme(ThemeMode, colors)`、`ThemeColors`；Sample 新增 Theme 页 | — | 6 | `…6C50` | Theme | ✅ |
| 标题栏 | 自定义标题栏：ABI `configure(flags)`，`GpuiApplication.UseCustomTitlebar`；Rust 用 `TitleBar::window_options()` 建窗，`Root` 叠加 `gpui_component::TitleBar`（拖拽/最小化/最大化/关闭由组件处理） | — | 6 | `…6C50` | — | ✅ |
| 修复 | 深色主题文字不可见：`ShellView` 根写死白色 `bg(rgba(0xFFFFFFFF))`，改用 `cx.theme().background`；`Root` 已用 `theme.background/foreground`。深色下背景/文字均随主题 | — | 6 | `…6C50` | Theme | ✅ |
| 修复 | HotReload 不刷新：`MetadataUpdateHandler` 只在库程序集声明，运行时只识别“被编辑程序集”上的特性；在 sample 程序集补 `[assembly: MetadataUpdateHandler(typeof(GpuiNetShell.HotReload))]`，且 `Command::Invalidate` 追加 `window.refresh()` | — | 6 | `…6C50` | — | ✅ |
| 主题切换 | 自定义标题栏内加主题切换图标按钮（`Moon`/`Sun`），点击调 `Theme::change` 翻转深浅色；左右两个标题栏簇 + `stop_propagation`（同 gpui-kit story 写法） | — | 6 | `…6C50` | — | ✅ |
| Tree | 新增 `Tree.render_item((ctx, TreeItemInfo) => Element)`：托管侧按 `Index/Id/Label/Depth/Selected/IsFolder/IsExpanded` 自定义每个节点；Sample 用 C# 字典按 id 渲染标签+大小 | — | 6 | `…6C51` | Tree | ✅ |
| Chat | `Message` 增加 `name`/`time`/`avatar`（头/脚/头像槽）；Chat 页改为 5000 条各类消息（气泡变体、附件、Marker、Shimmer、头像/名字/时间）虚拟化展示 | — | 6 | `…6C51` | Chat | ✅ |
| 数据量 | DataTable 示例 20 万行（按索引虚拟化）；结构化 Table 示例 1000 行 | — | 6 | `…6C51` | DataTable / Table | ✅ |
| 样式 | 颜色样式支持命名色（`red`/`orange-500`/`blue-600` 等）：`style.rs::as_color` 在非 `#` 时回退 `gpui_component::try_parse_color` | — | 6 | `…6C51` | 各页 | ✅ |
| 新增 | VirtualList（id 100）：`VirtualList(id, itemCount)` + `item_size`/`axis`(vertical/horizontal)/`render_item(index)`，基于 gpui-base 双向虚拟列表并保留滚动；与 DataTable/Tree/List 同一 index 回调模式，可由 `[GpuiCallback]` 统一生成 | 100 | 6 | `…6C52` | Virtual List | ✅ |
| 新增 | Image（id 101）：按路径加载图片/SVG，`fit`(cover/contain/fill/none/scale_down)+样式尺寸/圆角；宿主 `FileAssets` 提供图标资产 + 文件系统回退，解码纹理由 gpui 资产缓存托管、无引用即释放 | 101 | 6 | `…6C52` | Image | ✅ |
| 修复 | VirtualList 滚动条与不定尺寸：包裹 viewport 并用 `vertical_scrollbar`/`horizontal_scrollbar`（同一保留句柄）；新增 `item_sizes(() => "28\n44\n…")` 支持逐项不同尺寸；`GpuiApplication.AlwaysShowScrollbars`（configure bit1）让滚动条常显 | — | 6 | `…6C53` | Virtual List | ✅ |
| 增强 | VirtualList 交互：`on_select(index)` 选中、`ContextMenuItem` 子项成为行右键菜单（回调收行号）、`scroll_to(index)`+`scroll_token(token)` 跳转到顶/底/指定行列；Sample 加 Top/Go to 50,000/Bottom 按钮、选中高亮与行菜单演示 | — | 6 | `…6C54` | Virtual List | ✅ |
| 修复 | HotReload 真正生效：`PublishAot=true` 会被 `dotnet watch` 判定为“不支持热重载”并改为整进程重启，因此界面无刷新；sample csproj 增加 `<StartupHookSupport Condition="'$(Configuration)' == 'Debug'">true</StartupHookSupport>`（仅 Debug 开启，AOT/Release 发布不受影响）。实测编辑 Render 后同进程 `UpdateApplication`→`InvalidateAll`→`Invalidate`，UI 立即更新 | — | 6 | `…6C54` | — | ✅ |
| 修复 | Virtual List 页铺满：页面改为 `v_flex().size_full()`，两个列表各 `flex_1().min_h(0)` 上下均分高度且撑满视口；`GalleryView` 内容区去掉包裹用 `Div`（自动高度会吞掉百分比/flex 尺寸），改为 `Scroll(...).P(24).Add(page)`；行改为 `items_start`（原 `items_center` 让内容在定高行内垂直居中而位移） | — | 6 | `…6C54` | Virtual List | ✅ |
| 新增 | 多窗口：每个顶层窗口 = 一个独立会话。ABI 加 `open_window(parent, flags) -> session` / `close_window(session)`；Rust `host` 用会话 id 分配器 + 每会话 ingress，`OpenChild` 命令在父窗口任务里 `cx.open_window`，`on_window_closed` 清理会话并回调托管；托管 `GpuiApplication.Session` 拆分每窗口的 view/arena/registry，`OpenWindow(Func<View>) -> WindowHandle`（Run 前入队、首帧 flush）、`WindowHandle.Invalidate/Close`；`ManagedMenuAction` 携带所属会话；`SessionId`。Sample 加 Multi-Window 页 + `SecondaryWindowView`，`--open-windows=N` / `--close-after=MS` 验证 | — | 7 | `…6C55` | Multi-Window | ✅ |
| 增强 | 每窗口标题栏可单独配置：`WindowOptions.UseCustomTitlebar`（`bool?`，`null` 继承父窗口设置，`true/false` 显式覆盖）；`OpenWindow(factory, options)` 重载；`Session.UseCustomTitlebar` 解析后按窗口下发 flags（含 always-show-scrollbars）。Sample 三个按钮：继承/自定义/系统，`--open-mixed` 一次开三种验证 | — | 7 | `…6C55` | Multi-Window | ✅ |
| 修复 | DataTable 只显示表头：表体（`flex_grow_1`）在自动高度父列中塌缩；host 改为 `w_full().min_h(160)`，调用方 `.H(...)` 可覆盖 | — | 5 | `…6C3C` | Collections | ✅ |

## 渲染与内存优化（P0–P5）

- **P1 生命周期/缓冲**：`GpuiApplication.Session.OnWindowClosed` 释放两个 `RenderArena`（非托管缓冲），`RenderArena` 加 finalizer；`PublishBuffer` 改几何增长，避免逐帧 `AlignedFree/Alloc`。
- **P2 按代准备**：新增 `PreparedNode`。`materialize::prepare` 在描述构建时（dirty）折叠每个节点的 style/payload/methods/slots/children；`materialize_node` 只借用，clean repaint 不再解析 ops、重建 payload、记录方法。
- **P3 克隆/原子**：`NodeFactory`/`ElementCallback` 持 `Rc<FrozenComponentRegistry>`；`ComponentPayload` 由 `Arc` 改 `Rc`；`resolve_ops` 用 `std::mem::take` 去掉每样式克隆；`prepare` 借用方法名。
- **P4 arena 编码**：`AppendUtf8` 直接编码进 `_utf8`（无 `byte[]` + `AddRange`）；`PublishBuffer` 用 `CollectionsMarshal.AsSpan`（去掉每帧 4 次 `ToArray()`）；不变名字（方法/回调/slot/enum/`on_click`）UTF-8 缓存；几何扩容。
- **P5 事件/实体**：`EventRegistry` 复用 generation/scope 列表，`Register(Action)` 不再包闭包；生成回调改为**一次注册、稳定 token**（`RegisterStable*` + 生成器守卫）；`GalleryPage` 默认页 token 缓存。
- **P5b 组件方法 opcode**：组件方法名不再上线，传 FNV-1a 64 位 `MethodOps` code；`FrozenComponentRegistry` 建 `code -> name` 表。`SCHEMA_HASH` → `…6C61`（`ABI_VERSION` 8 不变）。
- **`previous` 快照（曾删除，后恢复）**：一度只保留 `current`、替换即退休旧代，但 `ContentHost` 会晚一帧重新物化，屏幕上仍是旧代元素树，旧代 token 被提前退休后点击会丢。现已恢复保留一代 `previous`，作为该窗口的宽限期。
- **P6 组件专项**：`VirtualListView` 缓存统一尺寸向量（仅 `item_count`/`item_size` 变化时重建），`item_sizes`/`row_menu` 改 `Rc` 共享，行菜单不再逐行 clone；`CanvasElement` 的 `commands`/`regions` 改 `Rc`，`click`/`hover` keyed id 与每个 `HitRegion` 的 `SharedString` 在构建期算好，paint 不再 `format!`/分配；`resolve_rows` 复用 buffer 并按 token 缓存 `Rc<Vec<Row>>`（构建新描述时清空），`List`/`Select`/`VirtualList` 不再每次重绘回调托管。
- **P7 保留子树/增量重绘**：内容物化移入保留的 `ContentHost` 实体，只有推入新描述时才重渲染；`prepare` 为描述计算结构指纹（屏蔽回调 token，因为 token 每代都变但不改变界面），`rebuild` 在指纹相同且无错误时**保留已显示描述与其 generation**（token 继续有效），改为 retire 新 generation。这样冗余 `Invalidate()`（界面未变）不做任何原生物化。仍**未做**的是「变化描述逐节点 diff、只重建变化子树」——需要节点级稳定结构键（React key 语义），当前描述不带。
- **Div 元素事件（v1，Div-only）**：托管 `Div` 可按需订阅 GPUI 元素事件（`on_click`/`on_aux_click`/`on_hover`/`on_mouse_down|up|move|down_out|up_out`/`on_mouse_pressure`/`on_scroll_wheel`/`on_key_down|up`），只有订阅的事件才写 op、才注册 GPUI 监听；事件名即 GPUI 方法名。新增 `element_events.rs`（枚举/线名/负载编码/绑定 helper）与 `Events/ElementEvents.cs`（负载类型 + 解码）。有状态事件（click/aux_click/hover）需要稳定的 `element_id`（原 `click_id` 重命名）；move/scroll 不自动重绘。`SCHEMA_HASH` → `…6C62`（`ABI_VERSION` 8 不变）。
- **自定义标题栏内容**：`View.RenderTitleBar(ref RenderContext) -> Element?`（单侧）由托管渲染，`GpuiApplication` 每代把根节点写入 arena 的 `titlebar_root`；native `Root` 用同一 snapshot 物化后放进 `gpui_component::TitleBar`（窗口控制按钮与拖动仍由 native 负责）。`null` 时保留默认「标题 + 主题切换」。另加 `GpuiApplication.WindowTitle` / `WindowOptions.Title` 配置 OS 标题（`set_window_title` / `open_window` 带标题）。`ABI_VERSION` → **9**（`GpuiNetArena` 加 `titlebar_root`，新增 `set_window_title`，`open_window` 带标题；`SCHEMA_HASH` 不变）。
- **托管每帧分配**：基准测试 100 行（约 400 节点）从 **77,632 B/帧 → 21,576 B/帧（−72%）**。
- **release DLL**：`[profile.release]` 加 `lto="fat"`/`codegen-units=1`/`strip="symbols"`；配合去除 inspector，36,299,264 → **30,985,216 字节**。

## 样式（gpui style）覆盖

- **无参样式**：由 `style.rs` 的 `style_vocabulary!` 声明为封闭的 `(名字, 直接调用)` 表，
  按 `u16` opcode 映射；`StyleOps.cs` 镜像同一顺序，`style.rs` 的
  `the_managed_vocabulary_matches` 测试防止漂移。不再启用 `gpui-base/inspector`，
  也移除了反射表与 `Box<dyn Any>`。当前覆盖 `flex_row`/`flex_col`/`w_full`/`h_full`/
  `size_full`/`items_center`/`font_bold` 等 18 个无参样式；新增样式需追加到两张表并 bump `SCHEMA_HASH`。
- **带参样式**：手工绑定于 `style.rs::apply_param`（同样按 opcode 索引）。已覆盖：
  `aspect_ratio`、`col_start`/`col_end`/`col_span`、`row_start`/`row_end`/`row_span`、
  `grid_cols`/`grid_cols_min_content`/`grid_cols_max_content`、`grid_rows`/`grid_rows_min_content`/
  `grid_rows_max_content`、`line_clamp`、`scrollbar_width`、`text_align`、`text_overflow`、
  `text_decoration_color`；整数样式做精确范围校验（`to_u16`/`to_i16`/`to_usize`）。
- C# 对应类型化方法：`Flex`/`WFull`/`HFull`/`Flex1`、`AspectRatio`、`ColStart`/`ColEnd`/`ColSpan`、
  `RowStart`/`RowEnd`/`RowSpan`、`GridCols*`/`GridRows*`、`LineClamp`、`ScrollbarWidth`、
  `TextAlign(TextAlignKind)`、`TextOverflow`、`TextDecorationColor`。
- **不可移植**：`font(Font)`、`font_features(FontFeatures)` 需要跨 ABI 传完整 `Font`/特性集。

## 当前统计

- **Sample 已重构**：主界面为「左侧 Sidebar 导航 + 右侧内容区（Scroll）」；**一个控件一个页面**，
  每个页面单独一个文件，位于 `samples/GpuiNetShell.Sample/Pages/`（`GalleryPage` 基类 +
  `PageRegistry`）。DataTable 页面展示 **2000 行**自定义数据并逐格 `RenderCell`；List 页面
  展示 **500 行**并用 `RenderRow` 自定义行渲染。
- 修复：主界面右栏内容区不显示 / 内容落到 Sidebar 下方——**根因是 gpui 的 `div()` 默认
  `display: Block`，`flex_row`/`flex_col` 只在 `display:flex` 下生效**。`HStack`/`VStack`
  现先 `.Flex()`（display:flex）再设方向；并新增类型化样式方法 `Flex`/`WFull`/`HFull`/`Flex1`，
  Sample 中已无 `.Style(...)` 调用。

- 已注册组件：**100**（id 0–99）。
- Charts（BarChart/LineChart/AreaChart/PieChart/RadarChart）按需求**跳过**。
- `List`/`Select`/`DataTable` 现支持自定义渲染：`render_row((ctx, fields) => Element)`、
  `DataTable.render_cell((ctx, [row, column]) => Element)`。未提供回调时回退到内置文本行。
  `TabBar`/`Accordion`/`Stepper`/`DescriptionList`/`Form`/`Sidebar`/`Settings`/`Tree`/`Table`/
  `Command` 用 `Carrier<T>` 承载 typed children。`Attachment`/`Bubble`/`Marker`/`Message` 直接
  组合普通子元素为内容；`MessageScroller` 通过 `render_item((ctx, index) => Element?)` 自定义行。
- **文字输入**：`Input`、`NumberInput`、`Textarea`、`OtpInput`、`Slider`、`ColorPicker`、
  `Calendar`、`DatePicker` 已支持（`on_change`）。仅剩 `Editor`（LSP/语法高亮，重）。
- **输入监控**：鼠标（按下/抬起/移动）、滚轮、键盘（按下/抬起）通过 `View.OnInput` 回调到托管层。
- 剩余待移植：**无**（`component-shell` 可移植控件已全部完成）。
- **当前 gpui-component 缺失、无法移植**：Empty 家族、Carousel 家族、Chat 顶层（`Chat`）、Image。

## 待补的框架能力

- **Array 参数**：`ArgumentSchema::Array`（Settings.keywords；Breadcrumb/columns 现用换行串代替）。
- **P6 原生菜单**：NativeMenu 家族依赖 OS 菜单。
- **Send + Sync 回调**：`Accordion.on_toggle` 需要 `Send + Sync` 处理器，当前未暴露。
