using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A retained color picker with preview and commit behavior.</summary>
public sealed class ColorPickerElement : Element
{
    internal ColorPickerElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the visible label above the picker.</summary>
    public ColorPickerElement Label(string label)
    {
        Arena.AddMethodString(Index, "label", label);
        return this;
    }

    /// <summary>Sets the announced name independently of the visible label.</summary>
    public ColorPickerElement AccessibilityLabel(string label)
    {
        Arena.AddMethodString(Index, "accessibility_label", label);
        return this;
    }

    /// <summary>Reports the committed color as a hex string.</summary>
    public ColorPickerElement OnChange(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}

/// <summary>A retained calendar for date navigation and selection.</summary>
public sealed class CalendarElement : Element
{
    internal CalendarElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the number of adjacent months to display.</summary>
    public CalendarElement NumberOfMonths(int months)
    {
        Arena.AddMethodNumber(Index, "number_of_months", months);
        return this;
    }

    /// <summary>Reports the selected date as an ISO string.</summary>
    public CalendarElement OnChange(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}

/// <summary>A retained single-date picker.</summary>
public sealed class DatePickerElement : Element
{
    internal DatePickerElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the empty-value prompt.</summary>
    public DatePickerElement Placeholder(string placeholder)
    {
        Arena.AddMethodString(Index, "placeholder", placeholder);
        return this;
    }

    public DatePickerElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the chosen date as an ISO string.</summary>
    public DatePickerElement OnChange(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }
}
