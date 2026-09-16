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

    [Fact]
    public void RowProvidersResolveAndRetireWithTheirGeneration()
    {
        var registry = new EventRegistry();

        registry.BeginGeneration(1);
        var token = registry.RegisterRows(() => "a\tb\nc\td");

        Assert.True(registry.TryGetRows(token, out var provider));
        Assert.Equal("a\tb\nc\td", provider());

        registry.BeginGeneration(2);
        registry.Retire(1);

        Assert.False(registry.TryGetRows(token, out _));
    }

    [Fact]
    public void ElementRenderersResolveAndRetireWithTheirGeneration()
    {
        var registry = new EventRegistry();

        registry.BeginGeneration(1);
        var token = registry.RegisterElement(
            (context, arguments) => context.Label(arguments[0])
        );

        Assert.True(registry.TryGetElement(token, out var renderer));
        Assert.NotNull(renderer);

        registry.BeginGeneration(2);
        registry.Retire(1);

        Assert.False(registry.TryGetElement(token, out _));
    }

    [Fact]
    public void APersistentElementOutlivesGenerationsAndReplacesInPlace()
    {
        var registry = new EventRegistry();

        registry.BeginGeneration(1);
        var first = registry.RegisterPersistentElement(7, (context, _) => context.Label("a"));
        registry.MarkEntityRendered(7);

        registry.BeginGeneration(2);
        registry.Retire(1);

        // The entity was rendered last generation, so its renderer is retained.
        Assert.True(registry.TryGetElement(first, out _));

        registry.MarkEntityRendered(7);
        Assert.True(registry.RendersEntity(7));

        var second = registry.RegisterPersistentElement(7, (context, _) => context.Label("b"));
        // Re-registering reuses the token so a retained subtree that still
        // references it keeps working; only the handler is replaced.
        Assert.Equal(first, second);
        Assert.True(registry.TryGetElement(first, out _));
        Assert.Equal(1, registry.ElementRendererCount);

        registry.ReleasePersistentElement(7);
        Assert.False(registry.TryGetElement(second, out _));
    }

    [Fact]
    public void AnEntityThatLeavesTheTreeReleasesItsPersistentRenderer()
    {
        var registry = new EventRegistry();

        registry.BeginGeneration(1);
        var token = registry.RegisterPersistentElement(9, (context, _) => context.Label("a"));
        registry.MarkEntityRendered(9);
        Assert.True(registry.TryGetElement(token, out _));

        // Generation 2 never renders entity 9; generation 3 prunes it.
        registry.BeginGeneration(2);
        registry.BeginGeneration(3);
        Assert.False(registry.TryGetElement(token, out _));
    }

    [Fact]
    public void RenderedEntitiesAreScopedToTheCurrentGeneration()
    {
        var registry = new EventRegistry();

        registry.BeginGeneration(1);
        registry.MarkEntityRendered(3);
        Assert.True(registry.RendersEntity(3));

        registry.BeginGeneration(2);
        Assert.False(registry.RendersEntity(3));
    }

    [Fact]
    public void EntityViewsReplaceByKeyAndOutliveGenerations()
    {
        var registry = new EventRegistry();

        var first = registry.RegisterEntityView("view", (context, _) => context.Label("a"));
        Assert.True(registry.TryGetElement(first, out _));

        registry.BeginGeneration(2);
        registry.Retire(1);

        // Entity views are keyed by name, not generation, so they persist.
        Assert.True(registry.TryGetElement(first, out _));

        var second = registry.RegisterEntityView("view", (context, _) => context.Label("b"));
        // The keyed token is reused; the handler is replaced in place.
        Assert.Equal(first, second);
        Assert.True(registry.TryGetElement(first, out _));
        Assert.Equal(1, registry.ElementRendererCount);
    }

    [Fact]
    public void ARepaintScopeRetiresThePreviousInvocationsHandlers()
    {
        var registry = new EventRegistry();
        registry.BeginGeneration(1);

        registry.BeginCallbackScope(42);
        var firstHandler = registry.Register(() => { });
        var firstElement = registry.RegisterElement((context, _) => context.Label("a"));
        var firstRows = registry.RegisterRows(() => "a");
        registry.EndCallbackScope();

        registry.BeginCallbackScope(42);
        var secondHandler = registry.Register(() => { });
        var secondElement = registry.RegisterElement((context, _) => context.Label("b"));
        registry.EndCallbackScope();

        // The first invocation's handlers are gone; the second's are live.
        Assert.False(registry.Dispatch(firstHandler));
        Assert.False(registry.TryGetElement(firstElement, out _));
        Assert.False(registry.TryGetRows(firstRows, out _));
        Assert.True(registry.Dispatch(secondHandler));
        Assert.True(registry.TryGetElement(secondElement, out _));
    }

    [Fact]
    public void RepeatedRepaintsKeepTheHandlerTablesBounded()
    {
        var registry = new EventRegistry();
        registry.BeginGeneration(1);

        for (var i = 0; i < 5_000; i++)
        {
            registry.BeginCallbackScope(7);
            registry.Register(() => { });
            registry.RegisterElement((context, _) => context.Label("x"));
            registry.RegisterRows(() => "x");
            registry.EndCallbackScope();
        }

        Assert.Equal(1, registry.HandlerCount);
        Assert.Equal(1, registry.ElementRendererCount);
        Assert.Equal(1, registry.RowProviderCount);
    }

    [Fact]
    public void ReplacingAnEntityViewRetiresItsRepaintScope()
    {
        var registry = new EventRegistry();
        registry.BeginGeneration(1);

        var first = registry.RegisterEntityView("view", (context, _) => context.Label("a"));
        registry.BeginCallbackScope(first);
        var handler = registry.Register(() => { });
        registry.EndCallbackScope();

        var second = registry.RegisterEntityView("view", (context, _) => context.Label("b"));

        // The token is reused, but the previous invocation's scope is retired.
        Assert.Equal(first, second);
        Assert.False(registry.Dispatch(handler));
    }
}
