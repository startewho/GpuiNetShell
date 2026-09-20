using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>
/// The right-hand preview pane: it shows the selected file through the first
/// provider in <see cref="PreviewProviders"/> that accepts it. Text is read off
/// the UI thread; images load through the native image element.
/// </summary>
internal sealed partial class FileManagerView
{
    /// <summary>How much of a text file is read for the preview.</summary>
    private const int MaxPreviewBytes = 128 * 1024;

    private Element BuildPreviewPane(
        RenderContext ui,
        FileManagerState state,
        Context<FileManagerState> cx
    )
    {
        var index = state.SelectedIndex;
        var entry = index >= 0 && index < state.Visible.Count ? state.Visible[index] : null;

        var header = ui.HStack(
                ui.Label(entry?.Name ?? "预览").TextSize(12).FontSemibold().LineClamp(1),
                ui.Div().Flex1(),
                ui.Div(ui.Label("✕").TextSize(12))
                    .Size(22)
                    .Rounded(4)
                    .Flex()
                    .ItemsCenter()
                    .JustifyCenter()
                    .OnClick("fm-preview-close", TogglePreview)
            )
            .Gap(6)
            .ItemsCenter()
            .WFull();

        return ui.VStack(header, BuildPreviewBody(ui, state, entry))
            .Gap(8)
            .P(8)
            .WFull()
            .HFull()
            .BorderL(1)
            .BorderColor(Divider);
    }

    private Element BuildPreviewBody(RenderContext ui, FileManagerState state, FileEntry? entry)
    {
        if (entry is null)
        {
            return PreviewHint(ui, "选择一个文件以预览");
        }
        if (entry.IsDirectory)
        {
            return PreviewHint(ui, "文件夹");
        }
        var provider = PreviewProviders.Find(entry);
        if (provider is null)
        {
            return PreviewHint(ui, "暂不支持预览此类型");
        }
        if (state.PreviewPath != entry.FullPath)
        {
            return PreviewHint(ui, "加载中…");
        }
        if (state.PreviewLoading)
        {
            return ui.Div(
                    ui.VStack(ui.Spinner().Size(ControlSize.Small), ui.Label("加载中…").TextSize(12))
                        .Gap(6)
                        .ItemsCenter()
                )
                .Flex1()
                .MinH(0)
                .WFull()
                .Flex()
                .ItemsCenter()
                .JustifyCenter();
        }
        if (!string.IsNullOrEmpty(state.PreviewError))
        {
            return PreviewHint(ui, state.PreviewError!);
        }

        return ui.VStack(
                provider.Render(
                    ui,
                    new PreviewContext(
                        entry,
                        state.PreviewText,
                        state.PreviewTruncated,
                        state.PreviewLoading,
                        state.PreviewError
                    )
                ),
                state.PreviewTruncated
                    ? ui.Label("（仅显示文件开头部分）").TextSize(11)
                    : ui.Div()
            )
            .Gap(4)
            .Flex1()
            .MinH(0)
            .WFull();
    }

    private static Element PreviewHint(RenderContext ui, string text) =>
        ui.Div(ui.Label(text).TextSize(12))
            .Flex1()
            .MinH(0)
            .WFull()
            .Flex()
            .ItemsCenter()
            .JustifyCenter();

    /// <summary>Loads the preview for the entry at <paramref name="index"/>.</summary>
    private void LoadPreview(Context<FileManagerState> cx, FileManagerState state, int index)
    {
        state.ResetPreview();
        if (!state.ShowPreview || index < 0 || index >= state.Visible.Count)
        {
            return;
        }
        var entry = state.Visible[index];
        state.PreviewPath = entry.FullPath;
        var provider = PreviewProviders.Find(entry);
        if (entry.IsDirectory || provider is null || !provider.NeedsText)
        {
            return;
        }

        state.PreviewLoading = true;
        var path = entry.FullPath;
        cx.Spawn(
            _ => Task.FromResult(FileSystemService.ReadTextPreview(path, MaxPreviewBytes)),
            (s, result, context) =>
            {
                if (s.PreviewPath != path)
                {
                    return;
                }
                s.PreviewLoading = false;
                s.PreviewText = result.Text;
                s.PreviewError = result.Error;
                s.PreviewTruncated = result.Truncated;
                context.Notify();
            }
        );
    }

    private void TogglePreview() =>
        Update(
            (s, cx) =>
            {
                s.ShowPreview = !s.ShowPreview;
                LoadPreview(cx, s, s.SelectedIndex);
                cx.Notify();
            }
        );
}
