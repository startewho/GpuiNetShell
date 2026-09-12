using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A retained native table backed by a row snapshot. Rows come from the
/// <c>DataTable</c> factory on <c>RenderContext</c>; each row is a tab-separated
/// list of cell strings.
/// </summary>
public sealed class DataTableElement : Element
{
    internal DataTableElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the column headers.</summary>
    public DataTableElement Columns(params string[] keys)
    {
        Arena.AddMethodString(Index, "columns", string.Join('\n', keys ?? []));
        return this;
    }

    /// <summary>Alternates row backgrounds.</summary>
    public DataTableElement Stripe(bool stripe = true)
    {
        Arena.AddMethodNumber(Index, "stripe", stripe ? 1 : 0);
        return this;
    }

    /// <summary>Draws cell borders.</summary>
    public DataTableElement Bordered(bool bordered = true)
    {
        Arena.AddMethodNumber(Index, "bordered", bordered ? 1 : 0);
        return this;
    }

    /// <summary>
    /// Renders each cell with managed code. The renderer receives
    /// <c>[row, column]</c>, where <c>row</c> is the tab-separated row, and
    /// returns the cell's element subtree.
    /// </summary>
    public DataTableElement RenderCell(
        Func<RenderContext, IReadOnlyList<string>, Element> renderer
    )
    {
        ArgumentNullException.ThrowIfNull(renderer);
        var token = Events.RegisterElement(renderer);
        Arena.AddCallback(Index, "render_cell", token);
        return this;
    }
}
