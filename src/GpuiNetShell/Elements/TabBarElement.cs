using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A typed tab list accepting only <see cref="TabElement"/> children.</summary>
public sealed class TabBarElement : Element
{
    internal TabBarElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Controls the selected zero-based tab index.</summary>
    public TabBarElement SelectedIndex(int index)
    {
        Arena.AddMethodNumber(Index, "selected_index", index);
        return this;
    }

    /// <summary>Sets one of the component's five tab variants.</summary>
    public TabBarElement Variant(TabVariantKind variant)
    {
        Arena.AddMethodEnum(Index, "variant", variant switch
        {
            TabVariantKind.Outline => "outline",
            TabVariantKind.Pill => "pill",
            TabVariantKind.Segmented => "segmented",
            TabVariantKind.Underline => "underline",
            _ => "tab",
        });
        return this;
    }

    /// <summary>Enables the overflow menu.</summary>
    public TabBarElement Menu(bool menu = true)
    {
        Arena.AddMethodNumber(Index, "menu", menu ? 1 : 0);
        return this;
    }

    /// <summary>Sets the semantic component size.</summary>
    public TabBarElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Reports the selected zero-based tab index.</summary>
    public TabBarElement OnChange(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler((int)value.Number));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }

    /// <summary>Adds the tab children.</summary>
    public TabBarElement Add(params TabElement[] tabs)
    {
        ArgumentNullException.ThrowIfNull(tabs);
        foreach (var tab in tabs)
        {
            Arena.AddChild(Index, tab.Index);
        }
        return this;
    }
}
