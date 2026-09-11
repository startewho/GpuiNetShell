using GpuiNetShell.Interop;

namespace GpuiNetShell;

/// <summary>Direct access to the native host's ABI identity.</summary>
public static unsafe class GpuiNativeHost
{
    /// <summary>The ABI version the native host implements.</summary>
    public static uint AbiVersion => NativeMethods.AbiVersion();

    /// <summary>The schema hash the native host was built with.</summary>
    public static ulong SchemaHash => NativeMethods.SchemaHash();

    /// <summary>
    /// True when the native host loads and negotiates the same ABI version and
    /// schema hash as this managed assembly.
    /// </summary>
    public static bool Verify()
    {
        var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        return api != null && api->SchemaHash == NativeProtocol.SchemaHash;
    }
}
