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
        NativeArena* arena,
        uint* root,
        ulong* revision
    )
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.RenderInto(arena, root, revision)
                : -1;
        }
        catch
        {
            return -1;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static int RenderCompletedCallback(ulong sessionId, ulong revision, int status)
    {
        try
        {
            return GpuiApplication.Find(sessionId) is { } application
                ? application.OnRenderCompleted(revision, status)
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
}
