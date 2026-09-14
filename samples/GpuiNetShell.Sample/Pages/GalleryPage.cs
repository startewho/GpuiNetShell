using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// One gallery page demonstrating a single control. Pages are written one per
/// file; the host view assigns <see cref="Host"/> (and <see cref="Application"/>)
/// so a page can request a re-render or open an overlay after changing state.
/// </summary>
internal abstract class GalleryPage
{
    internal View Host { get; set; } = null!;

    internal GpuiApplication Application { get; set; } = null!;

    /// <summary>The label shown in the navigation sidebar.</summary>
    public abstract string Title { get; }

    /// <summary>The navigation icon shown in the sidebar.</summary>
    public virtual SidebarIcon Icon => SidebarIcon.Components;

    /// <summary>Requests a re-render from this page (whole-window).</summary>
    protected void Invalidate() => Host.Invalidate();

    /// <summary>Describes this page's content.</summary>
    public abstract Element Render(ref RenderContext ui);

    /// <summary>A titled, described page with the given children.</summary>
    protected static Element Page(
        ref RenderContext ui,
        string title,
        string description,
        params Element[] children
    )
    {
        var column = new List<Element> { Section(ref ui, title, description) };
        column.AddRange(children);
        return ui.VStack(column.ToArray()).Gap(16);
    }

    /// <summary>A page heading with a short description.</summary>
    protected static Element Section(ref RenderContext ui, string title, string description) =>
        ui.VStack(ui.Label(title).TextSize(22).FontSemibold(), ui.Text(description)).Gap(4);
}

/// <summary>
/// A gallery page whose state lives in an <see cref="Entity{T}"/>. Content is
/// rendered through <c>ui.Child</c>, so a manual <c>cx.Notify()</c> repaints only
/// this page's subtree rather than the whole window. Handlers use
/// <see cref="Update"/> instead of <see cref="GalleryPage.Invalidate"/>.
/// </summary>
internal abstract class GalleryPage<TState> : GalleryPage
    where TState : class, new()
{
    private Entity<TState>? _entity;

    /// <summary>The page's entity; created on first render.</summary>
    protected Entity<TState> Entity =>
        _entity ??= Application.New<TState>(_ => CreateState());

    /// <summary>Initializes the page state; override for non-default values.</summary>
    protected virtual TState CreateState() => new();

    /// <summary>
    /// Mutates the page state. The handler is responsible for calling
    /// <c>cx.Notify()</c>; that is the only thing that repaints the subtree.
    /// </summary>
    protected void Update(Action<TState, Context<TState>> update)
    {
        ArgumentNullException.ThrowIfNull(update);
        Entity.Update(update);
    }

    public sealed override Element Render(ref RenderContext ui) =>
        ui.Child(Entity, (state, context, cx) => RenderState(state, context, cx));

    /// <summary>Describes this page's content from its entity state.</summary>
    protected abstract Element RenderState(TState state, RenderContext ui, Context<TState> cx);
}
