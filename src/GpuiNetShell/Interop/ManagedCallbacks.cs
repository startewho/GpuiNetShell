using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Interop;

/// <summary>
/// The managed callback table. Every callback is a cdecl unmanaged function
/// pointer and never lets a managed exception cross the boundary.
/// </summary>
internal static unsafe class ManagedCallbacks
{
    internal static NativeCallbacks Create() => new()
    {
        StructSize = (uint)sizeof(NativeCallbacks),
        Reserved = 0,
        ApplicationStarted = &ApplicationStartedCallback,
        WindowClosed = &WindowClosedCallback,
        Render = &RenderCallback,
        RenderCompleted = &RenderCompletedCallback,
        Click = &ClickCallback,
        RetireCallbacks = &RetireCallbacksCallback,
        Invoke = &InvokeCallback,
        ResolveRows = &ResolveRowsCallback,
        RenderElement = &RenderElementCallback,
        InputEvent = &InputEventCallback,
    };

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int ApplicationStartedCallback(ulong sessionId)
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnStarted()
                : NativeProtocol.StatusOk;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int WindowClosedCallback(ulong sessionId, int status)
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnWindowClosed(status)
                : NativeProtocol.StatusOk;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int RenderCallback(
        ulong sessionId,
        ulong generation,
        NativeArena* arena,
        uint* root
    )
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.RenderInto(generation, arena, root)
                : -1;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int RenderCompletedCallback(ulong sessionId, ulong generation, int status)
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnRenderCompleted(generation, status)
                : NativeProtocol.StatusOk;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int ClickCallback(ulong sessionId, ulong token)
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnClick(token)
                : -1;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int RetireCallbacksCallback(ulong sessionId, ulong generation)
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnRetireCallbacks(generation)
                : NativeProtocol.StatusOk;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int InvokeCallback(
        ulong sessionId,
        ulong token,
        uint kind,
        double number,
        byte* data,
        uint dataLength
    )
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnInvoke(token, kind, number, data, dataLength)
                : -1;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int ResolveRowsCallback(
        ulong sessionId,
        ulong token,
        byte* buffer,
        uint capacity,
        uint* outLen
    )
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnResolveRows(token, buffer, capacity, outLen)
                : -1;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int RenderElementCallback(
        ulong sessionId,
        ulong token,
        byte* arguments,
        uint argumentsLength,
        NativeArena* outArena,
        uint* outRoot
    )
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnRenderElement(token, arguments, argumentsLength, outArena, outRoot)
                : -1;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int InputEventCallback(
        ulong sessionId,
        uint kind,
        uint flags,
        float a,
        float b,
        float c,
        byte* text,
        uint textLength
    )
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnInputEvent(kind, flags, a, b, c, text, textLength)
                : NativeProtocol.StatusOk;
        }
        catch
        {
            return -1;
        }
    }
}
