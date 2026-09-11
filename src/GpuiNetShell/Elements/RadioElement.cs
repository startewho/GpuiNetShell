using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A controlled radio option. A radio used on its own reports its click through
/// <see cref="OnChange"/> with the new checked value.
/// </summary>
public sealed class RadioElement : Element
{
    internal RadioElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the visible label.</summary>
    public RadioElement Label(string label)
    {
        Arena.AddMethodString(Index, "label", label);
        return this;
    }

    /// <summary>Overrides the announced name.</summary>
    public RadioElement AccessibilityLabel(string label)
    {
        Arena.AddMethodString(Index, "accessibility_label", label);
        return this;
    }

    /// <summary>Controls checked state.</summary>
    public RadioElement Checked(bool checkedValue = true)
    {
        Arena.AddMethodNumber(Index, "checked", checkedValue ? 1 : 0);
        return this;
    }

    /// <summary>Controls keyboard tab-stop participation.</summary>
    public RadioElement TabStop(bool tabStop = true)
    {
        Arena.AddMethodNumber(Index, "tab_stop", tabStop ? 1 : 0);
        return this;
    }

    /// <summary>Sets the semantic size.</summary>
    public RadioElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    public RadioElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports a click with the new checked value.</summary>
    public RadioElement OnChange(Action<bool> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.Boolean));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}
