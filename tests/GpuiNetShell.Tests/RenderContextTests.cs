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
        Assert.Equal(NativeProtocol.OpFlexCol, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.OpGap, descriptor.Ops[1].Code);
    }
}
