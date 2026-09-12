using GpuiNetShell.Elements;
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

    /// <summary>Requests a re-render from this page.</summary>
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
