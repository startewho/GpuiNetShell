using GpuiNetShell.Entities;

namespace GpuiNetShell.Tests;

/// <summary>
/// The managed entity model: creation, identity, read/update, weak handles,
/// observe cascades, and typed events. Each test uses its own registry.
/// </summary>
public sealed class EntityTests
{
    private static EntityRegistry NewRegistry() => new();

    private sealed class Counter
    {
        public int Count { get; set; }
    }

    private sealed record Incremented(int Amount);

    [Fact]
    public void CreateRunsInitOnceAndAssignsAUniqueId()
    {
        var registry = NewRegistry();
        var inits = 0;
        var first = registry.Create<Counter>(_ =>
        {
            inits++;
            return new Counter();
        });
        var second = registry.Create<Counter>(_ => new Counter());

        Assert.Equal(1, inits);
        Assert.NotEqual(first.Id, second.Id);
        Assert.True(first.IsAlive);
    }

    [Fact]
    public void UpdateMutatesStateAndReadObserves()
    {
        var registry = NewRegistry();
        var counter = registry.Create<Counter>(_ => new Counter());

        counter.Update((state, _) => state.Count += 2);
        counter.Update((state, _) => state.Count += 3);

        Assert.Equal(5, counter.Read().Count);
    }

    [Fact]
    public void DowngradeUpgradesWhileAliveAndFailsAfterRelease()
    {
        var registry = NewRegistry();
        var counter = registry.Create<Counter>(_ => new Counter());
        var weak = counter.Downgrade();

        Assert.NotNull(weak.Upgrade());

        registry.Release(counter.Id.Value);

        Assert.False(counter.IsAlive);
        Assert.Null(weak.Upgrade());
    }

    [Fact]
    public void ObserveRunsWhenTheTargetNotifies()
    {
        var registry = NewRegistry();
        Counter? mirror = null;
        var source = registry.Create<Counter>(_ => new Counter { Count = 1 });

        var observer = registry.Create<Counter>(cx =>
        {
            cx.Observe(source, (state, target, _) => mirror = new Counter { Count = target.Read().Count * 2 });
            return new Counter();
        });

        source.Update((state, cx) =>
        {
            state.Count = 10;
            cx.Notify();
        });

        Assert.Equal(20, mirror!.Count);
        Assert.True(observer.IsAlive);
    }

    [Fact]
    public void ObserveCascadesThroughChainedEntities()
    {
        var registry = NewRegistry();
        var a = registry.Create<Counter>(_ => new Counter { Count = 1 });
        var b = registry.Create<Counter>(_ => new Counter());
        var c = registry.Create<Counter>(_ => new Counter());

        b.Update(
            (_, cx) =>
                cx.Observe(
                    a,
                    (state, target, context) =>
                    {
                        state.Count = target.Read().Count + 1;
                        context.Notify();
                    }
                )
        );

        c.Update(
            (state, cx) =>
                cx.Observe(
                    b,
                    (_, target, _) => state.Count = target.Read().Count + 1
                )
        );

        a.Update((state, cx) =>
        {
            state.Count = 5;
            cx.Notify();
        });

        Assert.Equal(6, b.Read().Count);
        Assert.Equal(7, c.Read().Count);
    }

    [Fact]
    public void SubscribeReceivesTypedEvents()
    {
        var registry = NewRegistry();
        var totals = new List<int>();
        var source = registry.Create<Counter>(_ => new Counter());

        _ = registry.Create<Counter>(cx =>
        {
            cx.Subscribe<Counter, Incremented>(
                source,
                (_, _, ev, _) => totals.Add(ev.Amount)
            );
            return new Counter();
        });

        source.Update((_, cx) =>
        {
            cx.Emit(new Incremented(2));
            cx.Emit(new Incremented(5));
        });

        Assert.Equal([2, 5], totals);
    }

    [Fact]
    public void SubscriptionsOfDifferentEventTypesAreIndependent()
    {
        var registry = NewRegistry();
        var increments = 0;
        var resets = 0;
        var source = registry.Create<Counter>(_ => new Counter());

        _ = registry.Create<Counter>(cx =>
        {
            cx.Subscribe<Counter, Incremented>(source, (_, _, _, _) => increments++);
            cx.Subscribe<Counter, string>(source, (_, _, _, _) => resets++);
            return new Counter();
        });

        source.Update((_, cx) =>
        {
            cx.Emit(new Incremented(1));
            cx.Emit("reset");
        });

        Assert.Equal(1, increments);
        Assert.Equal(1, resets);
    }

    [Fact]
    public void DisposeCancelsASubscription()
    {
        var registry = NewRegistry();
        var seen = 0;
        var source = registry.Create<Counter>(_ => new Counter());
        Subscription? subscription = null;

        _ = registry.Create<Counter>(cx =>
        {
            subscription = cx.Subscribe<Counter, Incremented>(source, (_, _, _, _) => seen++);
            return new Counter();
        });

        source.Update((_, cx) => cx.Emit(new Incremented(1)));
        subscription!.Dispose();
        source.Update((_, cx) => cx.Emit(new Incremented(1)));

        Assert.Equal(1, seen);
    }

    [Fact]
    public void AReEntrantNotifyDoesNotLoop()
    {
        var registry = NewRegistry();
        var source = registry.Create<Counter>(_ => new Counter());
        var observer = registry.Create<Counter>(cx =>
        {
            cx.Observe(source, (state, _, context) =>
            {
                state.Count++;
                context.Notify();
            });
            return new Counter();
        });

        source.Update((_, cx) => cx.Notify());

        Assert.Equal(1, observer.Read().Count);
    }
}
