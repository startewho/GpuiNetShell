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
| 96 | NativeMenuTrigger | | | | | | |

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
| 修复 | DataTable 只显示表头：表体（`flex_grow_1`）在自动高度父列中塌缩；host 改为 `w_full().min_h(160)`，调用方 `.H(...)` 可覆盖 | — | 5 | `…6C3C` | Collections | ✅ |

## 样式（gpui style）覆盖

- **无参样式**：由反射覆盖（`gpui_base::styled_ext_reflection_methods` + `gpui::styled_reflection::methods`
  + 手写的字重补充），`flex_row`/`flex_col`/`w_full`/`h_full`/`size_full`/`items_center`/
  `rounded_md`/`text_sm` 等数百个自动可用。
- **带参样式**：手工绑定于 `style.rs::apply_param`。本次补齐缺失项：
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

- 已注册组件：**97**（id 0–96）。
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
