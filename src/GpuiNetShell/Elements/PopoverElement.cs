using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A button-triggered popover. The trigger is built from the constructor's
/// <c>(id, label)</c>; <see cref="Content"/> is painted above the window when it
/// opens.
/// </summary>
public sealed class PopoverElement : Element
{
    internal PopoverElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the content painted when the surface opens.</summary>
    public PopoverElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }

    /// <summary>Positions the popover relative to its trigger.</summary>
    public PopoverElement CardAnchor(PopoverAnchor anchor)
    {
        Arena.AddMethodEnum(Index, "card_anchor", AnchorName(anchor));
        return this;
    }

    /// <summary>Sets the initial uncontrolled open state.</summary>
    public PopoverElement DefaultOpen(bool open = true)
    {
        Arena.AddMethodNumber(Index, "default_open", open ? 1 : 0);
        return this;
    }

    /// <summary>Controls whether the popover is open.</summary>
    public PopoverElement Open(bool open = true)
    {
        Arena.AddMethodNumber(Index, "open", open ? 1 : 0);
        return this;
    }

    /// <summary>Controls whether pressing outside dismisses the surface.</summary>
    public PopoverElement OverlayClosable(bool closable = true)
    {
        Arena.AddMethodNumber(Index, "overlay_closable", closable ? 1 : 0);
        return this;
    }

    /// <summary>Runs when pointer interaction changes the open state.</summary>
    public PopoverElement OnOpenChange(Action<bool> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.Boolean));
        Arena.AddCallback(Index, "on_open_change", token);
        return this;
    }

    private static string AnchorName(PopoverAnchor anchor) =>
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
