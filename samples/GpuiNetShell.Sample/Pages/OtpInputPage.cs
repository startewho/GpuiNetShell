using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class OtpInputPage : GalleryPage
{
    private string _code = "";

    public override string Title => "OtpInput";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "OtpInput",
            "A retained fixed-length one-time-password field.",
            ui.OtpInput("code").Length(6).Groups(2).OnChange(code =>
            {
                _code = code;
                Invalidate();
            }),
            ui.Label($"Code: {_code}")
        );
}
