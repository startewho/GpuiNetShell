using GpuiNetShell;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

// --check loads the native host and negotiates the ABI/schema without opening a
// window. It is the quickest end-to-end check of the C ABI boundary.
if (args.Contains("--check", StringComparer.Ordinal))
{
    var compatible = GpuiNativeHost.Verify();
    Console.WriteLine(
        compatible
            ? $"gpui-net-shell native host OK (abi {GpuiNativeHost.AbiVersion}, schema 0x{GpuiNativeHost.SchemaHash:X16})"
            : "gpui-net-shell native host is incompatible"
    );
    return compatible ? 0 : 1;
}

// The application is captured by the view so its buttons can open overlays; the
// factory runs on the native thread after this assignment.
var initialPage = 0;
var pageArgument = args.FirstOrDefault(argument =>
    argument.StartsWith("--page=", StringComparison.Ordinal)
);
if (
    pageArgument is not null
    && int.TryParse(pageArgument["--page=".Length..], out var parsedPage)
)
{
    initialPage = parsedPage;
}

GpuiApplication? application = null;
application = new GpuiApplication(() => new GalleryView(application!, initialPage));
application.Run();
return 0;

/// <summary>
/// A tabbed component gallery: one page per component, each showing how to use
/// it. The tab bar itself is the <c>Tabs</c> component.
/// </summary>
internal sealed class GalleryView : View
{
    private readonly GpuiApplication _application;
    private readonly string[] _pages =
    [
        "Button",
        "Label & Badge",
        "Progress",
        "Combobox",
        "Radio",
        "Scroll",
        "Resizable",
        "Popover",
        "Overlays",
        "Feedback",
    ];
    private readonly string[] _themes = ["Light", "Dark", "System"];
    private readonly string[] _options = ["Light", "Dark", "System"];

    private int _page;
    private int _themeIndex;
    private int _radioIndex;

    public GalleryView(GpuiApplication application, int initialPage = 0)
    {
        _application = application;
        _page = initialPage;
    }

    protected override Element Render(ref RenderContext ui)
    {
        var page = _page switch
        {
            0 => ButtonPage(ref ui),
            1 => LabelPage(ref ui),
            2 => ProgressPage(ref ui),
            3 => ComboboxPage(ref ui),
            4 => RadioPage(ref ui),
            5 => ScrollPage(ref ui),
            6 => ResizablePage(ref ui),
            7 => PopoverPage(ref ui),
            8 => OverlayPage(ref ui),
            9 => FeedbackPage(ref ui),
            _ => OverlayPage(ref ui),
        };

        return ui.VStack(
                ui.Tabs("pages").Options(_pages).Selected(_page).OnChange(index => _page = index),
                page
            )
            .Gap(20)
            .P(24)
            .Full()
            .ItemsStart();
    }

    private Element ButtonPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Button", "Variants, sizes, and the loading/disabled states."),
                ui.HStack(
                        ui.Button("b-primary")
                            .Label("Primary")
                            .Primary()
                            .Tooltip("Primary action")
                            .OnClick(() => { }),
                        ui.Button("b-secondary").Label("Secondary").Secondary(),
                        ui.Button("b-danger").Label("Danger").Danger(),
                        ui.Button("b-success").Label("Success").Success(),
                        ui.Button("b-ghost").Label("Ghost").Ghost(),
                        ui.Button("b-link").Label("Link").Link()
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.HStack(
                        ui.Button("b-small").Label("Small").Size(ButtonSize.Small),
                        ui.Button("b-medium").Label("Medium").Size(ButtonSize.Medium).Primary(),
                        ui.Button("b-large").Label("Large").Size(ButtonSize.Large),
                        ui.Button("b-loading").Label("Loading").Loading(),
                        ui.Button("b-disabled").Label("Disabled").Disabled()
                    )
                    .Gap(8)
                    .ItemsCenter()
            )
            .Gap(16);

    private Element LabelPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Label & Badge", "Labels at different sizes, badges for counts and dots."),
                ui.HStack(
                        ui.Label("Default"),
                        ui.Label("Small").TextSize(12),
                        ui.Label("Bold").FontBold(),
                        ui.Label("Semibold").FontSemibold()
                    )
                    .Gap(16)
                    .ItemsCenter(),
                ui.HStack(
                        ui.Label("Inbox"),
                        ui.Badge().Count(12),
                        ui.Label("Errors"),
                        ui.Badge().Count(3),
                        ui.Label("Online"),
                        ui.Badge().Dot()
                    )
                    .Gap(12)
                    .ItemsCenter()
            )
            .Gap(16);

    private Element ProgressPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Progress", "Determinate values and the indeterminate animation."),
                ui.Progress("p-25").Value(25).Full(),
                ui.Progress("p-60").Value(60).Full(),
                ui.Progress("p-90").Value(90).Full(),
                ui.Progress("p-loading").Loading().Full()
            )
            .Gap(12);

    private Element ComboboxPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Combobox", "Opens its options as a Bottom sheet; the choice is controlled."),
                ui.HStack(
                        ui.Label("Theme"),
                        ui.Combobox("theme")
                            .Options(_options)
                            .Selected(_themeIndex)
                            .OnChange(index => _themeIndex = index)
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.Label($"Selected theme: {_options[_themeIndex]}")
            )
            .Gap(12);

    private Element RadioPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Radio", "Controlled options; selecting one is reported through its token."),
                ui.HStack(
                        ui.Radio("r-light")
                            .Label("Light")
                            .Checked(_radioIndex == 0)
                            .OnChange(_ => _radioIndex = 0),
                        ui.Radio("r-dark")
                            .Label("Dark")
                            .Checked(_radioIndex == 1)
                            .OnChange(_ => _radioIndex = 1),
                        ui.Radio("r-system")
                            .Label("System")
                            .Checked(_radioIndex == 2)
                            .OnChange(_ => _radioIndex = 2)
                    )
                    .Gap(16)
                    .ItemsCenter(),
                ui.Label($"Selected option: {_options[_radioIndex]}")
            )
            .Gap(12);

    private Element OverlayPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Overlays", "Dialogs, sheets and notifications are native layers over the content."),
                ui.HStack(
                        ui.Button("dialog")
                            .Label("Open dialog")
                            .Primary()
                            .OnClick(() =>
                                _application.OpenDialog(
                                    "About GpuiNetShell",
                                    "A C#-hosted GPUI shell: managed state, native rendering."
                                )
                            ),
                        ui.Button("sheet")
                            .Label("Open sheet")
                            .OnClick(() =>
                                _application.OpenSheet(
                                    SheetPlacement.Right,
                                    "Details",
                                    "A sheet is a place in the window, below the dialog stack."
                                )
                            ),
                        ui.Button("notify")
                            .Label("Notify")
                            .Success()
                            .OnClick(() =>
                                _application.PushNotification(
                                    "Saved",
                                    NotificationLevel.Success
                                )
                            )
                    )
                    .Gap(8)
                    .ItemsCenter()
            )
            .Gap(12);

    private Element ScrollPage(ref RenderContext ui)
    {
        var rows = new List<Element>();
        for (var i = 1; i <= 20; i++)
        {
            rows.Add(ui.Label($"Row {i}"));
        }

        return ui.VStack(
                Section(ref ui, "Scroll", "A scrollable area and a bar that drives it by name."),
                ui.Div(
                        ui.Scroll("scroller")
                            .Axis(ScrollAxis.Vertical)
                            .Full()
                            .Add(ui.VStack(rows.ToArray()).Gap(8).P(8)),
                        ui.Scrollbar("scroller").Axis(ScrollAxis.Vertical)
                    )
                    .W(320)
                    .H(200)
                    .Relative()
            )
            .Gap(12);
    }

    private Element ResizablePage(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Resizable",
                    "Draggable panels; sizes are pixels, `*` is the flexible panel."
                ),
                ui.Resizable("panels")
                    .Axis(ResizeAxis.Horizontal)
                    .Sizes("160", "*", "160")
                    .H(160)
                    .Add(
                        ui.Div(ui.Label("Left")).P(12).Full(),
                        ui.Div(ui.Label("Center")).P(12).Full(),
                        ui.Div(ui.Label("Right")).P(12).Full()
                    )
            )
            .Gap(12);

    private Element PopoverPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Popover", "A button trigger with anchored content in the `content` slot."),
                ui.Popover("popover", "Show details")
                    .DefaultOpen()
                    .OverlayClosable()
                    .Content(
                        ui.VStack(
                                ui.Label("Details"),
                                ui.Text("Anchored content painted above the window.")
                            )
                            .Gap(4)
                            .P(8)
                    )
            )
            .Gap(12);

    private Element FeedbackPage(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Feedback",
                    "Spinners, separators, skeletons, and status tags."
                ),
                ui.HStack(
                        ui.Spinner(),
                        ui.Spinner().Size(ControlSize.Small),
                        ui.Spinner().Icon(SpinnerIcon.LoaderCircle).Color("blue-600"),
                        ui.Spinner().Size(ControlSize.Large).Ease(SpinnerEase.EaseOutQuint)
                    )
                    .Gap(20)
                    .ItemsCenter(),
                ui.Separator(),
                ui.Separator().Label("Account"),
                ui.DashedSeparator().Color("red-500"),
                ui.HStack(
                        ui.Skeleton().W(80).H(12),
                        ui.Skeleton().Secondary().W(120).H(12),
                        ui.Skeleton().W(60).H(12)
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.HStack(
                        ui.Tag().Variant(TagVariant.Primary).Add(ui.Text("Primary")),
                        ui.Tag().Variant(TagVariant.Success).RoundedFull().Add(ui.Text("Success")),
                        ui.Tag().Variant(TagVariant.Danger).Outline().Add(ui.Text("Danger")),
                        ui.Tag().Variant(TagVariant.Info).Size(ControlSize.Small).Add(ui.Text("Info"))
                    )
                    .Gap(8)
                    .ItemsCenter()
            )
            .Gap(16);

    private Element Section(ref RenderContext ui, string title, string description) =>
        ui.VStack(ui.Label(title).TextSize(20).FontSemibold(), ui.Text(description)).Gap(4);
}
