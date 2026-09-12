using GpuiNetShell;
using GpuiNetShell.Elements;
using GpuiNetShell.Events;
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
        "Elements",
        "Interactive",
        "Display",
        "Menus",
        "Collections",
        "Disclosure",
        "Structure",
        "Input Monitor",
        "Text Input",
    ];
    private readonly string[] _themes = ["Light", "Dark", "System"];
    private readonly string[] _options = ["Light", "Dark", "System"];

    private int _page;
    private int _themeIndex;
    private int _radioIndex;
    private int _rating = 3;
    private int _tabIndex;
    private string _selected = "";
    private int _step = 1;
    private string _lastInput = "(none)";
    private long _inputCount;
    private long _moveCount;
    private string _typed = "";
    private string _number = "";
    private string _textarea = "";
    private string _otp = "";
    private double _slider = 25;

    public GalleryView(GpuiApplication application, int initialPage = 0)
    {
        _application = application;
        _page = initialPage;
        OnInput(input =>
        {
            _lastInput = input.ToString();
            _inputCount++;
            if (input.Kind == InputEventKind.MouseMove)
            {
                _moveCount++;
                return;
            }
            Invalidate();
        });
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
            10 => ElementsPage(ref ui),
            11 => InteractivePage(ref ui),
            12 => DisplayPage(ref ui),
            13 => MenusPage(ref ui),
            14 => CollectionsPage(ref ui),
            15 => DisclosurePage(ref ui),
            16 => StructurePage(ref ui),
            17 => InputMonitorPage(ref ui),
            18 => TextInputPage(ref ui),
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

    private Element ElementsPage(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Elements",
                    "Links, keyboard shortcuts, avatars, and icons."
                ),
                ui.HStack(
                        ui.Link("docs")
                            .Href("https://gpui-kit.com")
                            .Add(ui.Text("Documentation")),
                        ui.Link("disabled")
                            .Href("https://example.com")
                            .Disabled()
                            .Add(ui.Text("Disabled link"))
                    )
                    .Gap(16)
                    .ItemsCenter(),
                ui.HStack(
                        ui.Kbd("ctrl-k"),
                        ui.Kbd("cmd-shift-p").Outline(),
                        ui.Kbd("alt-enter").Appearance(false)
                    )
                    .Gap(8)
                    .ItemsCenter(),
                ui.HStack(
                        ui.Avatar().Name("Ada Lovelace"),
                        ui.Avatar().Name("Grace Hopper").Size(ControlSize.Large),
                        ui.Icon("icons/check.svg").Size(ControlSize.Medium)
                    )
                    .Gap(12)
                    .ItemsCenter()
            )
            .Gap(16);

    private Element InteractivePage(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Interactive",
                    "Collapsible, pagination, rating, and clipboard."
                ),
                ui.Collapsible()
                    .Open()
                    .Add(ui.Button("collapse").Label("Toggle details").Secondary())
                    .Content(ui.Text("Revealed content in the `content` slot.")),
                ui.Pagination("pages")
                    .TotalPages(10)
                    .CurrentPage(_page + 1)
                    .VisiblePages(5)
                    .OnChange(page => _page = page - 1),
                ui.HStack(ui.Label("Rating"), ui.Rating("quality").Max(5).Value(_rating).OnChange(value => _rating = value))
                    .Gap(8)
                    .ItemsCenter(),
                ui.HStack(ui.Label("Copy id"), ui.Clipboard("copy").Value("gpui-net-shell").Tooltip("Copy to clipboard"))
                    .Gap(8)
                    .ItemsCenter()
            )
            .Gap(16);

    private Element DisplayPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Display", "Breadcrumbs, group boxes, status bars, and alerts."),
                ui.Breadcrumb("Home", "Settings", "Profile"),
                ui.GroupBox()
                    .Title("Options")
                    .Variant(GroupBoxVariant.Outline)
                    .Add(ui.Text("Grouped content inside a titled container.")),
                ui.StatusBar()
                    .LeftContent(ui.Label("Ready"))
                    .RightContent(ui.Label("v0.1.0"))
                    .Add(ui.Text("Three-region status bar")),
                ui.InfoAlert("info", "A short informational message.").Title("Heads up"),
                ui.SuccessAlert("ok", "Everything completed successfully."),
                ui.WarningAlert("warn", "Check your connection.").Banner(),
                ui.ErrorAlert("err", "Something went wrong.")
            )
            .Gap(16);

    private Element MenusPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Menus", "Tooltips, hover cards, and dropdown menus."),
                ui.HStack(
                        ui.Tooltip("tip", "Save", "Saves the current document"),
                        ui.HoverCard("hover")
                            .TriggerElement(ui.Button("hover-trigger").Label("Hover me").Secondary())
                            .Content(
                                ui.VStack(ui.Label("Hover card"), ui.Text("Shown on hover."))
                                    .Gap(4)
                                    .P(8)
                            ),
                        ui.DropdownMenu("menu", "Actions")
                            .Item("Copy", () => { })
                            .Item("Paste", () => { })
                            .Item("Delete", () => { }),
                        ui.DropdownButton("split", "Run")
                            .Variant(DropdownVariant.Primary)
                            .MenuItem("Run once", () => { })
                            .MenuItem("Run all", () => { })
                            .OnClick(() => { })
                    )
                    .Gap(16)
                    .ItemsCenter()
            )
            .Gap(16);

    private Element CollectionsPage(ref RenderContext ui) =>
        ui.VStack(
                Section(ref ui, "Collections", "Typed tabs, lists, selects, and tables."),
                ui.TabBar("tabs")
                    .SelectedIndex(_tabIndex)
                    .Variant(TabVariantKind.Pill)
                    .OnChange(index => _tabIndex = index)
                    .Add(
                        ui.Tab().Label("One"),
                        ui.Tab().Label("Two"),
                        ui.Tab().Label("Three").Disabled()
                    ),
                ui.List("people", ListRows).Full().H(120),
                ui.Select("theme", SelectRows, value => _selected = value).Placeholder("Pick one"),
                ui.Label($"Selected: {_selected}"),
                ui.DataTable("roles", TableRows)
                    .Columns("Name", "Role", "Status")
                    .Stripe()
                    .H(220)
                    .RenderCell(
                        (ctx, args) =>
                        {
                            var cells = args[0].Split('\t');
                            var value = args[1] switch
                            {
                                "Name" => cells.ElementAtOrDefault(0) ?? "",
                                "Role" => cells.ElementAtOrDefault(1) ?? "",
                                "Status" => cells.ElementAtOrDefault(2) ?? "",
                                _ => "",
                            };
                            Element cell =
                                args[1] == "Status"
                                    ? ctx.Tag()
                                        .Variant(
                                            value == "Active"
                                                ? TagVariant.Success
                                                : TagVariant.Warning
                                        )
                                        .Add(ctx.Text(value))
                                    : ctx.Label(value);
                            return cell;
                        }
                    )
            )
            .Gap(16);

    private static string ListRows() =>
        "ada\tAda Lovelace\ngrace\tGrace Hopper\nlinus\tLinus Torvalds\ttrue";

    private static string SelectRows() => "light\tLight\ndark\tDark\nsystem\tSystem";

    private static string TableRows() =>
        "Ada\tEngineer\tActive\nGrace\tAdmiral\tActive\nLinus\tMaintainer\tAway";

    private Element DisclosurePage(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Disclosure",
                    "Accordion and stepper built from typed children."
                ),
                ui.Accordion("faq")
                    .Multiple()
                    .Add(
                        ui.AccordionItem()
                            .Title(ui.Label("What is gpui-net-shell?"))
                            .Open()
                            .Add(ui.Text("A C#-hosted shell over a Rust GPUI runtime.")),
                        ui.AccordionItem()
                            .Title(ui.Label("How do the samples work?"))
                            .Add(ui.Text("Each tab is a managed page rendered through the ABI."))
                    ),
                ui.Stepper("steps")
                    .SelectedIndex(_step)
                    .OnChange(index => _step = index)
                    .Add(
                        ui.StepperItem().Add(ui.Label("Account")),
                        ui.StepperItem().Add(ui.Label("Profile")),
                        ui.StepperItem().Add(ui.Label("Review"))
                    )
            )
            .Gap(16);

    private Element StructurePage(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Structure",
                    "Description lists and forms built from typed children."
                ),
                ui.DescriptionList()
                    .Columns(2)
                    .Add(
                        ui.DescriptionItem("Name").Value("Ada Lovelace"),
                        ui.DescriptionItem("Role").Value("Engineer"),
                        ui.DescriptionItem("Bio").Value("Mathematician").Span(2)
                    ),
                ui.HForm()
                    .Columns(2)
                    .LabelWidth(90)
                    .Add(
                        ui.Field().Label("Name").Add(ui.Text("Ada Lovelace")),
                        ui.Field().Label("Role").Required().Add(ui.Text("Engineer")),
                        ui.Field()
                            .Description("Optional contact email.")
                            .ColSpan(2)
                            .Add(ui.Text("ada@example.com"))
                    )
            )
            .Gap(16);

    private Element InputMonitorPage(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Input Monitor",
                    "Mouse, wheel, and keyboard events forwarded to managed code."
                ),
                ui.Label("Click, scroll, or type anywhere; the last event is shown below."),
                ui.Label($"Last: {_lastInput}").TextSize(16).FontMedium(),
                ui.Label($"Events: {_inputCount}   Moves: {_moveCount}"),
                ui.Input("monitor-input")
                    .Placeholder("Type here…")
                    .OnChange(text =>
                    {
                        _typed = text;
                        Invalidate();
                    }),
                ui.Label($"Typed: {_typed}")
            )
            .Gap(12);

    private Element TextInputPage(ref RenderContext ui) =>
        ui.VStack(
                Section(
                    ref ui,
                    "Text Input",
                    "Number input, textarea, one-time password, and slider."
                ),
                ui.NumberInput("num").Placeholder("Amount").OnChange(value =>
                {
                    _number = value;
                    Invalidate();
                }),
                ui.Label($"Number: {_number}"),
                ui.Textarea("notes").Placeholder("Notes…").OnChange(value =>
                {
                    _textarea = value;
                    Invalidate();
                }),
                ui.Label($"Textarea: {_textarea.Length} chars"),
                ui.OtpInput("otp").Length(6).Groups(2).OnChange(value =>
                {
                    _otp = value;
                    Invalidate();
                }),
                ui.Label($"OTP: {_otp}"),
                ui.Slider("vol").Min(0).Max(100).Value(_slider).OnChange(value =>
                {
                    _slider = value;
                    Invalidate();
                }),
                ui.Label($"Slider: {_slider:0.#}")
            )
            .Gap(12);

    private Element Section(ref RenderContext ui, string title, string description) =>
        ui.VStack(ui.Label(title).TextSize(20).FontSemibold(), ui.Text(description)).Gap(4);
}
