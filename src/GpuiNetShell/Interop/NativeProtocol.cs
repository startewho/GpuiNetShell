namespace GpuiNetShell.Interop;

/// <summary>
/// The wire vocabulary shared with the native host. Every constant here mirrors
/// <c>crates/gpui-net-shell/src/schema.rs</c>. <see cref="SchemaHash"/> is the
/// value both sides must agree on; a test pins its literal on each side.
/// </summary>
/// <remarks>
/// The operation set mirrors <c>gpui-shell</c>'s <c>SpecOp</c>. Styling is not
/// enumerated per property: a style call carries a GPUI method name and an
/// argument that the native host resolves against GPUI's reflected style table.
/// Component behavior is a generic <see cref="OpMethod"/>, and event bindings
/// are a generic <see cref="OpCallback"/>.
/// </remarks>
public static class NativeProtocol
{
    public const uint AbiVersion = 1;

    /// <summary>Identifies the component/operation vocabulary below.</summary>
    public const ulong SchemaHash = 0x6E65_7473_6865_6C6F;

    // Components. Ids are registry indices: the native host resolves them
    // against the registered component catalog.
    public const uint ComponentDiv = 0;
    public const uint ComponentText = 1;
    public const uint ComponentButton = 2;

    // Operations. `a` is the packed UTF-8 range of a method name; `flags`
    // classifies the argument in `b`.
    public const ushort OpNullaryStyle = 1;
    public const ushort OpParamStyle = 2;
    public const ushort OpMethod = 3;
    public const ushort OpCallback = 4;

    public const ushort ArgNone = 0;
    public const ushort ArgNumber = 1;
    public const ushort ArgString = 2;

    public const int StatusOk = 0;
}
