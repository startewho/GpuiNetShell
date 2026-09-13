using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TreePage : GalleryPage
{
    // The node data lives on the managed side; the native tree only asks for
    // one node at a time through RenderItem.
    private static readonly Dictionary<string, (string Size, string Status)> Nodes = new()
    {
        ["src"] = ("3 items", "folder"),
        ["main"] = ("2.1 KB", "rust"),
        ["lib"] = ("1.4 KB", "rust"),
        ["nested"] = ("1 item", "folder"),
        ["deep"] = ("320 B", "rust"),
        ["cargo"] = ("512 B", "toml"),
        ["readme"] = ("3.2 KB", "markdown"),
        ["target"] = ("2 items", "folder"),
        ["debug"] = ("—", "build"),
        ["release"] = ("—", "build"),
    };

    public override string Title => "Tree";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Tree",
            "A retained tree whose per-node content is rendered by managed code.",
            ui.Tree("files")
                .Full()
                .H(360)
                .RenderItem(
                    (ctx, node) =>
                    {
                        var meta = Nodes.GetValueOrDefault(node.Id);
                        var label = ctx
                            .Label(node.Label)
                            .FontMedium()
                            .TextColor(node.IsFolder ? "blue-600" : "gray-700");
                        if (string.IsNullOrEmpty(meta.Size))
                        {
                            return label;
                        }
                        return ctx
                            .HStack(
                                label,
                                ctx
                                    .Label(meta.Size)
                                    .TextSize(12)
                                    .TextColor("gray-500")
                            )
                            .Gap(8)
                            .ItemsCenter();
                    }
                )
                .Add(
                    ui.TreeItem("src", "src")
                        .Expanded()
                        .Add(
                            ui.TreeItem("main", "main.rs"),
                            ui.TreeItem("lib", "lib.rs"),
                            ui.TreeItem("nested", "nested").Add(ui.TreeItem("deep", "deep.rs"))
                        ),
                    ui.TreeItem("cargo", "Cargo.toml"),
                    ui.TreeItem("readme", "README.md").Disabled(),
                    ui.TreeItem("target", "target").Add(
                        ui.TreeItem("debug", "debug"),
                        ui.TreeItem("release", "release")
                    )
                )
        );
}
