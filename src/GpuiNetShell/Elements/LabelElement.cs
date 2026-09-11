using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A styled text label with optional secondary text, masking, and highlights.
/// Styling is the shared surface in <see cref="StyleExtensions"/>.
/// </summary>
public sealed class LabelElement : Element
{
    internal LabelElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Adds muted secondary text.</summary>
    public LabelElement Secondary(string text)
    {
        Arena.AddMethodString(Index, "secondary", text);
        return this;
    }

    /// <summary>Controls whether the main text is masked.</summary>
    public LabelElement Masked(bool masked = true)
    {
        Arena.AddMethodNumber(Index, "masked", masked ? 1 : 0);
        return this;
    }

    /// <summary>Highlights matching text fragments.</summary>
    public LabelElement Highlights(string text)
    {
        Arena.AddMethodString(Index, "highlights", text);
        return this;
    }
}
