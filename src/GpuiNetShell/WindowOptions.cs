namespace GpuiNetShell;

/// <summary>
/// Per-window presentation options for a window opened with
/// <see cref="GpuiApplication.OpenWindow(System.Func{View})"/>.
/// </summary>
/// <remarks>
/// Every option falls back, at open time, to the value the owning application
/// was configured with. Set <see cref="UseCustomTitlebar"/> to give a window a
/// different title bar than its parent, or <see langword="null"/> to inherit the
/// parent's choice.
/// </remarks>
public sealed class WindowOptions
{
    /// <summary>
    /// Draw a custom title bar instead of the OS one. <see langword="null"/>
    /// inherits the parent window's mode.
    /// </summary>
    public bool? UseCustomTitlebar { get; set; }

    public WindowOptions() { }

    public WindowOptions(bool? useCustomTitlebar) => UseCustomTitlebar = useCustomTitlebar;
}
