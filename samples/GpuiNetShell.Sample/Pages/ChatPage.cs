using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ChatPage : GalleryPage
{
    public override string Title => "Chat";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Chat",
            "Message bubbles, status markers, attachments, shimmer text, and a virtualized transcript.",
            Group(
                ref ui,
                "Bubbles",
                ui.Bubble()
                    .Alignment("start")
                    .Add(ui.Label("Hello from the managed shell runtime.")),
                ui.Bubble()
                    .Alignment("end")
                    .Variant("filled")
                    .Add(ui.Label("Bubbles align to their sender edge."))
            ),
            Group(
                ref ui,
                "Marker & Shimmer",
                ui.Marker("marker-today").Variant("separator").Add(ui.Label("Today")),
                ui.ShimmerText("Loading more messages…").DurationMs(1400).Spread(0.4)
            ),
            Group(
                ref ui,
                "Attachment",
                ui.Attachment("report")
                    .Status("complete")
                    .Axis("horizontal")
                    .Size(ControlSize.Medium)
                    .Add(ui.Label("quarterly-report.pdf"))
            ),
            Group(
                ref ui,
                "Message",
                ui.Message()
                    .Alignment("start")
                    .Add(ui.Label("A message row composes ordinary children into its content."))
            ),
            Group(
                ref ui,
                "Message Scroller",
                ui.MessageScroller("transcript", 2000)
                    .Scrollbar()
                    .JumpButton()
                    .JumpButtonLabel("Jump to latest")
                    .H(320)
                    .RenderItem((context, index) => context.Label($"Message {index + 1}"))
            )
        );

    private static Element Group(
        ref RenderContext ui,
        string title,
        params Element[] children
    )
    {
        var column = new List<Element> { ui.Label(title).FontSemibold() };
        column.AddRange(children);
        return ui.VStack(column.ToArray()).Gap(6);
    }
}
