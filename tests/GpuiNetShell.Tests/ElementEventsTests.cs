using GpuiNetShell.Events;

namespace GpuiNetShell.Tests;

public sealed class ElementEventsTests
{
    [Fact]
    public void PointerPayloadDecodesPositionButtonAndModifiers()
    {
        var pointer = PointerEvent.Decode("12.5\t-3\t1\t2\t5");
        Assert.Equal(12.5, pointer.X);
        Assert.Equal(-3, pointer.Y);
        Assert.Equal(1, pointer.Button);
        Assert.Equal(2, pointer.ClickCount);
        Assert.Equal(InputModifiers.Shift | InputModifiers.Alt, pointer.Modifiers);
    }

    [Fact]
    public void ScrollAndKeyPayloadsDecode()
    {
        var scroll = ScrollEvent.Decode("1\t2\t-10\t20\t0");
        Assert.Equal(1, scroll.X);
        Assert.Equal(2, scroll.Y);
        Assert.Equal(-10, scroll.DeltaX);
        Assert.Equal(20, scroll.DeltaY);

        var key = KeyEvent.Decode("enter\t2\t1");
        Assert.Equal("enter", key.Key);
        Assert.Equal(InputModifiers.Control, key.Modifiers);
        Assert.True(key.IsHeld);
    }

    [Fact]
    public void AnEmptyPayloadDecodesToDefaults()
    {
        var pointer = PointerEvent.Decode(null);
        Assert.Equal(0, pointer.X);
        Assert.Equal(0, pointer.Button);
        Assert.Equal(InputModifiers.None, pointer.Modifiers);
    }
}
