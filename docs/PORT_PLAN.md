# 移植总计划（GpuiNetShell ← component-shell）

本文件是**总计划 + 每次大执行前的规划模板**。执行进度与 ABI 变更记录在
`PORT_PROGRESS.md`；本文件记录目标、工作流、批次划分与前置能力。

## 1. 目标

把 `D:\Data\Code\gpui-component\crates\component-shell\src\shell` 的组件逐个移植到
`crates/gpui-net-shell`（Rust 原生宿主）+ `src/GpuiNetShell`（C# 托管层），每个组件
都要有：Rust 模块、组件 id、C# Element/工厂、Sample 页面、测试。

## 2. 架构约束与既有能力

| 能力 | 说明 | 状态 |
|---|---|---|
| P1 线协议 | node/op/child/arena + 反射样式 | ✅ |
| P2 保留状态 | `use_keyed_state` / `update_entity` | ✅ |
| P3 元素参数 + 懒槽 | `ARG_ELEMENT` / `resolve_element` / `take_slot_factory` / `NodeFactory` | ✅ |
| P4 行快照回调 | `resolve_rows` / `RegisterRows` | ✅ |
| P5 typed children | `Carrier<T>` / `take_typed_children` / `take_children_of` | ✅ |
| P6 原生菜单 | NativeMenu 家族（OS 菜单） | ⛔ 未做 |
| P7 元素回调 | `render_element` / `ElementCallback` / `RegisterElement` | ✅ |
| 多参数方法 | `ARG_STRING_CALLBACK` / `GpuiNetOp.c` | ✅ |
| Enum/多构造 | `ARG_ENUM` / 导出名选择 / `CONSTRUCTOR_ARG_SEPARATOR` | ✅ |

**未补**：Array 参数、`Send + Sync` 回调（`Accordion.on_toggle`）。

## 3. 工作流（每次大执行前必须规划）

1. **规划**：写清本批 4 个组件、依赖的前置能力、需要新增的 ABI、Sample 页、验证方式。
2. **Rust**：`components/<name>.rs`；`components/mod.rs` 追加注册（**既有 id 不变**）；
   追加 schema id；必要时 bump `SCHEMA_HASH`；更新 catalog 顺序测试。
3. **C#**：`NativeProtocol` 常量、`Elements/<Name>Element.cs`、`RenderContext` 工厂。
4. **Sample**：新增页面并挂到 tab。
5. **验证**：`cargo test` / `cargo fmt --check` / `cargo clippy -D warnings` /
   `dotnet build` / `dotnet test` / `--check`；必要时冒烟运行页面查 materialization 报错。
6. **记录**：更新 `PORT_PROGRESS.md`（批次、id、hash、统计、剩余）。

## 4. 组件批次（每批 4 个）

| 批次 | 组件 | 状态 |
|---|---|---|
| 前置 | 13 组件移植 | ✅ |
| Batch 1–5 | Spinner…DropdownButton | ✅ |
| Batch 6 | AccordionItem/Accordion/StepperItem/Stepper | ✅ |
| Batch 8 | DescriptionItem/DescriptionList/Field/Form | ✅ |
| Batch 10 | Tab/TabBar/List/Select/DataTable | ✅ |
| **Batch 11** | Menu / MenuBar / MenuItem / MenuSeparator | ⬜ 下一批 |
| Batch 12 | Sidebar 家族（Sidebar/Header/Footer/Menu/MenuItem/ToggleButton） | ⬜ |
| Batch 13 | Settings 家族（Settings/SettingPage/SettingGroup/SettingItem） | ⬜ |
| Batch 14 | Tree / TreeItem | ⬜ |
| Batch 15 | Table | ⬜ |
| Batch 16 | Command / CommandGroup / CommandItem / CommandSeparator | ⬜ |
| Batch 17 | Chat 部件：Attachment / Bubble / Marker / Message | ⬜ |
| Batch 18 | Chat 部件：ShimmerText / MessageScroller | ⬜ |
| Batch 19 | RadioGroup + Window effects（Dialog/AlertDialog/Sheet/Notification） | ⬜ |
| Batch 20 | RadioGroup 相关 + 收尾 | ⬜ |
| ⛔ 不可移植 | Empty 家族 / Carousel 家族 / Chat 顶层 / Image（依赖缺失模块） | ⛔ |

## 5. 文字输入工作流（新增，最高优先级）

保留型输入控件（`Input`/`NumberInput`/`Textarea`/`Editor`/`OtpInput`）需要：
- 原生 `InputState`/`TextareaState` 实体（P2 keyed state）。
- 订阅 `InputEvent::Change` 并把文本回传托管层（String 类型回调）。

| 组件 | 依赖 | 状态 |
|---|---|---|
| Input | `InputState` + `InputEvent::Change` | ✅ 已完成 |
| NumberInput | `InputState`（NumberInput） | ✅ 已完成 |
| Textarea | `TextareaState`（`InputEvent::Change`） | ✅ 已完成 |
| OtpInput | `OtpState`（`OtpEvent::Change`） | ✅ 已完成 |
| Slider | `SliderState`（`SliderEvent::Change`） | ✅ 已完成 |
| ColorPicker | `ColorPickerState` | ⬜ |
| Calendar / DatePicker | `CalendarState`/`DatePickerState` | ⬜ |
| Editor | `EditorState`（LSP/语法高亮，重） | ⬜ |

## 6. 输入监控（已完成）

把原生输入事件转发到托管层：鼠标按下/抬起/移动、滚轮、键盘按下/抬起。

- ABI：`input_event(session, kind, flags, a, b, c, text, text_len)`（ABI 6）。
- 原生：`ShellView` 根节点安装鼠标/滚轮/键盘处理器并转发。
- 托管：`InputEvent`/`InputEventKind`/`InputModifiers` + `View.OnInput`。
- Sample：**Input Monitor** 页，实时显示最近事件并带一个 `Input` 文本框。

## 7. 每次执行的规划模板

```
批次：Batch N
组件：A, B, C, D
前置：P? / 缺失能力
新增 ABI：无 / 描述
组件 id：N1–N4
Sample 页：名称
验证：cargo test / fmt / clippy / dotnet build / test / --check / 冒烟
```
