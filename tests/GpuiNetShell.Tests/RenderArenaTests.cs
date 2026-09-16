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
        arena.AddMethodString(button, "label", "Save");
        arena.AddMethodNumber(button, "disabled", 1);
        arena.AddCallback(button, "on_click", 7);
        arena.AddChild(root, button);

        var descriptor = arena.Publish();

        Assert.Equal(2u, descriptor.NodesLen);
        Assert.Equal(3u, descriptor.OpsLen);
        Assert.Equal(1u, descriptor.ChildrenLen);
        Assert.Equal(4u, descriptor.Nodes[button].DataLen);
        Assert.Equal(NativeProtocol.OpMethod, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.ArgString, descriptor.Ops[0].Flags);
        Assert.Equal(NativeProtocol.OpCallback, descriptor.Ops[2].Code);
        Assert.Equal(7UL, descriptor.Ops[2].B);
        Assert.Equal((uint)root, descriptor.Children[0].Parent);
        Assert.Equal((uint)button, descriptor.Children[0].Child);
    }

    [Fact]
    public void MethodNamesTravelAsCodesAndValuesPackOffsetAndLength()
    {
        using var arena = new RenderArena();
        var node = arena.AddNode(NativeProtocol.ComponentButton);
        arena.AddMethodString(node, "label", "Save");

        var descriptor = arena.Publish();

        Assert.Equal(MethodOps.Code("label"), descriptor.Ops[0].A);
        var packed = descriptor.Ops[0].B;
        Assert.Equal(0ul, packed >> 32);
        Assert.Equal((ulong)"Save".Length, packed & 0xFFFF_FFFF);
        Assert.Equal((byte)'S', descriptor.Utf8[0]);
    }

    [Fact]
    public void TheMethodCodeIsStable()
    {
        // Pinned against `schema::method_code` in the native crate; the two
        // must produce identical codes or every component method would be
        // dropped on the native side.
        Assert.Equal(0x39F7_FCEC_8FCB_623DUL, MethodOps.Code("label"));
        Assert.Equal(0x0FA3_391D_B68E_4425UL, MethodOps.Code("disabled"));
    }

    [Fact]
    public void StyleCallsUseTheGenericStyleEnvelope()
    {
        using var arena = new RenderArena();
        var node = arena.AddNode(NativeProtocol.ComponentDiv);
        arena.AddNullaryStyle(node, "items_center");
        arena.AddParamStyle(node, "p", 12);
        arena.AddParamStyleString(node, "bg", "#112233");

        var descriptor = arena.Publish();

        Assert.Equal(NativeProtocol.OpNullaryStyle, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.ArgNone, descriptor.Ops[0].Flags);
        Assert.Equal(NullaryCode("items_center"), descriptor.Ops[0].A);
        Assert.Equal(NativeProtocol.OpParamStyle, descriptor.Ops[1].Code);
        Assert.Equal(NativeProtocol.ArgNumber, descriptor.Ops[1].Flags);
        Assert.Equal(ParamCode("p"), descriptor.Ops[1].A);
        Assert.Equal(12.0f, BitConverter.UInt32BitsToSingle((uint)descriptor.Ops[1].B));
        Assert.Equal(NativeProtocol.OpParamStyle, descriptor.Ops[2].Code);
        Assert.Equal(NativeProtocol.ArgString, descriptor.Ops[2].Flags);
        Assert.Equal(ParamCode("bg"), descriptor.Ops[2].A);
    }

    [Fact]
    public void AnUnknownStyleNameIsDropped()
    {
        using var arena = new RenderArena();
        var node = arena.AddNode(NativeProtocol.ComponentDiv);
        arena.AddNullaryStyle(node, "not_a_style_at_all");
        arena.AddParamStyle(node, "not_a_style_either", 3);

        var descriptor = arena.Publish();

        Assert.Equal(0u, descriptor.OpsLen);
    }

    private static ulong NullaryCode(string name) =>
        (ulong)Array.IndexOf(StyleOps.Nullary, name);

    private static ulong ParamCode(string name) =>
        (ulong)Array.IndexOf(StyleOps.Param, name);

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
