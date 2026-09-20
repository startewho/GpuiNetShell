using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.FileManager;

/// <summary>
/// A Windows 11 style file manager. The root view owns an
/// <see cref="Entity{T}"/>; the whole interface is rendered from that entity, so
/// a change repaints only this subtree and not the window.
/// </summary>
/// <remarks>
/// The UI is split across partial files by area:
/// <list type="bullet">
/// <item><c>FileManagerView.Tabs.cs</c> — the title bar tab strip.</item>
/// <item><c>FileManagerView.Search.cs</c> — the title bar and its search box.</item>
/// <item><c>FileManagerView.Toolbar.cs</c> — the navigation row and the optional command row.</item>
/// <item><c>FileManagerView.Breadcrumbs.cs</c> — the address bar and its folder dropdowns.</item>
/// <item><c>FileManagerView.Tree.cs</c> — the navigation tree.</item>
/// <item><c>FileManagerView.Content.cs</c> — the details/icons panes and status bar.</item>
/// <item><c>FileManagerView.Settings.cs</c> — the settings popover (theme and toolbar).</item>
/// <item><c>FileManagerView.Commands.cs</c> — navigation, tab, and file commands.</item>
/// </list>
/// </remarks>
[GpuiCallbacks]
internal sealed partial class FileManagerView : View
{
    /// <summary>A translucent separator that reads on both light and dark backgrounds.</summary>
    private const string Divider = "#80808055";

    /// <summary>
    /// A neutral highlight for a selected control (active tab, view toggle). The
    /// theme accent is reserved for the selected folder in the file list.
    /// </summary>
    private const string NeutralSelection = "#80808040";

    /// <summary>How many recursive search matches are kept.</summary>
    private const int SearchLimit = 2000;

    private readonly GpuiApplication _application;
    private Entity<FileManagerState>? _entity;
    private bool _themeApplied;
    /// <summary>The tab strip viewport width, measured from layout; 0 until known.</summary>
    private double _tabStripWidth;
    /// <summary>The first tab index shown when the strip is paginated.</summary>
    private int _tabStart;

    public FileManagerView(GpuiApplication application, string? initialPath)
    {
        _application = application;
        var path =
            initialPath is { Length: > 0 } && Directory.Exists(initialPath)
                ? initialPath
                : FileSystemService.DefaultPath();

        var listing = FileSystemService.ListDirectory(path);
        Entity.Update(
            (state, cx) =>
            {
                var tab = state.AddTab();
                state.ApplyListing(tab, listing, recordHistory: true);
                BuildTree(state);
                PreloadCrumbs(cx, state, tab);
            }
        );

        OnInput(HandleShortcut);
    }

    private Entity<FileManagerState> Entity =>
        _entity ??= _application.New<FileManagerState>(_ => new FileManagerState());

    protected override Element Render(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        EnsureTheme();
        return ui.Child(Entity, PageToken);
    }

    /// <summary>The title bar carries the tabs, the search box, and settings.</summary>
    protected override Element? RenderTitleBar(ref RenderContext ui) =>
        BuildTitleBar(ui, Entity.Read());

    /// <summary>
    /// Measures the tab strip viewport. The grid of tabs is paginated so only
    /// whole tabs are shown; the page size comes from this width.
    /// </summary>
    [GpuiCallback("MeasureTabStrip")]
    private string MeasureTabStrip(double availableWidth, double availableHeight)
    {
        if (availableWidth > 1 && Math.Abs(availableWidth - _tabStripWidth) > 0.5)
        {
            _tabStripWidth = availableWidth;
            Invalidate();
        }
        return "0\t0";
    }

    /// <summary>Applies the configured accent once, after the window exists.</summary>
    private void EnsureTheme()
    {
        if (_themeApplied)
        {
            return;
        }
        _themeApplied = true;
        var state = Entity.Read();
        _application.SetTheme(state.Mode, ThemePresets.Palette(state.AccentHex));
    }

    private void Update(Action<FileManagerState, Context<FileManagerState>> update)
    {
        Entity.Update(update);
        // The title bar (tabs, search, settings) is rendered from the main
        // snapshot, not the entity subtree, so a state change must also
        // invalidate the window for it to repaint.
        Invalidate();
    }

    // -- Page ---------------------------------------------------------------

    [GpuiCallback("Page")]
    private Element RenderPage(
        FileManagerState state,
        RenderContext ui,
        Context<FileManagerState> cx
    ) =>
        ui.VStack(
                BuildToolbar(ui, state, cx),
                ui.HStack(BuildTreePane(ui, state, cx), BuildContentPane(ui, state, cx))
                    .Gap(0)
                    .Flex1()
                    .MinH(0),
                BuildStatusBar(ui, state)
            )
            .Full();
}
