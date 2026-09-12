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

    /// <summary>
    /// Renders each row with managed code. The renderer receives the row's
    /// fields and returns the row's element subtree.
    /// </summary>
    public ListElement RenderRow(Func<RenderContext, IReadOnlyList<string>, Element> renderer)
    {
        ArgumentNullException.ThrowIfNull(renderer);
        var token = Events.RegisterElement(renderer);
        Arena.AddCallback(Index, "render_row", token);
        return this;
    }
}
