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

    internal void AddOp(
        int node,
        ushort code,
        ushort flags,
        ulong a = 0,
        ulong b = 0,
        ulong c = 0
    )
        => _ops.Add(
            new NativeOp
            {
                Node = (uint)node,
                Code = code,
                Flags = flags,
                A = a,
                B = b,
                C = c,
            }
        );

    /// <summary>Records a no-argument style method by opcode.</summary>
    internal void AddNullaryStyle(int node, string method)
    {
        if (StyleOps.TryNullary(method, out var code))
        {
            AddOp(node, NativeProtocol.OpNullaryStyle, NativeProtocol.ArgNone, code);
        }
    }

    /// <summary>Records a style method taking a pixel/number argument.</summary>
    internal void AddParamStyle(int node, string method, double value)
    {
        if (StyleOps.TryParam(method, out var code))
        {
            AddOp(
                node,
                NativeProtocol.OpParamStyle,
                NativeProtocol.ArgNumber,
                code,
                BitConverter.SingleToUInt32Bits((float)value)
            );
        }
    }

    /// <summary>Records a style method taking a string (or color/length) argument.</summary>
    internal void AddParamStyleString(int node, string method, string value)
    {
        if (StyleOps.TryParam(method, out var code))
        {
            AddOp(
                node,
                NativeProtocol.OpParamStyle,
                NativeProtocol.ArgString,
                code,
                PackString(value)
            );
        }
    }

    /// <summary>Records a component behavior method by name.</summary>
    internal void AddMethod(int node, string method)
        => AddOp(node, NativeProtocol.OpMethod, NativeProtocol.ArgNone, MethodOps.Code(method));

    /// <summary>Records a component behavior method taking a number argument.</summary>
    internal void AddMethodNumber(int node, string method, double value)
        => AddOp(
            node,
            NativeProtocol.OpMethod,
            NativeProtocol.ArgNumber,
            MethodOps.Code(method),
            BitConverter.SingleToUInt32Bits((float)value)
        );

    /// <summary>Records a component behavior method taking a string argument.</summary>
    internal void AddMethodString(int node, string method, string value)
        => AddOp(
            node,
            NativeProtocol.OpMethod,
            NativeProtocol.ArgString,
            MethodOps.Code(method),
            PackString(value)
        );

    /// <summary>Records a component behavior method taking a closed-set literal argument.</summary>
    internal void AddMethodEnum(int node, string method, string value)
        => AddOp(
            node,
            NativeProtocol.OpMethod,
            NativeProtocol.ArgEnum,
            MethodOps.Code(method),
            PackEnum(value)
        );

    /// <summary>
    /// Records a component behavior method taking an element argument (P3). The
    /// referenced node is not a child edge; it materializes only if the method
    /// resolves it.
    /// </summary>
    internal void AddMethodElement(int node, string method, int childNode)
        => AddOp(
            node,
            NativeProtocol.OpMethod,
            NativeProtocol.ArgElement,
            MethodOps.Code(method),
            (ulong)childNode
        );

    /// <summary>Records a component method taking a string and a callback token.</summary>
    internal void AddMethodStringCallback(int node, string method, string value, ulong token)
        => AddOp(
            node,
            NativeProtocol.OpMethod,
            NativeProtocol.ArgStringCallback,
            MethodOps.Code(method),
            PackString(value),
            token
        );

    /// <summary>Records an event binding by name and callback token.</summary>
    internal void AddCallback(int node, string name, ulong token)
        => AddOp(node, NativeProtocol.OpCallback, NativeProtocol.ArgNone, PackName(name), token);

    /// <summary>Records a named slot pointing at a child node.</summary>
    internal void AddSlot(int node, string name, int child)
        => AddOp(
            node,
            NativeProtocol.OpSlot,
            NativeProtocol.ArgNumber,
            PackName(name),
            (ulong)child
        );

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
        GC.SuppressFinalize(this);
    }

    ~RenderArena() => Dispose();

    /// <summary>
    /// Appends a dynamic value (a label, color, length, or number) as UTF-8 and
    /// returns its packed `(offset, length)`. The bytes are encoded straight into
    /// the arena's buffer with no intermediate array.
    /// </summary>
    private (uint Offset, uint Length) AppendUtf8(string value)
    {
        var byteCount = Encoding.UTF8.GetByteCount(value);
        if (byteCount == 0)
        {
            return ((uint)_utf8.Count, 0);
        }
        var offset = _utf8.Count;
        CollectionsMarshal.SetCount(_utf8, offset + byteCount);
        Encoding.UTF8.GetBytes(value.AsSpan(), CollectionsMarshal.AsSpan(_utf8).Slice(offset, byteCount));
        return ((uint)offset, (uint)byteCount);
    }

    private ulong PackString(string value)
    {
        var (offset, length) = AppendUtf8(value);
        return ((ulong)offset << 32) | length;
    }

    /// <summary>
    /// Appends an invariant name (a method, callback, slot, or enum literal).
    /// Names come from a closed set of compile-time strings, so their UTF-8 is
    /// cached once and copied in; only the first frame pays for the encode.
    /// </summary>
    private ulong PackName(string value)
    {
        if (!NameUtf8.TryGetValue(value, out var bytes))
        {
            bytes = Encoding.UTF8.GetBytes(value);
            NameUtf8[value] = bytes;
        }
        return AppendBytes(bytes);
    }

    /// <summary>Appends an invariant enum literal; see <see cref="PackName"/>.</summary>
    private ulong PackEnum(string value) => PackName(value);

    private ulong AppendBytes(byte[] bytes)
    {
        if (bytes.Length == 0)
        {
            return ((ulong)_utf8.Count) << 32;
        }
        var offset = _utf8.Count;
        CollectionsMarshal.SetCount(_utf8, offset + bytes.Length);
        bytes.CopyTo(CollectionsMarshal.AsSpan(_utf8).Slice(offset, bytes.Length));
        return ((ulong)offset << 32) | (uint)bytes.Length;
    }

    private static readonly Dictionary<string, byte[]> NameUtf8 = new(StringComparer.Ordinal);

    private static void PublishBuffer<T>(List<T> source, ref byte* buffer, ref nuint capacity)
        where T : unmanaged
    {
        var required = (nuint)(source.Count * sizeof(T));
        EnsureCapacity(ref buffer, ref capacity, required);

        if (required > 0)
        {
            fixed (T* pointer = CollectionsMarshal.AsSpan(source))
            {
                Buffer.MemoryCopy(pointer, buffer, (long)capacity, (long)required);
            }
        }
    }

    /// <summary>
    /// Grows a native buffer geometrically, so a tree that grows or oscillates
    /// across the exact needed size does not reallocate every frame.
    /// </summary>
    private static void EnsureCapacity(ref byte* buffer, ref nuint capacity, nuint required)
    {
        if (required <= capacity)
        {
            return;
        }

        var next = capacity == 0 ? (nuint)256 : capacity;
        while (next < required)
        {
            next = next < (nuint)4_194_304 ? next * 2 : next + (nuint)4_194_304;
        }

        if (buffer != null)
        {
            NativeMemory.AlignedFree(buffer);
        }
        buffer = (byte*)NativeMemory.AlignedAlloc(next, Alignment);
        capacity = next;
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
