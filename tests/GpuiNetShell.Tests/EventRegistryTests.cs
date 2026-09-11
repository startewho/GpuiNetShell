using GpuiNetShell.Events;

namespace GpuiNetShell.Tests;

public sealed class EventRegistryTests
{
    [Fact]
    public void DispatchRunsTheRegisteredHandlerOnce()
    {
        var registry = new EventRegistry();
        registry.BeginGeneration(1);
        var calls = 0;
        var token = registry.Register(() => calls++);

        Assert.True(registry.Dispatch(token));
        Assert.Equal(1, calls);
    }

    [Fact]
    public void RetiringAGenerationReleasesOnlyItsHandlers()
    {
        var registry = new EventRegistry();

        registry.BeginGeneration(1);
        var first = registry.Register(() => { });

        registry.BeginGeneration(2);
        var second = registry.Register(() => { });

        registry.Retire(1);

        Assert.False(registry.Dispatch(first));
        Assert.True(registry.Dispatch(second));
    }

    [Fact]
    public void TokensAreNeverReusedAcrossGenerations()
    {
        var registry = new EventRegistry();
        registry.BeginGeneration(1);
        var first = registry.Register(() => { });
        registry.BeginGeneration(2);
        var second = registry.Register(() => { });

        Assert.True(second > first);
    }

    [Fact]
    public void UnknownTokenIsHarmless()
    {
        var registry = new EventRegistry();
        Assert.False(registry.Dispatch(42));
    }
}
