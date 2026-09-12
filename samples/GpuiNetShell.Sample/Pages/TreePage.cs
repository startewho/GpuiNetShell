using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TreePage : GalleryPage
{
    public override string Title => "Tree";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Tree",
            "A retained tree with nested typed items; expansion and selection persist natively.",
            ui.Tree("files")
                .Full()
                .H(320)
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
