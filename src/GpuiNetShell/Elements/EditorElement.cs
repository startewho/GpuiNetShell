using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A retained native source editor.</summary>
public sealed class EditorElement : Element
{
    internal EditorElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the initial source text (first render only).</summary>
    public EditorElement Value(string value)
    {
        Arena.AddMethodString(Index, "value", value);
        return this;
    }

    /// <summary>Sets the syntax language: rust, json, or plaintext (first render only).</summary>
    public EditorElement Language(string language)
    {
        Arena.AddMethodEnum(Index, "language", language);
        return this;
    }

    public EditorElement Appearance(bool appearance = true)
    {
        Arena.AddMethodNumber(Index, "appearance", appearance ? 1 : 0);
        return this;
    }

    public EditorElement Bordered(bool bordered = true)
    {
        Arena.AddMethodNumber(Index, "bordered", bordered ? 1 : 0);
        return this;
    }

    public EditorElement Readonly(bool readOnly = true)
    {
        Arena.AddMethodNumber(Index, "readonly", readOnly ? 1 : 0);
        return this;
    }

    public EditorElement Disabled(bool disabled = true)
    {
        Arena.AddMethodNumber(Index, "disabled", disabled ? 1 : 0);
        return this;
    }

    /// <summary>Sets the editor accessibility label.</summary>
    public EditorElement AriaLabel(string label)
    {
        Arena.AddMethodString(Index, "aria_label", label);
        return this;
    }
}
