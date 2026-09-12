using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A typed native-menu item; consumed by a <see cref="NativeMenuTriggerElement"/>.</summary>
public sealed class NativeMenuItemElement : Element
{
    internal NativeMenuItemElement(RenderContext ui, int index)
        : base(ui, index) { }

    public NativeMenuItemElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Sets the native menu item checked state.</summary>
    public NativeMenuItemElement Checked(bool checkedValue = true)
    {
        Arena.AddMethodNumber(Index, "checked", checkedValue ? 1 : 0);
        return this;
    }

    /// <summary>Runs when the native menu item is selected.</summary>
    public NativeMenuItemElement OnSelect(Action handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        Arena.AddCallback(Index, "on_select", Events.Register(handler));
        return this;
    }
}

/// <summary>A typed native-menu separator.</summary>
public sealed class NativeMenuSeparatorElement : Element
{
    internal NativeMenuSeparatorElement(RenderContext ui, int index)
        : base(ui, index) { }
}

/// <summary>A real button that shows an OS native menu.</summary>
public sealed class NativeMenuTriggerElement : Element
{
    internal NativeMenuTriggerElement(RenderContext ui, int index)
        : base(ui, index) { }

    public NativeMenuTriggerElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Accepted for parity; native menu display is synchronous.</summary>
    public NativeMenuTriggerElement OnEffectError(Action<string> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = Events.Register(value => handler(value.String ?? string.Empty));
        Arena.AddCallback(Index, "on_effect_error", token);
        return this;
    }

    public NativeMenuTriggerElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }
}
