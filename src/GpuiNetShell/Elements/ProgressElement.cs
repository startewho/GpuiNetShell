using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A linear determinate or indeterminate progress indicator.</summary>
public sealed class ProgressElement : Element
{
    internal ProgressElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets percentage progress; the component clamps it to 0–100.</summary>
    public ProgressElement Value(double percent)
    {
        Arena.AddMethodNumber(Index, "value", percent);
        return this;
    }

    /// <summary>Enables indeterminate loading animation.</summary>
    public ProgressElement Loading(bool loading = true)
    {
        Arena.AddMethodNumber(Index, "loading", loading ? 1 : 0);
        return this;
    }

    /// <summary>Sets the accessible name.</summary>
    public ProgressElement AccessibilityLabel(string label)
    {
        Arena.AddMethodString(Index, "accessibility_label", label);
        return this;
    }

    /// <summary>Sets the semantic size.</summary>
    public ProgressElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }
}
