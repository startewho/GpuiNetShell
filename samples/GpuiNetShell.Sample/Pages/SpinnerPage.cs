using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class SpinnerPage : GalleryPage
{
    public override string Title => "Spinner";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Spinner",
            "Cycling loading spinners with sizes, icons, and easing.",
            ui.HStack(
                    ui.Spinner(),
                    ui.Spinner().Size(ControlSize.Small),
                    ui.Spinner().Size(ControlSize.Large),
                    ui.Spinner().Icon(SpinnerIcon.LoaderCircle).Color("blue-600"),
                    ui.Spinner().Ease(SpinnerEase.EaseOutQuint)
                )
                .Gap(20)
                .ItemsCenter()
        );
}
