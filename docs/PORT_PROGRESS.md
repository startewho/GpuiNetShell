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
| **P5 typed children** | `ChildElement` + `MaterializeRequest::take_children_of` / `child_component_names` | 父组件按注册名校验子组件；`DropdownMenu` 拒绝普通子节点 |

## 组件 id 表

| id | 名称 | id | 名称 | id | 名称 | id | 名称 |
|---|---|---|---|---|---|---|---|
| 0 | Div | 9 | Scroll | 18 | Kbd | 27 | StatusBar |
| 1 | Text | 10 | Scrollbar | 19 | Avatar | 28 | Alert |
| 2 | Button | 11 | Resizable | 20 | Icon | 29 | Tooltip |
| 3 | Label | 12 | Popover | 21 | Collapsible | 30 | HoverCard |
| 4 | Badge | 13 | Spinner | 22 | Pagination | 31 | DropdownMenu |
| 5 | Progress | 14 | Separator | 23 | Rating | 32 | DropdownButton |
| 6 | Combobox | 15 | Skeleton | 24 | Clipboard | | |
| 7 | Radio | 16 | Tag | 25 | Breadcrumb | | |
| 8 | Tabs | 17 | Link | 26 | GroupBox | | |

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

## 当前统计

- 已注册组件：**33**（id 0–32）。
- Charts（BarChart/LineChart/AreaChart/PieChart/RadarChart）按需求**跳过**。
- 剩余待移植：Accordion 家族、Empty 家族、Stepper、DescriptionList、Form/Field、
  Menu 家族、Sidebar 家族、Settings 家族、Command 家族、Tree、Table、DataTable、
  List、Select、Carousel 家族、Chat 家族、Retained forms（Input/NumberInput/
  OtpInput/Slider/ColorPicker/Calendar/DatePicker）、Textarea、Editor、Image、
  RadioGroup、TabBar/Tab、Window effects（Dialog/AlertDialog/Sheet/Notification）、
  NativeMenu 家族。

## 待补的框架能力

- **Array 参数**：`ArgumentSchema::Array`（Settings.keywords；Breadcrumb 现用换行串代替）。
- **P6 原生菜单**：NativeMenu 家族依赖 OS 菜单。
