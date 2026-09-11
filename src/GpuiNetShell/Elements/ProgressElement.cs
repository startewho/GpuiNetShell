using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A determinate progress bar.</summary>
public sealed class ProgressElement : Element
{
    internal ProgressElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the completion percentage, from 0 to 100.</summary>
    public ProgressElement Value(double percent)
    {
        Arena.AddMethodNumber(Index, "value", percent);
        return this;
    }

    /// <summary>Shows the indeterminate loading animation.</summary>
    public ProgressElement Loading(bool loading = true)
    {
        Arena.AddMethodNumber(Index, "loading", loading ? 1 : 0);
        return this;
    }
}
