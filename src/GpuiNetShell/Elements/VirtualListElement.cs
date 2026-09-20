using System.Globalization;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A virtualized list over managed data, vertical or horizontal. The native list
/// asks for one item index at a time through <see cref="RenderItem"/>.
/// </summary>
public sealed class VirtualListElement : Element
{
    internal VirtualListElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the fixed item size along the scroll axis, in pixels.</summary>
    public VirtualListElement ItemSize(double pixels)
    {
        Arena.AddMethodNumber(Index, "item_size", pixels);
        return this;
    }

    /// <summary>
    /// Supplies a per-item size for every item, as newline-separated numbers,
    /// allowing varying sizes. The count comes from the number of sizes.
    /// </summary>
    public VirtualListElement ItemSizes(Func<string> sizes)
    {
        ArgumentNullException.ThrowIfNull(sizes);
        var token = Events.RegisterRows(sizes);
        Arena.AddCallback(Index, "item_sizes", token);
        return this;
    }

    /// <summary>Scrolls vertically.</summary>
    public VirtualListElement Vertical()
    {
        Arena.AddMethodEnum(Index, "axis", "vertical");
        return this;
    }

    /// <summary>Scrolls horizontally.</summary>
    public VirtualListElement Horizontal()
    {
        Arena.AddMethodEnum(Index, "axis", "horizontal");
        return this;
    }

    /// <summary>Renders one item with managed code, receiving its index.</summary>
    public VirtualListElement RenderItem(Func<RenderContext, int, Element> renderer)
    {
        ArgumentNullException.ThrowIfNull(renderer);
        var token = Events.RegisterElement(
            (context, arguments) =>
                renderer(context, int.Parse(arguments[0], CultureInfo.InvariantCulture))
        );
        Arena.AddCallback(Index, "render_item", token);
        return this;
    }

    /// <summary>Reports the clicked item's index.</summary>
    public VirtualListElement OnSelect(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(
            Index,
            "on_select",
            Events.Register(value => handler((int)value.Number))
        );
        return this;
    }

    /// <summary>Reports the middle-clicked item's index (for example to open it in a new tab).</summary>
    public VirtualListElement OnMiddleClick(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(
            Index,
            "on_middle_click",
            Events.Register(value => handler((int)value.Number))
        );
        return this;
    }

    /// <summary>
    /// Adds a row right-click menu (<see cref="ContextMenuItemElement"/> /
    /// <see cref="ContextMenuSeparatorElement"/>). Item callbacks receive the
    /// right-clicked item index.
    /// </summary>
    public VirtualListElement RowMenu(params Element[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }

    /// <summary>
    /// Scrolls to <paramref name="index"/> once, when <paramref name="token"/>
    /// changes (use a monotonically increasing token).
    /// </summary>
    public VirtualListElement ScrollTo(int index, long token)
    {
        Arena.AddMethodNumber(Index, "scroll_to", index);
        Arena.AddMethodNumber(Index, "scroll_token", token);
        return this;
    }
}
