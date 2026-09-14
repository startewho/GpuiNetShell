using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class RadioPage : GalleryPage<RadioPage.State>
{
    internal sealed class State
    {
        public int Index { get; set; } = 1;
    }

    public override string Title => "Radio";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "Radio",
            "Controlled radio options; selection is reported as a checked value.",
            ui.HStack(
                    ui.Radio("light")
                        .Label("Light")
                        .Checked(state.Index == 0)
                        .OnChange(_ =>
                        {
                            Update((s, c) =>
                            {
                                s.Index = 0;
                                c.Notify();
                            });
                        }),
                    ui.Radio("dark")
                        .Label("Dark")
                        .Checked(state.Index == 1)
                        .OnChange(_ =>
                        {
                            Update((s, c) =>
                            {
                                s.Index = 1;
                                c.Notify();
                            });
                        }),
                    ui.Radio("system")
                        .Label("System")
                        .Checked(state.Index == 2)
                        .OnChange(_ =>
                        {
                            Update((s, c) =>
                            {
                                s.Index = 2;
                                c.Notify();
                            });
                        })
                )
                .Gap(16)
                .ItemsCenter(),
            ui.Label($"Selected option: {state.Index}")
        );
}
