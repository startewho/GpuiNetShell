using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class OtpInputPage : GalleryPage<OtpInputPage.State>
{
    internal sealed class State
    {
        public string Code { get; set; } = "";
    }

    public override string Title => "OtpInput";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx) =>
        Page(
            ref ui,
            "OtpInput",
            "A retained fixed-length one-time-password field.",
            ui.OtpInput("code").Length(6).Groups(2).OnChange(code =>
            {
                Update((s, c) => { s.Code = code; c.Notify(); });
            }),
            ui.Label($"Code: {state.Code}")
        );
}
