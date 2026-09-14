using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class ButtonPage : GalleryPage<ButtonPage.State>
{
    internal sealed class State
    {
        public int Count { get; set; }
    }

    public override string Title => "Button";

    // The source generator produced `PageToken` and registered `RenderPage` as
    // this page's entity-view renderer; the base renders `ui.Child(Entity, token)`.
    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Button",
            "Variants, sizes, and the loading/disabled states.",
            ui.HStack(
                    ui.Button("primary").Label("Primary").Primary(),
                    ui.Button("secondary").Label("Secondary").Secondary(),
                    ui.Button("danger").Label("Danger").Danger(),
                    ui.Button("success").Label("Success").Success(),
                    ui.Button("ghost").Label("Ghost").Ghost(),
                    ui.Button("link").Label("Link").Link()
                )
                .Gap(8)
                .ItemsCenter(),
            ui.HStack(
                    ui.Button("small").Label("Small").Size(ButtonSize.Small),
                    ui.Button("medium").Label("Medium").Size(ButtonSize.Medium),
                    ui.Button("large").Label("Large").Size(ButtonSize.Large),
                    ui.Button("loading").Label("Loading").Loading(),
                    ui.Button("disabled").Label("Disabled").Disabled()
                )
                .Gap(8)
                .ItemsCenter(),
            ui.Button("counter")
                .Label($"Clicked {state.Count} times")
                .OnClick(() =>
                    Update(
                        (s, c) =>
                        {
                            s.Count++;
                            c.Notify();
                        }
                    )
                )
        );
}
