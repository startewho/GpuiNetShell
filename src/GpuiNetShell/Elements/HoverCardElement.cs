using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A hover-triggered card. <see cref="TriggerElement"/> sets the hover target
/// (an element argument); <see cref="Content"/> sets the lazily materialized
/// surface.
/// </summary>
public sealed class HoverCardElement : Element
{
    internal HoverCardElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the element that owns the hover interaction.</summary>
    public HoverCardElement TriggerElement(Element trigger)
    {
        ArgumentNullException.ThrowIfNull(trigger);
        Arena.AddMethodElement(Index, "trigger_element", trigger.Index);
        return this;
    }

    /// <summary>Sets the card content.</summary>
    public HoverCardElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }

    /// <summary>Positions the card relative to its trigger.</summary>
    public HoverCardElement CardAnchor(PopoverAnchor anchor)
    {
        Arena.AddMethodEnum(Index, "card_anchor", AnchorName(anchor));
        return this;
    }

    /// <summary>Sets the hover-open delay in milliseconds (0–60000).</summary>
    public HoverCardElement OpenDelay(double milliseconds)
    {
        Arena.AddMethodNumber(Index, "open_delay", milliseconds);
        return this;
    }

    /// <summary>Sets the hover-close delay in milliseconds (0–60000).</summary>
    public HoverCardElement CloseDelay(double milliseconds)
    {
        Arena.AddMethodNumber(Index, "close_delay", milliseconds);
        return this;
    }

    /// <summary>Controls the component's popover surface styling.</summary>
    public HoverCardElement Appearance(bool appearance = true)
    {
        Arena.AddMethodNumber(Index, "appearance", appearance ? 1 : 0);
        return this;
    }

    /// <summary>Runs when pointer interaction opens or closes the card.</summary>
    public HoverCardElement OnOpenChange(Action<bool> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.Boolean));
        Arena.AddCallback(Index, "on_open_change", token);
        return this;
    }

    internal static string AnchorName(PopoverAnchor anchor) =>
        anchor switch
        {
            PopoverAnchor.TopCenter => "top_center",
            PopoverAnchor.TopRight => "top_right",
            PopoverAnchor.BottomLeft => "bottom_left",
            PopoverAnchor.BottomCenter => "bottom_center",
            PopoverAnchor.BottomRight => "bottom_right",
            PopoverAnchor.LeftCenter => "left_center",
            PopoverAnchor.RightCenter => "right_center",
            _ => "top_left",
        };
}
