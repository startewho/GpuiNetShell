using System.Runtime.InteropServices;
using GpuiNetShell.Interop;

namespace GpuiNetShell.Tests;

/// <summary>
/// Loads the real native host to check ABI/schema negotiation.
/// </summary>
/// <remarks>
/// The native host imports <c>TaskDialogIndirect</c>, whose Common Controls v6
/// export only activates for a process carrying a v6 manifest. The sample
/// executable embeds one; the dotnet test host does not, so these tests skip
/// there rather than fail. Run <c>GpuiNetShell.Sample</c> for the real path.
/// </remarks>
public sealed unsafe class NativeHostTests
{
    [Fact]
    public void ExposesTheNegotiatedProtocol()
    {
        if (!TryLoadHost())
        {
            Assert.Skip("The native host needs a Common Controls v6 manifest.");
        }
        Assert.Equal(NativeProtocol.AbiVersion, NativeMethods.AbiVersion());
        Assert.Equal(NativeProtocol.SchemaHash, NativeMethods.SchemaHash());
    }

    [Fact]
    public void ReturnsAnApiTableForTheSupportedVersionOnly()
    {
        if (!TryLoadHost())
        {
            Assert.Skip("The native host needs a Common Controls v6 manifest.");
        }
        Assert.True(NativeMethods.GetApi(NativeProtocol.AbiVersion) != null);
        Assert.True(NativeMethods.GetApi(NativeProtocol.AbiVersion + 1) == null);
    }

    private static bool TryLoadHost()
    {
        try
        {
            _ = NativeMethods.AbiVersion();
            return true;
        }
        catch (DllNotFoundException)
        {
            return false;
        }
        catch (EntryPointNotFoundException)
        {
            return false;
        }
    }
}
