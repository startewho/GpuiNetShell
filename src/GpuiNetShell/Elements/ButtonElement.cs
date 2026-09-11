using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A stateless command button. Identity is the constructor id; label, variant,
/// size, loading, compact, disabled, selected and activation are operations.
/// </summary>
public sealed class ButtonElement : Element
{
    internal ButtonElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the visible label.</summary>
    public ButtonElement Label(string label)
    {
        Arena.AddStringOp(Index, NativeProtocol.OpLabel, label);
        return this;
    }

    /// <summary>Sets concise hover help.</summary>
    public ButtonElement Tooltip(string tooltip)
    {
        Arena.AddStringOp(Index, NativeProtocol.OpTooltip, tooltip);
        return this;
    }

    public ButtonElement Variant(ButtonVariant variant)
    {
        Arena.AddOp(Index, NativeProtocol.OpButtonVariant, (ulong)variant);
        return this;
    }

    public ButtonElement Size(ButtonSize size)
    {
        Arena.AddOp(Index, NativeProtocol.OpButtonSize, (ulong)size);
        return this;
    }

    public ButtonElement Primary() => Variant(ButtonVariant.Primary);

    public ButtonElement Secondary() => Variant(ButtonVariant.Secondary);

    public ButtonElement Danger() => Variant(ButtonVariant.Danger);

    public ButtonElement Success() => Variant(ButtonVariant.Success);

    public ButtonElement Warning() => Variant(ButtonVariant.Warning);

    public ButtonElement Ghost() => Variant(ButtonVariant.Ghost);

    public ButtonElement Link() => Variant(ButtonVariant.Link);

    public ButtonElement Loading(bool loading = true)
    {
        Arena.AddBoolOp(Index, NativeProtocol.OpLoading, loading);
        return this;
    }

    public ButtonElement Compact()
    {
        Arena.AddOp(Index, NativeProtocol.OpCompact);
        return this;
    }

    public ButtonElement Disabled(bool disabled = true)
    {
        Arena.AddBoolOp(Index, NativeProtocol.OpDisabled, disabled);
        return this;
    }

    public ButtonElement Selected(bool selected = true)
    {
        Arena.AddBoolOp(Index, NativeProtocol.OpSelected, selected);
        return this;
    }

    /// <summary>
    /// Binds activation. The handler runs on the native application thread; the
    /// host requests a re-render after it returns.
    /// </summary>
    public ButtonElement OnClick(Action handler)
    {
        var token = Events.Register(handler);
        Arena.AddOp(Index, NativeProtocol.OpOnClick, token);
        return this;
    }

    public ButtonElement Padding(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpPadding, pixels);
        return this;
    }

    public ButtonElement Width(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpWidth, pixels);
        return this;
    }
}
