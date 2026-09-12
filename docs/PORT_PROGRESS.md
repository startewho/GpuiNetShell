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
| 修复 | DataTable 只显示表头：表体（`flex_grow_1`）在自动高度父列中塌缩；host 改为 `w_full().min_h(160)`，调用方 `.H(...)` 可覆盖 | — | 5 | `…6C3C` | Collections | ✅ |

## 当前统计

- **Sample 已重构**：主界面为「左侧 Sidebar 导航 + 右侧内容区（Scroll）」；**一个控件一个页面**，
  每个页面单独一个文件，位于 `samples/GpuiNetShell.Sample/Pages/`（`GalleryPage` 基类 +
  `PageRegistry`）。DataTable 页面展示 **2000 行**自定义数据并逐格 `RenderCell`；List 页面
  展示 **500 行**并用 `RenderRow` 自定义行渲染。
- 修复：主界面右栏内容区不显示——外层行原先 `ItemsStart` 且内容未 `FlexGrow`，导致滚动区高度塌陷。
  现改为：内容区 `Scroll().FlexGrow(1).H(420)`（页面 `Div` 用 `w_full`），侧栏 `H(420).FlexShrink(0)`。

- 已注册组件：**68**（id 0–67）。
- Charts（BarChart/LineChart/AreaChart/PieChart/RadarChart）按需求**跳过**。
- `List`/`Select`/`DataTable` 现支持自定义渲染：`render_row((ctx, fields) => Element)`、
  `DataTable.render_cell((ctx, [row, column]) => Element)`。未提供回调时回退到内置文本行。
  `TabBar`/`Accordion`/`Stepper`/`DescriptionList`/`Form` 用 `Carrier<T>` 承载 typed children。
- **文字输入**：`Input`、`NumberInput`、`Textarea`、`OtpInput`、`Slider`、`ColorPicker`、
  `Calendar`、`DatePicker` 已支持（`on_change`）。仅剩 `Editor`（LSP/语法高亮，重）。
- **输入监控**：鼠标（按下/抬起/移动）、滚轮、键盘（按下/抬起）通过 `View.OnInput` 回调到托管层。
- 剩余待移植：Menu 家族、Sidebar 家族、Settings 家族、Command 家族、Tree、Table、
  Chat 家族（Attachment/Bubble/Marker/Message/ShimmerText/MessageScroller）、其余保留型
  输入、RadioGroup、Window effects（Dialog/AlertDialog/Sheet/Notification）、NativeMenu 家族。
- **当前 gpui-component 缺失、无法移植**：Empty 家族、Carousel 家族、Chat 顶层、Image。

## 待补的框架能力

- **Array 参数**：`ArgumentSchema::Array`（Settings.keywords；Breadcrumb/columns 现用换行串代替）。
- **P6 原生菜单**：NativeMenu 家族依赖 OS 菜单。
- **Send + Sync 回调**：`Accordion.on_toggle` 需要 `Send + Sync` 处理器，当前未暴露。
