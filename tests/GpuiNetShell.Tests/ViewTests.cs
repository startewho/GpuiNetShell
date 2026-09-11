using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Tests;

public sealed class ViewTests
{
    [Fact]
    public void InvalidateInvokesTheAttachedInvalidator()
    {
        var view = new ProbeView();
        var calls = 0;
        view.AttachInvalidator(() => calls++);

        view.Invalidate();

        Assert.Equal(1, calls);
    }

    [Fact]
    public void InvalidateBeforeAttachIsHarmless()
    {
        var view = new ProbeView();
        view.Invalidate();
    }

    private sealed class ProbeView : View
    {
        protected override Element Render(ref RenderContext ui) => ui.Div();
    }
}
