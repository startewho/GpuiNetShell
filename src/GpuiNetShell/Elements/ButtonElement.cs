using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A stateless command button. Identity is the constructor id; label, variant,
/// size, loading, compact, disabled, selected and activation are recorded as
/// behavior methods and a callback, matching the shell's `Behavior`.
/// </summary>
public sealed class ButtonElement : Element
{
    internal ButtonElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the visible label.</summary>
    public ButtonElement Label(string label) => Method("label", label);

    /// <summary>Sets concise hover help.</summary>
    public ButtonElement Tooltip(string tooltip) => Method("tooltip", tooltip);

    /// <summary>Selects a visual variant by its method name.</summary>
    public ButtonElement Variant(ButtonVariant variant)
    {
        Arena.AddMethod(Index, VariantName(variant));
        return this;
    }

    public ButtonElement Size(ButtonSize size)
    {
        Arena.AddMethodNumber(Index, "size", (double)(int)size);
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
        Arena.AddMethodNumber(Index, "loading", loading ? 1 : 0);
        return this;
    }

    public ButtonElement Compact()
    {
        Arena.AddMethod(Index, "compact");
        return this;
    }

    public ButtonElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    public ButtonElement Selected(bool selected = true)
    {
        Arena.AddMethodNumber(Index, "selected", selected ? 1 : 0);
        return this;
    }

    /// <summary>
    /// Binds activation. The handler runs on the native application thread; the
    /// host requests a re-render after it returns.
    /// </summary>
    public ButtonElement OnClick(Action handler)
    {
        var token = Events.Register(handler);
        Arena.AddCallback(Index, "on_click", token);
        return this;
    }

    private ButtonElement Method(string name, string value)
    {
        Arena.AddMethodString(Index, name, value);
        return this;
    }

    private static string VariantName(ButtonVariant variant) =>
        variant switch
        {
            ButtonVariant.Default => "default",
            ButtonVariant.Primary => "primary",
            ButtonVariant.Secondary => "secondary",
            ButtonVariant.Danger => "danger",
            ButtonVariant.Success => "success",
            ButtonVariant.Warning => "warning",
            ButtonVariant.Ghost => "ghost",
            ButtonVariant.Link => "link",
            _ => "default",
        };
}
