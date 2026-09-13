using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ChatPage : GalleryPage
{
    private const int MessageCount = 5000;

    private static readonly string[] Senders = ["Ada", "Grace", "Linus", "Margaret", "Alan"];

    public override string Title => "Chat";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Chat",
            $"{MessageCount} messages, virtualized: the transcript renders only the visible rows.",
            ui.MessageScroller("transcript", MessageCount)
                .Scrollbar()
                .JumpButton()
                .JumpButtonLabel("Jump to latest")
                .H(520)
                .RenderItem((context, index) => BuildMessage(context, index))
        );

    private static Element BuildMessage(RenderContext ui, int index)
    {
        var sender = Senders[index % Senders.Length];
        var outgoing = index % 3 == 0;
        var alignment = outgoing ? "end" : "start";

        // Every 11th row is a day separator.
        if (index % 11 == 0)
        {
            return ui.Marker($"marker-{index}")
                .Variant("separator")
                .Add(ui.Label($"Day {index / 11 + 1}"));
        }

        var message = ui.Message()
            .Alignment(alignment)
            .Name(sender)
            .Time($"10:{(index / 5) % 60:00}")
            .Avatar(sender);

        switch (index % 9)
        {
            case 1:
                message = message.Add(
                    ui.Bubble()
                        .Alignment(alignment)
                        .Variant("secondary")
                        .Add(ui.Text($"Attachment shared in message {index + 1}."))
                );
                return message.Add(
                    ui.Attachment($"attachment-{index}")
                        .Status("complete")
                        .Axis("horizontal")
                        .Size(ControlSize.Small)
                        .Add(ui.Label($"design-{index}.png"))
                );
            case 2:
                return message.Add(
                    ui.Bubble().Alignment(alignment).Add(ui.ShimmerText("Assistant is typing…"))
                );
            case 3:
                return message.Add(
                    ui.Bubble()
                        .Alignment(alignment)
                        .Variant("outline")
                        .Add(ui.Text($"Question {index + 1}: how should we render {index * 3} rows?"))
                );
            case 4:
                return message.Add(
                    ui.Bubble()
                        .Alignment(alignment)
                        .Variant("tinted")
                        .Add(
                            ui.VStack(
                                    ui.Label("Answer").FontSemibold(),
                                    ui.Text("Ask the managed side for one row at a time.")
                                )
                                .Gap(4)
                        )
                );
            default:
                return message.Add(
                    ui.Bubble()
                        .Alignment(alignment)
                        .Variant(outgoing ? "filled" : index % 2 == 0 ? "secondary" : "muted")
                        .Add(
                            ui.Text(
                                $"Message {index + 1}: the quick brown fox jumps over the lazy dog."
                            )
                        )
                );
        }
    }
}
