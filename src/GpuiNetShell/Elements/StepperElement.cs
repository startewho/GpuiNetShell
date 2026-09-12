using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A typed progress stepper accepting only <see cref="StepperItemElement"/> children.</summary>
public sealed class StepperElement : Element
{
    internal StepperElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Controls the current zero-based step.</summary>
    public StepperElement SelectedIndex(int index)
    {
        Arena.AddMethodNumber(Index, "selected_index", index);
        return this;
    }

    /// <summary>Switches between vertical and horizontal layout.</summary>
    public StepperElement Vertical(bool vertical = true)
    {
        Arena.AddMethodNumber(Index, "vertical", vertical ? 1 : 0);
        return this;
    }

    /// <summary>Centers each step's text in horizontal layouts.</summary>
    public StepperElement TextCenter(bool center = true)
    {
        Arena.AddMethodNumber(Index, "text_center", center ? 1 : 0);
        return this;
    }

    /// <summary>Disables every step.</summary>
    public StepperElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Sets the semantic component size.</summary>
    public StepperElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }

    /// <summary>Reports the newly selected zero-based index.</summary>
    public StepperElement OnChange(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler((int)value.Number));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }

    /// <summary>Adds the step children.</summary>
    public StepperElement Add(params StepperItemElement[] items)
    {
        ArgumentNullException.ThrowIfNull(items);
        foreach (var item in items)
        {
            Arena.AddChild(Index, item.Index);
        }
        return this;
    }
}
