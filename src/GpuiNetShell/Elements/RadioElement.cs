using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A controlled radio option.</summary>
public sealed class RadioElement : Element
{
    internal RadioElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the option label.</summary>
    public RadioElement Label(string label)
    {
        Arena.AddMethodString(Index, "label", label);
        return this;
    }

    /// <summary>Sets the controlled checked state.</summary>
    public RadioElement Checked(bool checkedValue = true)
    {
        Arena.AddMethodNumber(Index, "checked", checkedValue ? 1 : 0);
        return this;
    }

    public RadioElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Receives the activation. A radio can only ever report chosen.</summary>
    public RadioElement OnClick(Action handler)
    {
        var token = Events.Register(handler);
        Arena.AddCallback(Index, "on_click", token);
        return this;
    }
}
