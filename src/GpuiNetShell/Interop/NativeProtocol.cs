namespace GpuiNetShell.Interop;

/// <summary>
/// The wire vocabulary shared with the native host. Every constant here mirrors
/// <c>crates/gpui-net-shell/src/schema.rs</c>. <see cref="SchemaHash"/> is the
/// value both sides must agree on; a test pins its literal on each side.
/// </summary>
public static class NativeProtocol
{
    public const uint AbiVersion = 1;

    /// <summary>Identifies the component/operation vocabulary below.</summary>
    public const ulong SchemaHash = 0x6E65_7473_6865_6C6C;

    // Components.
    public const uint ComponentDiv = 1;
    public const uint ComponentText = 2;
    public const uint ComponentButton = 3;

    // Shared style operations.
    public const ushort OpSize = 1;
    public const ushort OpWidth = 2;
    public const ushort OpHeight = 3;
    public const ushort OpPadding = 4;
    public const ushort OpPaddingX = 5;
    public const ushort OpPaddingY = 6;
    public const ushort OpGap = 7;
    public const ushort OpRounded = 8;
    public const ushort OpFlexCol = 9;
    public const ushort OpFlexRow = 10;
    public const ushort OpItemsCenter = 11;
    public const ushort OpJustifyCenter = 12;
    public const ushort OpSizeFull = 13;
    public const ushort OpBackgroundRgba = 14;
    public const ushort OpTextRgba = 15;
    public const ushort OpFontSize = 16;

    // Shared control operations.
    public const ushort OpDisabled = 20;
    public const ushort OpSelected = 21;
    public const ushort OpOnClick = 22;

    // Button operations.
    public const ushort OpLabel = 40;
    public const ushort OpTooltip = 41;
    public const ushort OpLoading = 42;
    public const ushort OpButtonVariant = 43;
    public const ushort OpButtonSize = 44;
    public const ushort OpCompact = 45;

    public const int StatusOk = 0;
}
