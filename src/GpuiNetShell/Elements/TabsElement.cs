using System.Globalization;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A controlled tab bar. Options and their callback tokens are carried as data,
/// like <see cref="ComboboxElement"/>; the native bar reports the clicked index
/// through that option's token.
/// </summary>
public sealed class TabsElement : Element
{
    private string[] _options = [];

    internal TabsElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the tab labels. Call before <see cref="OnChange"/>.</summary>
    public TabsElement Options(params string[] options)
    {
        _options = options ?? [];
        Arena.AddMethodString(Index, "options", string.Join('\n', _options));
        return this;
    }

    /// <summary>Sets the selected tab index.</summary>
    public TabsElement Selected(int index)
    {
        Arena.AddMethodNumber(Index, "selected", index);
        return this;
    }

    /// <summary>Receives the index of the clicked tab.</summary>
    public TabsElement OnChange(Action<int> handler)
    {
        ArgumentNullException.ThrowIfNull(handler);
        var tokens = _options.Select((_, index) => Events.Register(() => handler(index)));
        Arena.AddMethodString(
            Index,
            "tokens",
            string.Join(',', tokens.Select(token => token.ToString(CultureInfo.InvariantCulture)))
        );
        return this;
    }
}
