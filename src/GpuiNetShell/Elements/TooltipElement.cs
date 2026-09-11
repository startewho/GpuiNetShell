using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>
/// A button trigger with a text tooltip. Identity, trigger label, and tooltip
/// text all arrive through the factory on <c>RenderContext</c>.
/// </summary>
public sealed class TooltipElement : Element
{
    internal TooltipElement(RenderContext ui, int index)
        : base(ui, index) { }
}
