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
        Assert.Equal(NativeProtocol.OpMethod, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.ArgString, descriptor.Ops[0].Flags);
        Assert.Equal(NativeProtocol.OpMethod, descriptor.Ops[1].Code);
        Assert.Equal(NativeProtocol.ArgNone, descriptor.Ops[1].Flags);
        Assert.Equal(NativeProtocol.OpCallback, descriptor.Ops[2].Code);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        Assert.Equal("label", DecodePacked(descriptor.Ops[0].A, utf8));
        Assert.Equal("Save", DecodePacked(descriptor.Ops[0].B, utf8));
        Assert.Equal("primary", DecodePacked(descriptor.Ops[1].A, utf8));
        Assert.Equal("on_click", DecodePacked(descriptor.Ops[2].A, utf8));
        Assert.Equal(0, calls);
    }

    [Fact]
    public void ContainerRecordsChildEdgesAndStyleCalls()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry());

        ui.VStack(ui.Text("one"), ui.Text("two")).Gap(8);

        var descriptor = arena.Publish();
        Assert.Equal(3u, descriptor.NodesLen);
        Assert.Equal(2u, descriptor.ChildrenLen);
        Assert.Equal(NativeProtocol.ComponentText, descriptor.Nodes[1].Component);
        Assert.Equal(2u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpNullaryStyle, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.OpParamStyle, descriptor.Ops[1].Code);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        Assert.Equal("flex_col", DecodePacked(descriptor.Ops[0].A, utf8));
        Assert.Equal("gap", DecodePacked(descriptor.Ops[1].A, utf8));
        Assert.Equal(8.0f, BitConverter.UInt32BitsToSingle((uint)descriptor.Ops[1].B));
    }

    [Fact]
    public void StyleCallsRecordTheGpuiMethodNameAndArgument()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry());

        ui.Div().P(12).Bg("#112233").Full();

        var descriptor = arena.Publish();
        Assert.Equal(3u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpParamStyle, descriptor.Ops[0].Code);
        Assert.Equal(12.0f, BitConverter.UInt32BitsToSingle((uint)descriptor.Ops[0].B));
        Assert.Equal(NativeProtocol.OpParamStyle, descriptor.Ops[1].Code);
        Assert.Equal(NativeProtocol.OpNullaryStyle, descriptor.Ops[2].Code);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        Assert.Equal("p", DecodePacked(descriptor.Ops[0].A, utf8));
        Assert.Equal("bg", DecodePacked(descriptor.Ops[1].A, utf8));
        Assert.Equal("#112233", DecodePacked(descriptor.Ops[1].B, utf8));
        Assert.Equal("size_full", DecodePacked(descriptor.Ops[2].A, utf8));
    }

    private static string DecodePacked(ulong packed, ReadOnlySpan<byte> utf8)
    {
        var offset = (int)(packed >> 32);
        var length = (int)(packed & 0xFFFF_FFFF);
        return System.Text.Encoding.UTF8.GetString(utf8.Slice(offset, length));
    }
}
