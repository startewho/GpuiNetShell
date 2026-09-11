using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// An anchored surface. <see cref="Trigger"/> is what is on screen while closed
/// and <see cref="Content"/> is painted above the window when it opens.
/// </summary>
public sealed class PopoverElement : Element
{
    internal PopoverElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the trigger shown while the surface is closed.</summary>
    public PopoverElement Trigger(Element trigger)
    {
        ArgumentNullException.ThrowIfNull(trigger);
        Arena.AddSlot(Index, "trigger", trigger.Index);
        return this;
    }

    /// <summary>Sets the content painted when the surface opens.</summary>
    public PopoverElement Content(Element content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Arena.AddSlot(Index, "content", content.Index);
        return this;
    }

    /// <summary>Starts the surface open when it is uncontrolled.</summary>
    public PopoverElement DefaultOpen(bool open = true)
    {
        Arena.AddMethodNumber(Index, "default_open", open ? 1 : 0);
        return this;
    }

    /// <summary>Sets whether pressing outside closes the surface.</summary>
    public PopoverElement OverlayClosable(bool closable = true)
    {
        Arena.AddMethodNumber(Index, "overlay_closable", closable ? 1 : 0);
        return this;
    }
}
