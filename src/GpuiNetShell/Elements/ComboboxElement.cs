using System.Globalization;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A trigger that opens the native popup with a list of options. Selection is
/// delivered to <see cref="OnChange"/> through one callback token per option.
/// </summary>
public sealed class ComboboxElement : Element
{
    private string[] _options = [];

    internal ComboboxElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the option labels. Call before <see cref="OnChange"/>.</summary>
    public ComboboxElement Options(params string[] options)
    {
        _options = options ?? [];
        Arena.AddMethodString(Index, "options", string.Join('\n', _options));
        return this;
    }

    /// <summary>Sets the selected option index.</summary>
    public ComboboxElement Selected(int index)
    {
        Arena.AddMethodNumber(Index, "selected", index);
        return this;
    }

    /// <summary>Receives the index chosen in the popup.</summary>
    public ComboboxElement OnChange(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var tokens = _options.Select((_, index) =>
            Events.Register(() => handler(index))
        );
        Arena.AddMethodString(
            Index,
            "tokens",
            string.Join(',', tokens.Select(token => token.ToString(CultureInfo.InvariantCulture)))
        );
        return this;
    }
}
