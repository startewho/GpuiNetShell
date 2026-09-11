using System.Runtime.InteropServices;

namespace GpuiNetShell.Interop;

/// <summary>Direct P/Invoke entry points. Only the API table crosses the boundary.</summary>
internal static unsafe class NativeMethods
{
    private const string Library = "gpui_net_shell";

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "gpui_net_shell_get_api")]
    internal static extern GpuiNetShellApi* GetApi(uint requestedVersion);

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "gpui_net_shell_abi_version")]
    internal static extern uint AbiVersion();

    [DllImport(Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "gpui_net_shell_schema_hash")]
    internal static extern ulong SchemaHash();
}
