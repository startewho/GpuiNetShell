using GpuiNetShell.Events;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>Base of a declared element. Holds its arena node index.</summary>
public abstract class Element
{
    private protected Element(RenderContext ui, int index)
    {
        Ui = ui;
        Index = index;
    }

    internal RenderContext Ui { get; }

    internal int Index { get; }

    internal RenderArena Arena => Ui.Arena;

    internal EventRegistry Events => Ui.Events;
}
