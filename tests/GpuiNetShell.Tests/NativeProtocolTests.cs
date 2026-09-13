using GpuiNetShell.Interop;

namespace GpuiNetShell.Tests;

public sealed class NativeProtocolTests
{
    [Fact]
    public void SchemaHashMatchesTheNativeLiteral()
    {
        Assert.Equal(0x6E65_7473_6865_6C55UL, NativeProtocol.SchemaHash);
    }

    [Fact]
    public void AbiVersionMatchesTheNativeLiteral()
    {
        Assert.Equal(7u, NativeProtocol.AbiVersion);
    }
}
