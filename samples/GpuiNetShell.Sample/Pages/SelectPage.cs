using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SelectPage : GalleryPage<SelectPage.State>
{
    internal sealed class State
    {
        public string Selected { get; set; } = "";
    }

    public override string Title => "Select";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Select",
            "A retained single-value select backed by a row snapshot.",
            ui.Select("theme", Options, value =>
            {
                Update((s, c) =>
                {
                    s.Selected = value;
                    c.Notify();
                });
            })
                .Placeholder("Pick a theme")
                .RenderRow(
                    (ctx, fields) =>
                        ctx
                            .HStack(
                                ctx.Label(fields.ElementAtOrDefault(1) ?? ""),
                                ctx.Label(fields.ElementAtOrDefault(0) ?? "").TextSize(12)
                            )
                            .Gap(8)
                            .ItemsCenter()
                ),
            ui.Label($"Selected: {state.Selected}")
        );

    private static string Options() => "light\tLight\ndark\tDark\nsystem\tSystem";
}
