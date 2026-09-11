namespace GpuiNetShell.Interop;

/// <summary>
/// The wire vocabulary shared with the native host. Every constant here mirrors
/// <c>crates/gpui-net-shell/src/schema.rs</c>. <see cref="SchemaHash"/> is the
/// value both sides must agree on; a test pins its literal on each side.
/// </summary>
/// <remarks>
/// Styling is deliberately not enumerated. The host sends a GPUI style method
/// name and its argument; Rust resolves the name against GPUI's reflected style
/// table. Only component behavior and the style-call envelope have codes.
/// </remarks>
public static class NativeProtocol
{
    public const uint AbiVersion = 1;

    /// <summary>Identifies the component/operation vocabulary below.</summary>
    public const ulong SchemaHash = 0x6E65_7473_6865_6C6D;

    // Components.
    public const uint ComponentDiv = 1;
    public const uint ComponentText = 2;
    public const uint ComponentButton = 3;

    // Component behavior operations.
    public const ushort OpDisabled = 20;
    public const ushort OpSelected = 21;
    public const ushort OpOnClick = 22;

    public const ushort OpLabel = 40;
    public const ushort OpTooltip = 41;
    public const ushort OpLoading = 42;
    public const ushort OpButtonVariant = 43;
    public const ushort OpButtonSize = 44;
    public const ushort OpCompact = 45;

    // Generic style operations. `a` is the packed UTF-8 range of the style
    // method name; `b` is the argument (zero, an f32 bit pattern, or a packed
    // UTF-8 range).
    public const ushort OpStyleNullary = 50;
    public const ushort OpStyleLength = 51;
    public const ushort OpStyleNumber = 52;
    public const ushort OpStyleColor = 53;
    public const ushort OpStyleString = 54;

    public const int StatusOk = 0;
}
