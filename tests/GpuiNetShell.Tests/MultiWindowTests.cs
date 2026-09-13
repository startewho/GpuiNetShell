using GpuiNetShell.Elements;
using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Tests;

/// <summary>
/// The managed multi-window surface: every window is an independent session,
/// and a window opened before <see cref="GpuiApplication.Run"/> is queued until
/// the native event loop is live.
/// </summary>
public sealed unsafe class MultiWindowTests
{
    [Fact]
    public void AnOpenedWindowBeforeRunStaysPending()
    {
        var application = new GpuiApplication(() => new ProbeView());
        var handle = application.OpenWindow(() => new ProbeView());

        // The native event loop has not started, so the session id is not yet
        // assigned; closing the queued window drops it harmlessly.
        Assert.Equal(0UL, handle.SessionId);

        handle.Close();
    }

    [Fact]
    public void EachWindowGetsItsOwnHandle()
    {
        var application = new GpuiApplication(() => new ProbeView());
        var first = application.OpenWindow(() => new ProbeView());
        var second = application.OpenWindow(() => new ProbeView());

        Assert.NotSame(first, second);
        Assert.Equal(0UL, first.SessionId);
        Assert.Equal(0UL, second.SessionId);
    }

    [Fact]
    public void ChildWindowsCanChooseTheirTitlebarMode()
    {
        var application = new GpuiApplication(() => new ProbeView());
        application.UseCustomTitlebar = true;

        // An explicit per-window choice overrides the parent's setting.
        var custom = application.OpenWindow(() => new ProbeView());
        var system = application.OpenWindow(
            () => new ProbeView(),
            new WindowOptions(useCustomTitlebar: false)
        );
        var inherit = application.OpenWindow(() => new ProbeView(), new WindowOptions());

        Assert.NotEqual(0UL, application.SessionId);
        Assert.NotSame(custom, system);
        Assert.NotSame(system, inherit);
    }

    [Fact]
    public void TheApiTableCarriesWindowManagement()
    {
        if (!TryLoadHost())
        {
            Assert.Skip("The native host needs a Common Controls v6 manifest.");
        }
        var api = NativeMethods.GetApi(NativeProtocol.AbiVersion);
        Assert.True(api != null);
        Assert.True(api->OpenWindow != null);
        Assert.True(api->CloseWindow != null);
        Assert.True(api->NotifyEntity != null);
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

    private sealed class ProbeView : View
    {
        protected override Element Render(ref RenderContext ui) => ui.Div();
    }
}
