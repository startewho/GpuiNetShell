using System.Runtime.InteropServices;
using System.Text;
using GpuiNetShell.Interop;

namespace GpuiNetShell.Rendering;

/// <summary>
/// Builds a flat element description and copies it into unmanaged buffers for
/// the native host to decode.
/// </summary>
/// <remarks>
/// The buffers are owned here and reused across renders. Native decoding is
/// synchronous inside the render callback, so a published descriptor stays
/// valid until the next <see cref="Reset"/>.
/// </remarks>
internal sealed unsafe class RenderArena : IDisposable
{
    private const int Alignment = 16;

    private readonly List<NativeNode> _nodes = [];
    private readonly List<NativeOp> _ops = [];
    private readonly List<NativeChild> _children = [];
    private readonly List<byte> _utf8 = [];

    private byte* _nodesBuffer;
    private nuint _nodesCapacity;
    private byte* _opsBuffer;
    private nuint _opsCapacity;
    private byte* _childrenBuffer;
    private nuint _childrenCapacity;
    private byte* _utf8Buffer;
    private nuint _utf8Capacity;

    internal int AddNode(uint component)
    {
        _nodes.Add(new NativeNode { Component = component });
        return _nodes.Count - 1;
    }

    internal void SetNodeData(int index, string value)
    {
        var (offset, length) = AppendUtf8(value);
        var node = _nodes[index];
        node.DataOffset = offset;
        node.DataLen = length;
        _nodes[index] = node;
    }

    internal void AddOp(int node, ushort code, ushort flags, ulong a = 0, ulong b = 0)
        => _ops.Add(
            new NativeOp
            {
                Node = (uint)node,
                Code = code,
                Flags = flags,
                A = a,
                B = b,
            }
        );

    /// <summary>Records a no-argument style method by name.</summary>
    internal void AddNullaryStyle(int node, string method)
        => AddOp(node, NativeProtocol.OpNullaryStyle, NativeProtocol.ArgNone, PackString(method));

    /// <summary>Records a style method taking a pixel/number argument.</summary>
    internal void AddParamStyle(int node, string method, double value)
        => AddOp(
            node,
            NativeProtocol.OpParamStyle,
            NativeProtocol.ArgNumber,
            PackString(method),
            BitConverter.SingleToUInt32Bits((float)value)
        );

    /// <summary>Records a style method taking a string (or color) argument.</summary>
    internal void AddParamStyleString(int node, string method, string value)
        => AddOp(
            node,
            NativeProtocol.OpParamStyle,
            NativeProtocol.ArgString,
            PackString(method),
            PackString(value)
        );

    /// <summary>Records a component behavior method by name.</summary>
    internal void AddMethod(int node, string method)
        => AddOp(node, NativeProtocol.OpMethod, NativeProtocol.ArgNone, PackString(method));

    /// <summary>Records a component behavior method taking a number argument.</summary>
    internal void AddMethodNumber(int node, string method, double value)
        => AddOp(
            node,
            NativeProtocol.OpMethod,
            NativeProtocol.ArgNumber,
            PackString(method),
            BitConverter.SingleToUInt32Bits((float)value)
        );

    /// <summary>Records a component behavior method taking a string argument.</summary>
    internal void AddMethodString(int node, string method, string value)
        => AddOp(
            node,
            NativeProtocol.OpMethod,
            NativeProtocol.ArgString,
            PackString(method),
            PackString(value)
        );

    /// <summary>Records an event binding by name and callback token.</summary>
    internal void AddCallback(int node, string name, ulong token)
        => AddOp(node, NativeProtocol.OpCallback, NativeProtocol.ArgNone, PackString(name), token);

    internal void AddChild(int parent, int child)
        => _children.Add(new NativeChild { Parent = (uint)parent, Child = (uint)child });

    internal NativeArena Publish()
    {
        PublishBuffer(_nodes, ref _nodesBuffer, ref _nodesCapacity);
        PublishBuffer(_ops, ref _opsBuffer, ref _opsCapacity);
        PublishBuffer(_children, ref _childrenBuffer, ref _childrenCapacity);
        PublishBuffer(_utf8, ref _utf8Buffer, ref _utf8Capacity);

        return new NativeArena
        {
            Nodes = (NativeNode*)_nodesBuffer,
            NodesLen = (uint)_nodes.Count,
            Ops = (NativeOp*)_opsBuffer,
            OpsLen = (uint)_ops.Count,
            Children = (NativeChild*)_childrenBuffer,
            ChildrenLen = (uint)_children.Count,
            Utf8 = _utf8Buffer,
            Utf8Len = (uint)_utf8.Count,
        };
    }

    internal void Reset()
    {
        _nodes.Clear();
        _ops.Clear();
        _children.Clear();
        _utf8.Clear();
    }

    public void Dispose()
    {
        Free(ref _nodesBuffer, ref _nodesCapacity);
        Free(ref _opsBuffer, ref _opsCapacity);
        Free(ref _childrenBuffer, ref _childrenCapacity);
        Free(ref _utf8Buffer, ref _utf8Capacity);
    }

    private (uint Offset, uint Length) AppendUtf8(string value)
    {
        var bytes = Encoding.UTF8.GetBytes(value);
        var offset = (uint)_utf8.Count;
        _utf8.AddRange(bytes);
        return (offset, (uint)bytes.Length);
    }

    private ulong PackString(string value)
    {
        var (offset, length) = AppendUtf8(value);
        return ((ulong)offset << 32) | length;
    }

    private static void PublishBuffer<T>(List<T> source, ref byte* buffer, ref nuint capacity)
        where T : unmanaged
    {
        var required = (nuint)(source.Count * sizeof(T));
        if (required > capacity)
        {
            var next = required < 256 ? (nuint)256 : required;
            if (buffer != null)
            {
                NativeMemory.AlignedFree(buffer);
            }
            buffer = (byte*)NativeMemory.AlignedAlloc(next, Alignment);
            capacity = next;
        }

        if (required > 0)
        {
            var values = source.ToArray();
            fixed (T* pointer = values)
            {
                Buffer.MemoryCopy(pointer, buffer, (long)capacity, (long)required);
            }
        }
    }

    private static void Free(ref byte* buffer, ref nuint capacity)
    {
        if (buffer != null)
        {
            NativeMemory.AlignedFree(buffer);
            buffer = null;
            capacity = 0;
        }
    }
}
