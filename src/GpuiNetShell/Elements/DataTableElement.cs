using System.Globalization;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A retained native table whose row objects stay on the managed side; the
/// native table asks for one row index at a time through <see cref="RenderCell"/>.
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
    /// Renders one cell with managed code. The renderer receives the row index
    /// and the column key, and indexes the managed row list itself.
    /// </summary>
    public DataTableElement RenderCell(
        Func<RenderContext, int, string, Element> renderer
    )
    {
        ArgumentNullException.ThrowIfNull(renderer);
        var token = Events.RegisterElement(
            (context, arguments) =>
                renderer(
                    context,
                    int.Parse(arguments[0], CultureInfo.InvariantCulture),
                    arguments[1]
                )
        );
        Arena.AddCallback(Index, "render_cell", token);
        return this;
    }

    /// <summary>
    /// Adds row right-click menu entries (<see cref="ContextMenuItemElement"/> /
    /// <see cref="ContextMenuSeparatorElement"/>). Item callbacks receive the
    /// right-clicked row index.
    /// </summary>
    public DataTableElement RowMenu(params Element[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}
