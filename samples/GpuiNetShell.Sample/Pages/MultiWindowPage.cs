using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// Demonstrates opening secondary top-level windows. Every window is an
/// independent session with its own view, state, and title-bar mode: a window
/// can inherit the primary window's custom title bar or use the system one.
/// </summary>
internal sealed class MultiWindowPage : GalleryPage
{
    private readonly List<(WindowHandle Handle, string Title, bool Custom)> _windows = [];
    private int _opened;
    private string _status = "No windows opened yet.";

    public override string Title => "Multi-Window";

    public override Element Render(ref RenderContext ui)
    {
        var rows = new List<Element>(_windows.Count);
        for (var i = 0; i < _windows.Count; i++)
        {
            var (handle, title, custom) = _windows[i];
            var index = i;
            rows.Add(
                ui.HStack(
                        ui.Label(title).FontMedium(),
                        ui
                            .Label(custom ? "custom titlebar" : "system titlebar")
                            .TextSize(12)
                            .TextColor("gray-500"),
                        ui
                            .Label(handle.SessionId == 0 ? "pending" : "open")
                            .TextSize(12)
                            .TextColor("gray-500"),
                        ui.Button($"invalidate-{index}")
                            .Label("Invalidate")
                            .OnClick(() =>
                            {
                                _windows[index].Handle.Invalidate();
                                Invalidate();
                            }),
                        ui.Button($"close-{index}")
                            .Label("Close")
                            .Danger()
                            .OnClick(() =>
                            {
                                _windows[index].Handle.Close();
                                _status = $"Closed {_windows[index].Title}.";
                                _windows.RemoveAt(index);
                                Invalidate();
                            })
                    )
                    .Gap(8)
                    .ItemsCenter()
            );
        }

        return ui.VStack(
                Section(
                    ref ui,
                    "Multi-Window",
                    "Open additional top-level windows, each with an independent managed session and its own title-bar mode."
                ),
                ui.Label("Title bar")
                    .TextSize(12)
                    .TextColor("gray-500"),
                ui.HStack(
                        ui.Button("open-inherit")
                            .Label("Open (inherit parent)")
                            .Primary()
                            .OnClick(() => OpenWindow(null)),
                        ui.Button("open-custom")
                            .Label("Open with custom")
                            .OnClick(() => OpenWindow(true)),
                        ui.Button("open-system")
                            .Label("Open with system")
                            .OnClick(() => OpenWindow(false)),
                        ui.Label($"opened: {_opened}")
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.Label(_status).TextSize(12).TextColor("gray-500"),
                ui.Div(ui.VStack(rows.ToArray()).Gap(8)).WFull()
            )
            .Gap(16)
            .WFull();
    }

    private void OpenWindow(bool? useCustomTitlebar)
    {
        _opened++;
        var ordinal = _opened;
        var custom = useCustomTitlebar ?? Application.UseCustomTitlebar;
        var mode = custom ? "custom" : "system";
        var title = $"Child window {ordinal} ({mode})";
        WindowHandle? handle = null;
        var view = new SecondaryWindowView(Application, ordinal, title, () => handle?.Close());
        handle = Application.OpenWindow(() => view, new WindowOptions(useCustomTitlebar));
        _windows.Add((handle, title, custom));
        _status = $"Opened {title}.";
        Invalidate();
    }
}
