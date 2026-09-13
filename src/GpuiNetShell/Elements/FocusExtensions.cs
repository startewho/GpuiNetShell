namespace GpuiNetShell.Elements;

/// <summary>
/// Focus/blur callbacks. They are meaningful on the retained input family
/// (<c>Input</c>, <c>NumberInput</c>, <c>Textarea</c>, <c>OtpInput</c>,
/// <c>Editor</c>), which declare the <c>on_focus</c>/<c>on_blur</c> methods;
/// other components ignore them.
/// </summary>
public static class FocusExtensions
{
    /// <summary>Runs when the element gains focus.</summary>
    public static T OnFocus<T>(this T element, Action handler)
        where T : Element
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = element.Events.Register(handler);
        element.Arena.AddCallback(element.Index, "on_focus", token);
        return element;
    }

    /// <summary>Runs when the element loses focus.</summary>
    public static T OnBlur<T>(this T element, Action handler)
        where T : Element
    {
        ArgumentNullException.ThrowIfNull(handler);
        var token = element.Events.Register(handler);
        element.Arena.AddCallback(element.Index, "on_blur", token);
        return element;
    }
}
