using System.Runtime.InteropServices;

namespace GpuiNetShell.Interop;

/// <summary>Mirrors <c>GpuiNetNode</c>. 16 bytes.</summary>
[StructLayout(LayoutKind.Sequential)]
internal struct NativeNode
{
    public uint Component;
    public uint Flags;
    public uint DataOffset;
    public uint DataLen;
}

/// <summary>Mirrors <c>GpuiNetOp</c>. 32 bytes.</summary>
[StructLayout(LayoutKind.Sequential)]
internal struct NativeOp
{
    public uint Node;
    public ushort Code;
    public ushort Flags;
    public ulong A;
    public ulong B;
    public ulong C;
}

/// <summary>Mirrors <c>GpuiNetChild</c>. 8 bytes.</summary>
[StructLayout(LayoutKind.Sequential)]
internal struct NativeChild
{
    public uint Parent;
    public uint Child;
}

/// <summary>Mirrors <c>GpuiNetArena</c>. 64 bytes.</summary>
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeArena
{
    public NativeNode* Nodes;
    public uint NodesLen;
    public uint Pad0;
    public NativeOp* Ops;
    public uint OpsLen;
    public uint Pad1;
    public NativeChild* Children;
    public uint ChildrenLen;
    public uint Pad2;
    public byte* Utf8;
    public uint Utf8Len;
    public uint Pad3;
}

/// <summary>Mirrors <c>GpuiNetCallbacks</c>.</summary>
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeCallbacks
{
    public uint StructSize;
    public uint Reserved;
    public delegate* unmanaged[Cdecl]<ulong, int> ApplicationStarted;
    public delegate* unmanaged[Cdecl]<ulong, int, int> WindowClosed;
    public delegate* unmanaged[Cdecl]<ulong, ulong, NativeArena*, uint*, int> Render;
    public delegate* unmanaged[Cdecl]<ulong, ulong, int, int> RenderCompleted;
    public delegate* unmanaged[Cdecl]<ulong, ulong, int> Click;
    public delegate* unmanaged[Cdecl]<ulong, ulong, int> RetireCallbacks;
    public delegate* unmanaged[Cdecl]<ulong, ulong, uint, double, byte*, uint, int> Invoke;
    public delegate* unmanaged[Cdecl]<ulong, ulong, byte*, uint, uint*, int> ResolveRows;
    public delegate* unmanaged[Cdecl]<ulong, ulong, byte*, uint, NativeArena*, uint*, int> RenderElement;
    public delegate* unmanaged[Cdecl]<ulong, uint, uint, float, float, float, byte*, uint, int> InputEvent;
}

/// <summary>Mirrors <c>GpuiNetShellApi</c>.</summary>
[StructLayout(LayoutKind.Sequential)]
internal unsafe struct GpuiNetShellApi
{
    public uint StructSize;
    public uint AbiVersion;
    public ulong SchemaHash;
    public delegate* unmanaged[Cdecl]<ulong, NativeCallbacks*, int> RunApplication;
    public delegate* unmanaged[Cdecl]<ulong, int> Invalidate;
    public ulong Reserved;
}
