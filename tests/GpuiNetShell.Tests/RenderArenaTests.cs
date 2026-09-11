using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Tests;

public sealed unsafe class RenderArenaTests
{
    [Fact]
    public void EncodesNodesOpsChildrenAndUtf8()
    {
        using var arena = new RenderArena();
        var root = arena.AddNode(NativeProtocol.ComponentDiv);
        var button = arena.AddNode(NativeProtocol.ComponentButton);
        arena.SetNodeData(button, "save");
        arena.AddStringOp(button, NativeProtocol.OpLabel, "Save");
        arena.AddBoolOp(button, NativeProtocol.OpDisabled, true);
        arena.AddChild(root, button);

        var descriptor = arena.Publish();

        Assert.Equal(2u, descriptor.NodesLen);
        Assert.Equal(2u, descriptor.OpsLen);
        Assert.Equal(1u, descriptor.ChildrenLen);
        Assert.Equal(4u, descriptor.Nodes[button].DataLen);
        Assert.Equal(NativeProtocol.OpLabel, descriptor.Ops[0].Code);
        Assert.Equal(1UL, descriptor.Ops[1].A);
        Assert.Equal((uint)root, descriptor.Children[0].Parent);
        Assert.Equal((uint)button, descriptor.Children[0].Child);
    }

    [Fact]
    public void StringOperationsPackOffsetAndLength()
    {
        using var arena = new RenderArena();
        var node = arena.AddNode(NativeProtocol.ComponentButton);
        arena.AddStringOp(node, NativeProtocol.OpLabel, "Save");

        var descriptor = arena.Publish();
        var packed = descriptor.Ops[0].A;

        Assert.Equal(0ul, packed >> 32);
        Assert.Equal((ulong)"Save".Length, packed & 0xFFFF_FFFF);
        Assert.Equal((byte)'S', descriptor.Utf8[0]);
    }

    [Fact]
    public void ResetClearsTheArena()
    {
        using var arena = new RenderArena();
        arena.AddNode(NativeProtocol.ComponentDiv);
        arena.Reset();

        var descriptor = arena.Publish();

        Assert.Equal(0u, descriptor.NodesLen);
        Assert.Equal(0u, descriptor.OpsLen);
    }
}
