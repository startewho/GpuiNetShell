using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A navigation trail built from an ordered list of labels passed to the
/// <c>Breadcrumb</c> factory on <see cref="RenderContext"/>.
/// </summary>
public sealed class BreadcrumbElement : Element
{
    internal BreadcrumbElement(RenderContext ui, int index)
        : base(ui, index) { }
}
