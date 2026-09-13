using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Tests;

/// <summary>
/// P6: UI-thread marshalling (<c>Context.Spawn</c>) and process-wide global
/// state. Entity state is read on the UI thread, so a spawned completion must be
/// queued and drained there.
/// </summary>
public sealed class EntityP6Tests
{
    private sealed class Counter
    {
        public int Count { get; set; }
    }

    [Fact]
    public void SpawnActionRunsOnDrainWithCurrentState()
    {
        UiDispatcher.ResetForTests();
        var registry = new EntityRegistry();
        var counter = registry.Create<Counter>(_ => new Counter { Count = 4 });
        var applied = 0;

        counter.Read(out var cx);
        cx.Spawn((state, _) => applied = state.Count * 2);

        Assert.Equal(0, applied);
        Assert.True(UiDispatcher.PendingCount > 0);

        UiDispatcher.Drain();

        Assert.Equal(8, applied);
    }

    [Fact]
    public void SpawnOnAReleasedEntityIsDropped()
    {
        UiDispatcher.ResetForTests();
        var registry = new EntityRegistry();
        var counter = registry.Create<Counter>(_ => new Counter());
        var ran = false;

        counter.Read(out var cx);
        cx.Spawn((_, _) => ran = true);
        registry.Release(counter.Id.Value);

        UiDispatcher.Drain();

        Assert.False(ran);
    }

    [Fact]
    public async Task SpawnBackgroundWorkAppliesItsResultOnTheUiThread()
    {
        UiDispatcher.ResetForTests();
        var registry = new EntityRegistry();
        var counter = registry.Create<Counter>(_ => new Counter());

        counter.Read(out var cx);
        cx.Spawn(
            async _ =>
            {
                await Task.Yield();
                return 5;
            },
            (state, result, _) => state.Count += result
        );

        var deadline = DateTime.UtcNow.AddSeconds(5);
        while (UiDispatcher.PendingCount == 0 && DateTime.UtcNow < deadline)
        {
            await Task.Delay(10, TestContext.Current.CancellationToken);
        }
        Assert.True(UiDispatcher.PendingCount > 0, "the background result was not posted");

        UiDispatcher.Drain();

        Assert.Equal(5, registry.Read<Counter>(counter.Id.Value).Count);
    }

    [Fact]
    public async Task SpawnBackgroundFailureRunsTheErrorHandler()
    {
        UiDispatcher.ResetForTests();
        var registry = new EntityRegistry();
        var counter = registry.Create<Counter>(_ => new Counter());
        var failure = new TaskCompletionSource<Exception>(
            TaskCreationOptions.RunContinuationsAsynchronously
        );

        counter.Read(out var cx);
        cx.Spawn<int>(
            _ => throw new InvalidOperationException("boom"),
            (_, _, _) => { },
            exception => failure.TrySetResult(exception)
        );

        var exception = await failure.Task.WaitAsync(
            TimeSpan.FromSeconds(5),
            TestContext.Current.CancellationToken
        );
        Assert.IsType<InvalidOperationException>(exception);
        Assert.Equal(0, UiDispatcher.PendingCount);
    }

    [Fact]
    public void GlobalStoreRoundTripsAndNotifies()
    {
        var store = new GlobalStore();
        var changed = 0;
        Action<Type> handler = _ => changed++;
        GlobalStore.Changed += handler;
        try
        {
            Assert.False(store.TryGet<Counter>(out _));

            store.Set(new Counter { Count = 1 });
            Assert.True(store.TryGet<Counter>(out var value));
            Assert.Equal(1, value.Count);

            store.Set(new Counter { Count = 2 });
            Assert.Equal(2, store.Get<Counter>()!.Count);

            Assert.True(store.Remove<Counter>());
            Assert.False(store.Remove<Counter>());
        }
        finally
        {
            GlobalStore.Changed -= handler;
        }

        Assert.Equal(3, changed);
    }

    [Fact]
    public void AContextReachesGlobalState()
    {
        var registry = new EntityRegistry();
        var counter = registry.Create<Counter>(_ => new Counter());
        counter.Read(out var cx);

        Assert.Null(cx.TryGlobal<Counter>());

        cx.SetGlobal(new Counter { Count = 7 });

        Assert.Equal(7, cx.TryGlobal<Counter>()!.Count);

        GlobalStore.Default.Remove<Counter>();
    }

    [Fact]
    public void TheApplicationExposesGlobalState()
    {
        var application = new GpuiApplication(() => new ProbeView());

        application.SetGlobal(new Counter { Count = 3 });

        Assert.True(application.TryGetGlobal<Counter>(out var value));
        Assert.Equal(3, value.Count);
        Assert.Equal(3, application.Global<Counter>()!.Count);

        GlobalStore.Default.Remove<Counter>();
    }

    private sealed class ProbeView : View
    {
        protected override Element Render(ref RenderContext ui) => ui.Div();
    }
}
