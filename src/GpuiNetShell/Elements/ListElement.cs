using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A retained native list backed by an immutable row snapshot. Rows come from
/// the <c>List</c> factory on <c>RenderContext</c>.
/// </summary>
public sealed class ListElement : Element
{
    internal ListElement(RenderContext ui, int index)
        : base(ui, index) { }
}
