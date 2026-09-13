using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// Demonstrates opening secondary top-level windows. Every window is an
/// independent session with its own view, state, and event loop entry, so a
/// click in one window does not re-render another.
/// </summary>
internal sealed class MultiWindowPage : GalleryPage
{
    private readonly List<(WindowHandle Handle, string Title)> _windows = [];
    private int _opened;
    private string _status = "No windows opened yet.";

    public override string Title => "Multi-Window";

    public override Element Render(ref RenderContext ui)
    {
        var rows = new List<Element>(_windows.Count);
        for (var i = 0; i < _windows.Count; i++)
        {
            var (handle, title) = _windows[i];
            var index = i;
            rows.Add(
                ui.HStack(
                        ui.Label(title).FontMedium(),
                        ui
                            .Label(handle.SessionId == 0 ? "pending" : "open")
                            .TextSize(12)
                            .TextColor("gray-500"),
                        ui.Button($"focus-{index}")
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
                    "Open additional top-level windows, each with an independent managed session."
                ),
                ui.HStack(
                        ui.Button("open-window")
                            .Label("Open a window")
                            .Primary()
                            .OnClick(OpenWindow),
                        ui.Button("open-two").Label("Open two").OnClick(() =>
                        {
                            OpenWindow();
                            OpenWindow();
                        }),
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

    private void OpenWindow()
    {
        _opened++;
        var ordinal = _opened;
        var title = $"Child window {ordinal}";
        WindowHandle? handle = null;
        var view = new SecondaryWindowView(Application, ordinal, title, () => handle?.Close());
        handle = Application.OpenWindow(() => view);
        _windows.Add((handle, title));
        _status = $"Opened {title}.";
        Invalidate();
    }
}
