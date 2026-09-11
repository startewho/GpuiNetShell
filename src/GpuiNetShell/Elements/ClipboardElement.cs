using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A button that copies a configured string to the system clipboard.</summary>
public sealed class ClipboardElement : Element
{
    internal ClipboardElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the text copied when the button is pressed.</summary>
    public ClipboardElement Value(string value)
    {
        Arena.AddMethodString(Index, "value", value);
        return this;
    }

    /// <summary>Sets the copy button tooltip.</summary>
    public ClipboardElement Tooltip(string tooltip)
    {
        Arena.AddMethodString(Index, "tooltip", tooltip);
        return this;
    }
}
