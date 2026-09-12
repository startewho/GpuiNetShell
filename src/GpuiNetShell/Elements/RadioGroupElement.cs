using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A controlled radio set that composes <see cref="RadioElement"/> children and
/// reports the selected zero-based index.
/// </summary>
public sealed class RadioGroupElement : Element
{
    internal RadioGroupElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Controls the selected zero-based radio index.</summary>
    public RadioGroupElement SelectedIndex(int index)
    {
        Arena.AddMethodNumber(Index, "selected_index", index);
        return this;
    }

    public RadioGroupElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Reports the selected zero-based index.</summary>
    public RadioGroupElement OnChange(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler((int)value.Number));
        Arena.AddCallback(Index, "on_change", token);
        return this;
    }

    public RadioGroupElement Add(params RadioElement[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
