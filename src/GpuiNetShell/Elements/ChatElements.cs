using System.Globalization;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A file or image attachment; children compose into its content.</summary>
public sealed class AttachmentElement : Element
{
    internal AttachmentElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the lifecycle status: pending, uploading, processing, failed, complete.</summary>
    public AttachmentElement Status(string status)
    {
        Arena.AddMethodEnum(Index, "status", status);
        return this;
    }

    /// <summary>Sets the layout axis: horizontal or vertical.</summary>
    public AttachmentElement Axis(string axis)
    {
        Arena.AddMethodEnum(Index, "axis", axis);
        return this;
    }

    /// <summary>Sets the semantic attachment size.</summary>
    public AttachmentElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    public AttachmentElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A message bubble whose children form its visible content.</summary>
public sealed class BubbleElement : Element
{
    internal BubbleElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the message-edge alignment: start or end.</summary>
    public BubbleElement Alignment(string alignment)
    {
        Arena.AddMethodEnum(Index, "alignment", alignment);
        return this;
    }

    /// <summary>Sets the treatment: filled, secondary, muted, tinted, outline, ghost, destructive.</summary>
    public BubbleElement Variant(string variant)
    {
        Arena.AddMethodEnum(Index, "variant", variant);
        return this;
    }

    public BubbleElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A compact conversation status marker with composable children.</summary>
public sealed class MarkerElement : Element
{
    internal MarkerElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the treatment: plain, separator, or border.</summary>
    public MarkerElement Variant(string variant)
    {
        Arena.AddMethodEnum(Index, "variant", variant);
        return this;
    }

    public MarkerElement Loading(bool loading = true)
    {
        Arena.AddMethodNumber(Index, "loading", loading ? 1 : 0);
        return this;
    }

    /// <summary>Sets the loading treatment: spinner or shimmer.</summary>
    public MarkerElement LoadingStyle(string loadingStyle)
    {
        Arena.AddMethodEnum(Index, "loading_style", loadingStyle);
        return this;
    }

    public MarkerElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A message row whose children compose into its content slot.</summary>
public sealed class MessageElement : Element
{
    internal MessageElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the sender-edge alignment: start or end.</summary>
    public MessageElement Alignment(string alignment)
    {
        Arena.AddMethodEnum(Index, "alignment", alignment);
        return this;
    }

    public MessageElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>Theme-aware animated loading text.</summary>
public sealed class ShimmerTextElement : Element
{
    internal ShimmerTextElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets an explicit stable animation identity.</summary>
    public ShimmerTextElement Id(string id)
    {
        Arena.AddMethodString(Index, "id", id);
        return this;
    }

    /// <summary>Sets one shimmer sweep duration in milliseconds.</summary>
    public ShimmerTextElement DurationMs(double milliseconds)
    {
        Arena.AddMethodNumber(Index, "duration_ms", milliseconds);
        return this;
    }

    /// <summary>Sets the relative highlight half-width.</summary>
    public ShimmerTextElement Spread(double spread)
    {
        Arena.AddMethodNumber(Index, "spread", spread);
        return this;
    }

    public ShimmerTextElement Reverse(bool reverse = true)
    {
        Arena.AddMethodNumber(Index, "reverse", reverse ? 1 : 0);
        return this;
    }

    public ShimmerTextElement Once(bool once = true)
    {
        Arena.AddMethodNumber(Index, "once", once ? 1 : 0);
        return this;
    }
}

/// <summary>A virtualized message transcript with retained scroll state.</summary>
public sealed class MessageScrollerElement : Element
{
    internal MessageScrollerElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Enables its virtual-list scrollbar.</summary>
    public MessageScrollerElement Scrollbar(bool scrollbar = true)
    {
        Arena.AddMethodNumber(Index, "scrollbar", scrollbar ? 1 : 0);
        return this;
    }

    /// <summary>Enables the jump-to-latest button.</summary>
    public MessageScrollerElement JumpButton(bool jumpButton = true)
    {
        Arena.AddMethodNumber(Index, "jump_button", jumpButton ? 1 : 0);
        return this;
    }

    /// <summary>Sets the jump-to-latest button label.</summary>
    public MessageScrollerElement JumpButtonLabel(string label)
    {
        Arena.AddMethodString(Index, "jump_button_label", label);
        return this;
    }

    /// <summary>
    /// Renders each transcript row with managed code. The renderer receives the
    /// row index; returning <see langword="null"/> renders an empty row.
    /// </summary>
    public MessageScrollerElement RenderItem(Func<RenderContext, int, Element?> renderer)
    {
        ArgumentNullException.ThrowIfNull(renderer);
        var token = Events.RegisterElement(
            (context, arguments) =>
                renderer(
                    context,
                    int.Parse(arguments[0], CultureInfo.InvariantCulture)
                ) ?? context.Div()
        );
        Arena.AddCallback(Index, "render_item", token);
        return this;
    }
}
