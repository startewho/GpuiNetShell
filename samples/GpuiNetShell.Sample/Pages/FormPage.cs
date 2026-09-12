using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class FormPage : GalleryPage
{
    public override string Title => "Form";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Form",
            "Vertical and horizontal forms accepting Field children.",
            ui.Form()
                .Columns(2)
                .Add(
                    ui.Field().Label("Name").Add(ui.Text("Ada Lovelace")),
                    ui.Field().Label("Role").Required().Add(ui.Text("Engineer")),
                    ui.Field()
                        .Description("Optional contact email.")
                        .ColSpan(2)
                        .Add(ui.Text("ada@example.com"))
                ),
            ui.HForm()
                .Columns(2)
                .LabelWidth(90)
                .Add(
                    ui.Field().Label("Team").Add(ui.Text("Platform")),
                    ui.Field().Label("Time zone").Add(ui.Text("UTC"))
                )
        );
}
