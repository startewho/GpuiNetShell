using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// A standalone window opened from the Multi-Window page. It has its own
/// session and state: its counter is independent of the main window, which is
/// the point of the example.
/// </summary>
internal sealed class SecondaryWindowView : View
{
    private sealed class CounterState
    {
        public int Count { get; set; }
    }

    private readonly GpuiApplication _application;
    private readonly int _ordinal;
    private readonly string _title;
    private readonly Action _close;
    private Entity<CounterState>? _state;

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

    protected override Element Render(ref RenderContext ui)
    {
        var state = _state ??= _application.New<CounterState>(_ => new CounterState());
        return ui
            .Child(
                state,
                (s, ctx, _) =>
                    ctx
                        .VStack(
                            ctx.Label($"Window #{_ordinal}").TextSize(22).FontSemibold(),
                            ctx.Text(
                                "This is a separate native window with its own session. "
                                    + "Its state is independent of the main window."
                            ),
                            ctx.Div(
                                ctx.Label($"Title: {_title}").TextSize(12).TextColor("gray-500")
                            ),
                            ctx
                                .HStack(
                                    ctx.Button("secondary-increment")
                                        .Label($"Clicked {s.Count} times")
                                        .Primary()
                                        .OnClick(() =>
                                            state.Update(
                                                (st, c) =>
                                                {
                                                    st.Count++;
                                                    c.Notify();
                                                }
                                            )
                                        ),
                                    ctx.Button("secondary-close")
                                        .Label("Close this window")
                                        .OnClick(_close)
                                )
                                .Gap(8)
                                .ItemsCenter()
                        )
                        .Gap(16)
            )
            .P(24)
            .Full();
    }
}
