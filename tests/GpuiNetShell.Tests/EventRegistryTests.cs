using GpuiNetShell.Events;

namespace GpuiNetShell.Tests;

public sealed class EventRegistryTests
{
    [Fact]
    public void DispatchRunsTheRegisteredHandlerOnce()
    {
        var registry = new EventRegistry();
        var calls = 0;
        var token = registry.Register(() => calls++);

        Assert.True(registry.Dispatch(token));
        Assert.Equal(1, calls);
    }

    [Fact]
    public void ResetRetiresHandlersButKeepsTokensMonotonic()
    {
        var registry = new EventRegistry();
        var first = registry.Register(() => { });

        registry.Reset();
        var second = registry.Register(() => { });

        Assert.False(registry.Dispatch(first));
        Assert.True(second > first);
    }

    [Fact]
    public void UnknownTokenIsHarmless()
    {
        var registry = new EventRegistry();
        Assert.False(registry.Dispatch(42));
    }
}
