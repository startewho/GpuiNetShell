using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A platform-formatted keyboard shortcut keycap.</summary>
public sealed class KbdElement : Element
{
    internal KbdElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Controls whether the keystroke uses keycap presentation.</summary>
    public KbdElement Appearance(bool appearance = true)
    {
        Arena.AddMethodNumber(Index, "appearance", appearance ? 1 : 0);
        return this;
    }

    /// <summary>Uses the outlined keycap presentation.</summary>
    public KbdElement Outline()
    {
        Arena.AddMethod(Index, "outline");
        return this;
    }
}
