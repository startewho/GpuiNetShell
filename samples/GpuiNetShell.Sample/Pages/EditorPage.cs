using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class EditorPage : GalleryPage
{
    public override string Title => "Editor";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Editor",
            "A retained native source editor with syntax highlighting.",
            ui.Editor("code-editor")
                .Value("fn main() {\n    println!(\"hello, shell\");\n}\n")
                .Language("rust")
                .Bordered()
                .H(220)
        );
}
