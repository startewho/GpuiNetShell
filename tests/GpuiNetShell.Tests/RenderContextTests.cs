using GpuiNetShell.Elements;
using GpuiNetShell.Events;
using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Tests;

public sealed unsafe class RenderContextTests
{
    [Fact]
    public void ButtonBuildsIdentityLabelVariantAndClick()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry());
        var calls = 0;

        ui.Button("save").Label("Save").Primary().OnClick(() => calls++);

        var descriptor = arena.Publish();
        Assert.Equal(1u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentButton, descriptor.Nodes[0].Component);
        Assert.Equal(3u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpLabel, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.OpButtonVariant, descriptor.Ops[1].Code);
        Assert.Equal((ulong)ButtonVariant.Primary, descriptor.Ops[1].A);
        Assert.Equal(NativeProtocol.OpOnClick, descriptor.Ops[2].Code);
        Assert.Equal(0, calls);
    }

    [Fact]
    public void ContainerRecordsChildEdges()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry());

        ui.VStack(ui.Text("one"), ui.Text("two")).Gap(8);

        var descriptor = arena.Publish();
        Assert.Equal(3u, descriptor.NodesLen);
        Assert.Equal(2u, descriptor.ChildrenLen);
        Assert.Equal(NativeProtocol.ComponentText, descriptor.Nodes[1].Component);
        Assert.Equal(2u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpStyleNullary, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.OpStyleLength, descriptor.Ops[1].Code);
    }

    [Fact]
    public void StyleCallsRecordTheGpuiMethodNameAndArgument()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry());

        ui.Div().P(12).Bg("#112233").Full();

        var descriptor = arena.Publish();
        Assert.Equal(3u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpStyleLength, descriptor.Ops[0].Code);
        Assert.Equal(12.0f, BitConverter.UInt32BitsToSingle((uint)descriptor.Ops[0].B));
        Assert.Equal(NativeProtocol.OpStyleColor, descriptor.Ops[1].Code);
        Assert.Equal(NativeProtocol.OpStyleNullary, descriptor.Ops[2].Code);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        var paddingName = DecodePacked(descriptor.Ops[0].A, utf8);
        var colorName = DecodePacked(descriptor.Ops[1].A, utf8);
        var colorValue = DecodePacked(descriptor.Ops[1].B, utf8);
        var fullName = DecodePacked(descriptor.Ops[2].A, utf8);
        Assert.Equal("p", paddingName);
        Assert.Equal("bg", colorName);
        Assert.Equal("#112233", colorValue);
        Assert.Equal("size_full", fullName);
    }

    private static string DecodePacked(ulong packed, ReadOnlySpan<byte> utf8)
    {
        var offset = (int)(packed >> 32);
        var length = (int)(packed & 0xFFFF_FFFF);
        return System.Text.Encoding.UTF8.GetString(utf8.Slice(offset, length));
    }
}
