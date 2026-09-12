using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A step part accepted only as a direct <see cref="StepperElement"/> child. It
/// carries its native value to the parent and renders nothing on its own.
/// </summary>
public sealed class StepperItemElement : Element
{
    internal StepperItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Disables this step independently of its parent.</summary>
    public StepperItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Adds the step's label content.</summary>
    public StepperItemElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
