using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A table header accepting <see cref="TableRowElement"/> children.</summary>
public sealed class TableHeaderElement : Element
{
    internal TableHeaderElement(RenderContext ui, int index)
        : base(ui, index) { }

    public TableHeaderElement Add(params TableRowElement[] rows)
    {
        ArgumentNullException.ThrowIfNull(rows);
        foreach (var row in rows)
        {
            Arena.AddChild(Index, row.Index);
        }
        return this;
    }
}

/// <summary>A table body accepting <see cref="TableRowElement"/> children.</summary>
public sealed class TableBodyElement : Element
{
    internal TableBodyElement(RenderContext ui, int index)
        : base(ui, index) { }

    public TableBodyElement Add(params TableRowElement[] rows)
    {
        ArgumentNullException.ThrowIfNull(rows);
        foreach (var row in rows)
        {
            Arena.AddChild(Index, row.Index);
        }
        return this;
    }
}

/// <summary>A table footer accepting <see cref="TableRowElement"/> children.</summary>
public sealed class TableFooterElement : Element
{
    internal TableFooterElement(RenderContext ui, int index)
        : base(ui, index) { }

    public TableFooterElement Add(params TableRowElement[] rows)
    {
        ArgumentNullException.ThrowIfNull(rows);
        foreach (var row in rows)
        {
            Arena.AddChild(Index, row.Index);
        }
        return this;
    }
}

/// <summary>A table row accepting <see cref="TableHeadElement"/> and <see cref="TableCellElement"/> children.</summary>
public sealed class TableRowElement : Element
{
    internal TableRowElement(RenderContext ui, int index)
        : base(ui, index) { }

    public TableRowElement Add(params Element[] cells)
    {
        ArgumentNullException.ThrowIfNull(cells);
        foreach (var cell in cells)
        {
            Arena.AddChild(Index, cell.Index);
        }
        return this;
    }
}

/// <summary>A table header cell.</summary>
public sealed class TableHeadElement : Element
{
    internal TableHeadElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the number of columns occupied by the cell.</summary>
    public TableHeadElement ColSpan(int span)
    {
        Arena.AddMethodNumber(Index, "col_span", span);
        return this;
    }

    /// <summary>Centers the cell content.</summary>
    public TableHeadElement TextCenter()
    {
        Arena.AddMethod(Index, "text_center");
        return this;
    }

    /// <summary>Right-aligns the cell content.</summary>
    public TableHeadElement TextRight()
    {
        Arena.AddMethod(Index, "text_right");
        return this;
    }

    public TableHeadElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A table data cell.</summary>
public sealed class TableCellElement : Element
{
    internal TableCellElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the number of columns occupied by the cell.</summary>
    public TableCellElement ColSpan(int span)
    {
        Arena.AddMethodNumber(Index, "col_span", span);
        return this;
    }

    /// <summary>Centers the cell content.</summary>
    public TableCellElement TextCenter()
    {
        Arena.AddMethod(Index, "text_center");
        return this;
    }

    /// <summary>Right-aligns the cell content.</summary>
    public TableCellElement TextRight()
    {
        Arena.AddMethod(Index, "text_right");
        return this;
    }

    public TableCellElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A table caption.</summary>
public sealed class TableCaptionElement : Element
{
    internal TableCaptionElement(RenderContext ui, int index)
        : base(ui, index) { }

    public TableCaptionElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}

/// <summary>A simple table composed from typed table-part children.</summary>
public sealed class TableElement : Element
{
    internal TableElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the table's screen-reader accessible name.</summary>
    public TableElement AccessibilityLabel(string label)
    {
        Arena.AddMethodString(Index, "accessibility_label", label);
        return this;
    }

    /// <summary>Sets the table density.</summary>
    public TableElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    public TableElement Add(params Element[] parts)
    {
        ArgumentNullException.ThrowIfNull(parts);
        foreach (var part in parts)
        {
            Arena.AddChild(Index, part.Index);
        }
        return this;
    }
}
