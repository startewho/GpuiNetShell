using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A run of styled text. Content is the constructor argument.</summary>
public sealed class TextElement : Element
{
    internal TextElement(RenderContext ui, int index)
        : base(ui, index) { }

    public TextElement FontSize(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpFontSize, pixels);
        return this;
    }

    public TextElement TextColor(uint rgba)
    {
        Arena.AddColorOp(Index, NativeProtocol.OpTextRgba, rgba);
        return this;
    }
}
