namespace GpuiNetShell.Sample.Pages;

/// <summary>
/// The ordered list of gallery pages, one per control. The navigation sidebar
/// shows these in order.
/// </summary>
internal static class PageRegistry
{
    public static IReadOnlyList<GalleryPage> Create() =>
        [
            new TextPage(),
            new LabelPage(),
            new ButtonPage(),
            new BadgePage(),
            new TagPage(),
            new ProgressPage(),
            new SpinnerPage(),
            new SeparatorPage(),
            new SkeletonPage(),
            new LinkPage(),
            new KbdPage(),
            new AvatarPage(),
            new IconPage(),
            new CollapsiblePage(),
            new PaginationPage(),
            new RatingPage(),
            new ClipboardPage(),
            new BreadcrumbPage(),
            new GroupBoxPage(),
            new StatusBarPage(),
            new AlertPage(),
            new TooltipPage(),
            new HoverCardPage(),
            new DropdownMenuPage(),
            new DropdownButtonPage(),
            new MenuBarPage(),
            new TabBarPage(),
            new AccordionPage(),
            new StepperPage(),
            new DescriptionListPage(),
            new FormPage(),
            new InputPage(),
            new NumberInputPage(),
            new TextareaPage(),
            new OtpInputPage(),
            new SliderPage(),
            new ColorPickerPage(),
            new CalendarPage(),
            new DatePickerPage(),
            new RadioPage(),
            new ListPage(),
            new SelectPage(),
            new DataTablePage(),
            new SidebarPage(),
            new SettingsPage(),
            new TreePage(),
            new TablePage(),
            new CommandPage(),
            new PopoverPage(),
            new OverlaysPage(),
        ];
}
