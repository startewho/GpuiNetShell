using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>Controlled page navigation.</summary>
public sealed class PaginationElement : Element
{
    internal PaginationElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the current 1-based page.</summary>
    public PaginationElement CurrentPage(int page)
    {
        Arena.AddMethodNumber(Index, "current_page", page);
        return this;
    }

    /// <summary>Sets the positive page count.</summary>
    public PaginationElement TotalPages(int pages)
    {
        Arena.AddMethodNumber(Index, "total_pages", pages);
        return this;
    }

    /// <summary>Sets the maximum visible page buttons.</summary>
    public PaginationElement VisiblePages(int pages)
    {
        Arena.AddMethodNumber(Index, "visible_pages", pages);
        return this;
    }

    /// <summary>Shows only previous and next icon buttons.</summary>
    public PaginationElement Compact()
    {
        Arena.AddMethod(Index, "compact");
        return this;
    }

    /// <summary>Sets semantic size.</summary>
    public PaginationElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    public PaginationElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the page the reader asked for.</summary>
    public PaginationElement OnChange(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler((int)value.Number));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}
