using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A plain container: shared style operations plus children.</summary>
public sealed class DivElement : Element
{
    internal DivElement(RenderContext ui, int index)
        : base(ui, index) { }

    public DivElement Add(params Element[] children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }

    public DivElement Children(IEnumerable<Element> children)
    {
        ArgumentNullException.ThrowIfNull(children);
        foreach (var child in children)
        {
            Arena.AddChild(Index, child.Index);
        }
        return this;
    }

    public DivElement FlexColumn()
    {
        Arena.AddOp(Index, NativeProtocol.OpFlexCol);
        return this;
    }

    public DivElement FlexRow()
    {
        Arena.AddOp(Index, NativeProtocol.OpFlexRow);
        return this;
    }

    public DivElement Gap(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpGap, pixels);
        return this;
    }

    public DivElement Padding(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpPadding, pixels);
        return this;
    }

    public DivElement PaddingX(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpPaddingX, pixels);
        return this;
    }

    public DivElement PaddingY(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpPaddingY, pixels);
        return this;
    }

    public DivElement Rounded(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpRounded, pixels);
        return this;
    }

    public DivElement Width(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpWidth, pixels);
        return this;
    }

    public DivElement Height(double pixels)
    {
        Arena.AddLengthOp(Index, NativeProtocol.OpHeight, pixels);
        return this;
    }

    public DivElement Full()
    {
        Arena.AddOp(Index, NativeProtocol.OpSizeFull);
        return this;
    }

    public DivElement ItemsCenter()
    {
        Arena.AddOp(Index, NativeProtocol.OpItemsCenter);
        return this;
    }

    public DivElement JustifyCenter()
    {
        Arena.AddOp(Index, NativeProtocol.OpJustifyCenter);
        return this;
    }

    public DivElement Background(uint rgba)
    {
        Arena.AddColorOp(Index, NativeProtocol.OpBackgroundRgba, rgba);
        return this;
    }

    public DivElement TextColor(uint rgba)
    {
        Arena.AddColorOp(Index, NativeProtocol.OpTextRgba, rgba);
        return this;
    }
}
