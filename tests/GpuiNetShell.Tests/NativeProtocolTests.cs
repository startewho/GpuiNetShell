using GpuiNetShell.Interop;

namespace GpuiNetShell.Tests;

public sealed class NativeProtocolTests
{
    [Fact]
    public void SchemaHashMatchesTheNativeLiteral()
    {
        Assert.Equal(0x6E65_7473_6865_6C6CUL, NativeProtocol.SchemaHash);
    }

    [Fact]
    public void AbiVersionIsOne()
    {
        Assert.Equal(1u, NativeProtocol.AbiVersion);
    }
}
