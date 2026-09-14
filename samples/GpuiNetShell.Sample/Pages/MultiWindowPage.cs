using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// Demonstrates opening secondary top-level windows. Every window is an
/// independent session with its own view, state, and title-bar mode: a window
/// can inherit the primary window's custom title bar or use the system one.
/// </summary>
internal sealed class MultiWindowPage : GalleryPage<MultiWindowPage.State>
{
    internal sealed class State
    {
        public List<(WindowHandle Handle, string Title, bool Custom)> Windows { get; } = [];
        public int Opened { get; set; }
        public string Status { get; set; } = "No windows opened yet.";
    }

    public override string Title => "Multi-Window";

    protected override Element RenderState(State state, RenderContext ui, Context<State> cx)
    {
        var rows = new List<Element>(state.Windows.Count);
        for (var i = 0; i < state.Windows.Count; i++)
        {
            var (handle, title, custom) = state.Windows[i];
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
                                Update(
                                    (s, c) =>
                                    {
                                        s.Windows[index].Handle.Invalidate();
                                        c.Notify();
                                    }
                                )
                            ),
                        ui.Button($"close-{index}")
                            .Label("Close")
                            .Danger()
                            .OnClick(() =>
                                Update(
                                    (s, c) =>
                                    {
                                        s.Windows[index].Handle.Close();
                                        s.Status = $"Closed {s.Windows[index].Title}.";
                                        s.Windows.RemoveAt(index);
                                        c.Notify();
                                    }
                                )
                            )
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
                        ui.Label($"opened: {state.Opened}")
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.Label(state.Status).TextSize(12).TextColor("gray-500"),
                ui.Div(ui.VStack(rows.ToArray()).Gap(8)).WFull()
            )
            .Gap(16)
            .WFull();
    }

    private void OpenWindow(bool? useCustomTitlebar) =>
        Update(
            (s, c) =>
            {
                s.Opened++;
                var ordinal = s.Opened;
                var custom = useCustomTitlebar ?? Application.UseCustomTitlebar;
                var mode = custom ? "custom" : "system";
                var title = $"Child window {ordinal} ({mode})";
                WindowHandle? handle = null;
                var view = new SecondaryWindowView(Application, ordinal, title, () => handle?.Close());
                handle = Application.OpenWindow(() => view, new WindowOptions(useCustomTitlebar));
                s.Windows.Add((handle, title, custom));
                s.Status = $"Opened {title}.";
                c.Notify();
            }
        );
}
