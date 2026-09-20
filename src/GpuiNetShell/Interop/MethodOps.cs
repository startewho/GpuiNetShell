using System.Collections.Concurrent;

namespace GpuiNetShell.Interop;

/// <summary>
/// Computes the stable numeric code a component method name travels as.
/// Mirrors <c>schema::method_code</c> in
/// <c>crates/gpui-net-shell/src/schema.rs</c>: FNV-1a over the name's bytes.
/// Component method names are ASCII identifiers known to both sides, so a name
/// never crosses the ABI; only this code does.
/// </summary>
/// <remarks>
/// Codes are cached per name, so the steady-state cost is one dictionary lookup
/// and no allocation. The cache is shared across sessions that may build frames
/// on different threads, so it must be concurrent.
/// </remarks>
internal static class MethodOps
{
    private const ulong OffsetBasis = 0xcbf2_9ce4_8422_2325;
    private const ulong Prime = 0x0000_0100_0000_01b3;

    private static readonly ConcurrentDictionary<string, ulong> Codes = new(StringComparer.Ordinal);

    /// <summary>The code for <paramref name="name"/>. Names are ASCII.</summary>
    internal static ulong Code(string name) =>
        Codes.GetOrAdd(
            name,
            static value =>
            {
                var hash = OffsetBasis;
                foreach (var character in value)
                {
                    hash ^= (byte)character;
                    hash *= Prime;
                }
                return hash;
            }
        );
}
