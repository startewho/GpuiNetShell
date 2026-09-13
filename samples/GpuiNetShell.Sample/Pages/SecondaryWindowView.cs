using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// A standalone window opened from the Multi-Window page. It has its own
/// session and state: its counter is independent of the main window, which is
/// the point of the example.
/// </summary>
internal sealed class SecondaryWindowView : View
{
    private readonly GpuiApplication _application;
    private readonly int _ordinal;
    private readonly string _title;
    private readonly Action _close;
    private int _count;

    public SecondaryWindowView(
        GpuiApplication application,
        int ordinal,
        string title,
        Action close
    )
    {
        _application = application;
        _ordinal = ordinal;
        _title = title;
        _close = close;
    }

    protected override Element Render(ref RenderContext ui) =>
        ui.VStack(
                ui.Label($"Window #{_ordinal}").TextSize(22).FontSemibold(),
                ui.Text(
                    "This is a separate native window with its own session. "
                        + "Its state is independent of the main window."
                ),
                ui.Div(ui.Label($"Title: {_title}").TextSize(12).TextColor("gray-500")),
                ui.HStack(
                        ui.Button("secondary-increment")
                            .Label($"Clicked {_count} times")
                            .Primary()
                            .OnClick(() =>
                            {
                                _count++;
                                Invalidate();
                            }),
                        ui.Button("secondary-close").Label("Close this window").OnClick(_close)
                    )
                    .Gap(8)
                    .ItemsCenter()
            )
            .Gap(16)
            .P(24)
            .Full();
}
