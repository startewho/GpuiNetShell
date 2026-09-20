using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A button-triggered native popup menu. Items are appended with
/// <see cref="Item"/> in call order.
/// </summary>
public sealed class DropdownMenuElement : Element
{
    internal DropdownMenuElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Uses the ghost (bare) button treatment for the trigger.</summary>
    public DropdownMenuElement Ghost()
    {
        Arena.AddMethod(Index, "ghost");
        return this;
    }

    /// <summary>Appends a command item that runs <paramref name="handler"/>.</summary>
    public DropdownMenuElement Item(string label, Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(handler);
        Arena.AddMethodStringCallback(Index, "item", label, token);
        return this;
    }
}
