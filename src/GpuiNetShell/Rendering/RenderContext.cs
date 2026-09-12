using GpuiNetShell.Elements;
using GpuiNetShell.Events;
using GpuiNetShell.Interop;

namespace GpuiNetShell.Rendering;

/// <summary>
/// The element declaration surface passed to <see cref="View.Render"/>. Every
/// factory records into the arena; nothing renders here.
/// </summary>
public sealed class RenderContext
{
    private readonly RenderArena _arena;
    private readonly EventRegistry _events;
    private readonly Action _notify;
    private bool _rendering;

    internal RenderContext(RenderArena arena, EventRegistry events, Action notify)
    {
        _arena = arena;
        _events = events;
        _notify = notify;
    }

    internal RenderArena Arena => _arena;

    internal EventRegistry Events => _events;

    /// <summary>
    /// Requests a native re-render, the managed equivalent of gpui's
    /// <c>cx.notify()</c>. Call it from an event or task after changing state
    /// that <see cref="View.Render"/> reads.
    /// </summary>
    /// <exception cref="InvalidOperationException">
    /// Thrown when called during <see cref="View.Render"/>, where requesting
    /// another render would loop, matching gpui's own rule.
    /// </exception>
    public void Notify()
    {
        if (_rendering)
        {
            throw new InvalidOperationException(
                "Cannot notify during Render(); change state from an event or task instead."
            );
        }
        _notify();
    }

    /// <summary>Marks that managed rendering has begun, so <see cref="Notify"/> is refused.</summary>
    internal void BeginRender() => _rendering = true;

    /// <summary>Marks that managed rendering has ended.</summary>
    internal void EndRender() => _rendering = false;

    /// <summary>Declares a button. <paramref name="id"/> is its stable identity.</summary>
    public ButtonElement Button(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentButton);
        _arena.SetNodeData(index, id);
        return new ButtonElement(this, index);
    }

    /// <summary>Declares a run of text.</summary>
    public TextElement Text(string content)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentText);
        _arena.SetNodeData(index, content);
        return new TextElement(this, index);
    }

    /// <summary>Declares a container with the given children.</summary>
    public DivElement Div(params Element[] children) =>
        new DivElement(this, _arena.AddNode(NativeProtocol.ComponentDiv)).Add(children);

    /// <summary>Declares a styled label.</summary>
    public LabelElement Label(string value)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentLabel);
        _arena.SetNodeData(index, value);
        return new LabelElement(this, index);
    }

    /// <summary>Declares a count or dot badge configured through methods.</summary>
    public BadgeElement Badge() =>
        new BadgeElement(this, _arena.AddNode(NativeProtocol.ComponentBadge));

    /// <summary>Declares a progress bar. <paramref name="id"/> is its stable identity.</summary>
    public ProgressElement Progress(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentProgress);
        _arena.SetNodeData(index, id);
        return new ProgressElement(this, index);
    }

    /// <summary>Declares a combobox. <paramref name="id"/> is its stable identity.</summary>
    public ComboboxElement Combobox(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentCombobox);
        _arena.SetNodeData(index, id);
        return new ComboboxElement(this, index);
    }

    /// <summary>Declares a radio option. <paramref name="id"/> is its stable identity.</summary>
    public RadioElement Radio(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentRadio);
        _arena.SetNodeData(index, id);
        return new RadioElement(this, index);
    }

    /// <summary>Declares a tab bar. <paramref name="id"/> is its stable identity.</summary>
    public TabsElement Tabs(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentTabs);
        _arena.SetNodeData(index, id);
        return new TabsElement(this, index);
    }

    /// <summary>Declares a scrollable area. <paramref name="id"/> is its stable identity.</summary>
    public ScrollElement Scroll(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentScroll);
        _arena.SetNodeData(index, id);
        return new ScrollElement(this, index);
    }

    /// <summary>Declares a scrollbar that drives the area named <paramref name="target"/>.</summary>
    public ScrollbarElement Scrollbar(string target)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentScrollbar);
        _arena.SetNodeData(index, target);
        return new ScrollbarElement(this, index);
    }

    /// <summary>Declares resizable panels. <paramref name="id"/> is its stable identity.</summary>
    public ResizableElement Resizable(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentResizable);
        _arena.SetNodeData(index, id);
        return new ResizableElement(this, index);
    }

    /// <summary>
    /// Declares an anchored popover. <paramref name="id"/> is its stable
    /// identity; <paramref name="label"/> is the trigger's label.
    /// </summary>
    public PopoverElement Popover(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentPopover);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + label);
        return new PopoverElement(this, index);
    }

    /// <summary>Declares a cycling loading spinner.</summary>
    public SpinnerElement Spinner() =>
        new SpinnerElement(this, _arena.AddNode(NativeProtocol.ComponentSpinner));

    /// <summary>Declares a horizontal separator.</summary>
    public SeparatorElement Separator() => Separator("Separator");

    /// <summary>Declares a vertical separator.</summary>
    public SeparatorElement VerticalSeparator() => Separator("VerticalSeparator");

    /// <summary>Declares a dashed horizontal separator.</summary>
    public SeparatorElement DashedSeparator() => Separator("DashedSeparator");

    /// <summary>Declares a dashed vertical separator.</summary>
    public SeparatorElement VerticalDashedSeparator() => Separator("VerticalDashedSeparator");

    /// <summary>Declares an animated loading placeholder.</summary>
    public SkeletonElement Skeleton() =>
        new SkeletonElement(this, _arena.AddNode(NativeProtocol.ComponentSkeleton));

    /// <summary>Declares a compact semantic status tag.</summary>
    public TagElement Tag() =>
        new TagElement(this, _arena.AddNode(NativeProtocol.ComponentTag));

    private SeparatorElement Separator(string export)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSeparator);
        _arena.SetNodeData(index, export);
        return new SeparatorElement(this, index);
    }

    /// <summary>Declares an external-resource link. <paramref name="id"/> is its identity.</summary>
    public LinkElement Link(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentLink);
        _arena.SetNodeData(index, id);
        return new LinkElement(this, index);
    }

    /// <summary>Declares a keyboard shortcut keycap from a keystroke string.</summary>
    public KbdElement Kbd(string keystroke)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentKbd);
        _arena.SetNodeData(index, keystroke);
        return new KbdElement(this, index);
    }

    /// <summary>Declares a circular avatar with a name-derived fallback.</summary>
    public AvatarElement Avatar() =>
        new AvatarElement(this, _arena.AddNode(NativeProtocol.ComponentAvatar));

    /// <summary>Declares an SVG icon from a relative asset path.</summary>
    public IconElement Icon(string path)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentIcon);
        _arena.SetNodeData(index, path);
        return new IconElement(this, index);
    }

    /// <summary>Declares a trigger container with an optional `content` reveal.</summary>
    public CollapsibleElement Collapsible() =>
        new CollapsibleElement(this, _arena.AddNode(NativeProtocol.ComponentCollapsible));

    /// <summary>Declares controlled page navigation.</summary>
    public PaginationElement Pagination(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentPagination);
        _arena.SetNodeData(index, id);
        return new PaginationElement(this, index);
    }

    /// <summary>Declares an interactive star rating.</summary>
    public RatingElement Rating(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentRating);
        _arena.SetNodeData(index, id);
        return new RatingElement(this, index);
    }

    /// <summary>Declares a clipboard copy button.</summary>
    public ClipboardElement Clipboard(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentClipboard);
        _arena.SetNodeData(index, id);
        return new ClipboardElement(this, index);
    }

    /// <summary>Declares a navigation trail from an ordered list of labels.</summary>
    public BreadcrumbElement Breadcrumb(params string[] labels)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentBreadcrumb);
        _arena.SetNodeData(index, string.Join('\n', labels ?? []));
        return new BreadcrumbElement(this, index);
    }

    /// <summary>Declares a titled container for grouping related content.</summary>
    public GroupBoxElement GroupBox() =>
        new GroupBoxElement(this, _arena.AddNode(NativeProtocol.ComponentGroupBox));

    /// <summary>Declares a three-region status bar.</summary>
    public StatusBarElement StatusBar() =>
        new StatusBarElement(this, _arena.AddNode(NativeProtocol.ComponentStatusBar));

    /// <summary>Declares a default message banner.</summary>
    public AlertElement Alert(string id, string message) => Alert("Alert", id, message);

    /// <summary>Declares an informational message banner.</summary>
    public AlertElement InfoAlert(string id, string message) => Alert("InfoAlert", id, message);

    /// <summary>Declares a success message banner.</summary>
    public AlertElement SuccessAlert(string id, string message) => Alert("SuccessAlert", id, message);

    /// <summary>Declares a warning message banner.</summary>
    public AlertElement WarningAlert(string id, string message) => Alert("WarningAlert", id, message);

    /// <summary>Declares an error message banner.</summary>
    public AlertElement ErrorAlert(string id, string message) => Alert("ErrorAlert", id, message);

    private AlertElement Alert(string export, string id, string message)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentAlert);
        _arena.SetNodeData(
            index,
            export
                + NativeProtocol.ConstructorArgSeparator
                + id
                + NativeProtocol.ConstructorArgSeparator
                + message
        );
        return new AlertElement(this, index);
    }

    /// <summary>Declares a button trigger with a text tooltip.</summary>
    public TooltipElement Tooltip(string id, string label, string text)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentTooltip);
        _arena.SetNodeData(
            index,
            id
                + NativeProtocol.ConstructorArgSeparator
                + label
                + NativeProtocol.ConstructorArgSeparator
                + text
        );
        return new TooltipElement(this, index);
    }

    /// <summary>Declares a hover-triggered card.</summary>
    public HoverCardElement HoverCard(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentHoverCard);
        _arena.SetNodeData(index, id);
        return new HoverCardElement(this, index);
    }

    /// <summary>Declares a button-triggered popup menu.</summary>
    public DropdownMenuElement DropdownMenu(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentDropdownMenu);
        _arena.SetNodeData(
            index,
            id + NativeProtocol.ConstructorArgSeparator + label
        );
        return new DropdownMenuElement(this, index);
    }

    /// <summary>Declares a split dropdown button.</summary>
    public DropdownButtonElement DropdownButton(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentDropdownButton);
        _arena.SetNodeData(
            index,
            id + NativeProtocol.ConstructorArgSeparator + label
        );
        return new DropdownButtonElement(this, index);
    }

    /// <summary>Declares a typed tab list. <paramref name="id"/> is its identity.</summary>
    public TabBarElement TabBar(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentTabBar);
        _arena.SetNodeData(index, id);
        return new TabBarElement(this, index);
    }

    /// <summary>Declares a tab for a <see cref="TabBar"/>.</summary>
    public TabElement Tab() => new TabElement(this, _arena.AddNode(NativeProtocol.ComponentTab));

    /// <summary>
    /// Declares a retained list. <paramref name="rows"/> returns newline-
    /// separated rows of tab-separated `id`, `label`, and optional `disabled`.
    /// </summary>
    public ListElement List(string id, Func<string> rows)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentList);
        var token = Events.RegisterRows(rows);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + token);
        return new ListElement(this, index);
    }

    /// <summary>
    /// Declares a retained select. <paramref name="rows"/> returns
    /// newline-separated `id\tlabel[\tdisabled]` rows; <paramref name="onSelect"/>
    /// receives the selected row id.
    /// </summary>
    public SelectElement Select(string id, Func<string> rows, Action<string> onSelect)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSelect);
        var rowsToken = Events.RegisterRows(rows);
        var selectToken = Events.Register(value => onSelect(value.String ?? string.Empty));
        _arena.SetNodeData(
            index,
            id
                + NativeProtocol.ConstructorArgSeparator
                + rowsToken
                + NativeProtocol.ConstructorArgSeparator
                + selectToken
        );
        return new SelectElement(this, index);
    }

    /// <summary>
    /// Declares a retained table. <paramref name="rows"/> returns newline-
    /// separated rows of tab-separated cell strings.
    /// </summary>
    public DataTableElement DataTable(string id, Func<string> rows)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentDataTable);
        var token = Events.RegisterRows(rows);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + token);
        return new DataTableElement(this, index);
    }

    /// <summary>Declares a typed accordion. <paramref name="id"/> is its identity.</summary>
    public AccordionElement Accordion(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentAccordion);
        _arena.SetNodeData(index, id);
        return new AccordionElement(this, index);
    }

    /// <summary>Declares an accordion item for an <see cref="AccordionElement"/>.</summary>
    public AccordionItemElement AccordionItem() =>
        new AccordionItemElement(this, _arena.AddNode(NativeProtocol.ComponentAccordionItem));

    /// <summary>Declares a typed stepper. <paramref name="id"/> is its identity.</summary>
    public StepperElement Stepper(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentStepper);
        _arena.SetNodeData(index, id);
        return new StepperElement(this, index);
    }

    /// <summary>Declares a step for a <see cref="StepperElement"/>.</summary>
    public StepperItemElement StepperItem() =>
        new StepperItemElement(this, _arena.AddNode(NativeProtocol.ComponentStepperItem));

    /// <summary>Declares a description item with the given label.</summary>
    public DescriptionItemElement DescriptionItem(string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentDescriptionItem);
        _arena.SetNodeData(index, label);
        return new DescriptionItemElement(this, index);
    }

    /// <summary>Declares a structured label/value list.</summary>
    public DescriptionListElement DescriptionList() =>
        new DescriptionListElement(
            this,
            _arena.AddNode(NativeProtocol.ComponentDescriptionList)
        );

    /// <summary>Declares a form field.</summary>
    public FieldElement Field() =>
        new FieldElement(this, _arena.AddNode(NativeProtocol.ComponentField));

    /// <summary>Declares a vertical form.</summary>
    public FormElement Form() => Form("Form");

    /// <summary>Declares a vertical form.</summary>
    public FormElement VForm() => Form("VForm");

    /// <summary>Declares a horizontal form.</summary>
    public FormElement HForm() => Form("HForm");

    private FormElement Form(string export)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentForm);
        _arena.SetNodeData(index, export);
        return new FormElement(this, index);
    }

    /// <summary>Declares a retained single-line text field.</summary>
    public InputElement Input(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentInput);
        _arena.SetNodeData(index, id);
        return new InputElement(this, index);
    }

    /// <summary>Declares a retained numeric text field.</summary>
    public NumberInputElement NumberInput(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentNumberInput);
        _arena.SetNodeData(index, id);
        return new NumberInputElement(this, index);
    }

    /// <summary>Declares a retained multi-line text field.</summary>
    public TextareaElement Textarea(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentTextarea);
        _arena.SetNodeData(index, id);
        return new TextareaElement(this, index);
    }

    /// <summary>Declares a retained one-time-password field.</summary>
    public OtpInputElement OtpInput(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentOtpInput);
        _arena.SetNodeData(index, id);
        return new OtpInputElement(this, index);
    }

    /// <summary>Declares a retained numeric slider.</summary>
    public SliderElement Slider(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSlider);
        _arena.SetNodeData(index, id);
        return new SliderElement(this, index);
    }

    /// <summary>Declares a retained color picker.</summary>
    public ColorPickerElement ColorPicker(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentColorPicker);
        _arena.SetNodeData(index, id);
        return new ColorPickerElement(this, index);
    }

    /// <summary>Declares a retained calendar.</summary>
    public CalendarElement Calendar(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentCalendar);
        _arena.SetNodeData(index, id);
        return new CalendarElement(this, index);
    }

    /// <summary>Declares a retained single-date picker.</summary>
    public DatePickerElement DatePicker(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentDatePicker);
        _arena.SetNodeData(index, id);
        return new DatePickerElement(this, index);
    }

    /// <summary>Declares a menu item for a <see cref="MenuElement"/>.</summary>
    public MenuItemElement MenuItem(string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentMenuItem);
        _arena.SetNodeData(index, label);
        return new MenuItemElement(this, index);
    }

    /// <summary>Declares a menu separator for a <see cref="MenuElement"/>.</summary>
    public MenuSeparatorElement MenuSeparator() =>
        new MenuSeparatorElement(this, _arena.AddNode(NativeProtocol.ComponentMenuSeparator));

    /// <summary>Declares a top-level menu for a <see cref="MenuBarElement"/>.</summary>
    public MenuElement Menu(string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentMenu);
        _arena.SetNodeData(index, label);
        return new MenuElement(this, index);
    }

    /// <summary>Declares an in-window menu bar.</summary>
    public MenuBarElement MenuBar(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentMenuBar);
        _arena.SetNodeData(index, id);
        return new MenuBarElement(this, index);
    }

    /// <summary>Declares a sidebar navigation row.</summary>
    public SidebarMenuItemElement SidebarMenuItem(string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSidebarMenuItem);
        _arena.SetNodeData(index, label);
        return new SidebarMenuItemElement(this, index);
    }

    /// <summary>Declares a typed sidebar menu.</summary>
    public SidebarMenuElement SidebarMenu() =>
        new SidebarMenuElement(this, _arena.AddNode(NativeProtocol.ComponentSidebarMenu));

    /// <summary>Declares a sidebar header.</summary>
    public SidebarHeaderElement SidebarHeader() =>
        new SidebarHeaderElement(this, _arena.AddNode(NativeProtocol.ComponentSidebarHeader));

    /// <summary>Declares a sidebar footer.</summary>
    public SidebarFooterElement SidebarFooter() =>
        new SidebarFooterElement(this, _arena.AddNode(NativeProtocol.ComponentSidebarFooter));

    /// <summary>Declares an application sidebar.</summary>
    public SidebarElement Sidebar(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSidebar);
        _arena.SetNodeData(index, id);
        return new SidebarElement(this, index);
    }

    /// <summary>Declares a sidebar toggle button.</summary>
    public SidebarToggleButtonElement SidebarToggleButton() =>
        new SidebarToggleButtonElement(
            this,
            _arena.AddNode(NativeProtocol.ComponentSidebarToggleButton)
        );

    /// <summary>Declares a setting item.</summary>
    public SettingItemElement SettingItem(string title)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSettingItem);
        _arena.SetNodeData(index, title);
        return new SettingItemElement(this, index);
    }

    /// <summary>Declares a setting group.</summary>
    public SettingGroupElement SettingGroup() =>
        new SettingGroupElement(this, _arena.AddNode(NativeProtocol.ComponentSettingGroup));

    /// <summary>Declares a setting page.</summary>
    public SettingPageElement SettingPage(string title)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSettingPage);
        _arena.SetNodeData(index, title);
        return new SettingPageElement(this, index);
    }

    /// <summary>Declares a settings surface.</summary>
    public SettingsElement Settings(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSettings);
        _arena.SetNodeData(index, id);
        return new SettingsElement(this, index);
    }

    /// <summary>Declares a tree item with a unique id and label.</summary>
    public TreeItemElement TreeItem(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentTreeItem);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + label);
        return new TreeItemElement(this, index);
    }

    /// <summary>Declares a retained tree.</summary>
    public TreeElement Tree(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentTree);
        _arena.SetNodeData(index, id);
        return new TreeElement(this, index);
    }

    /// <summary>Declares a simple table.</summary>
    public TableElement Table() => new TableElement(this, _arena.AddNode(NativeProtocol.ComponentTable));

    /// <summary>Declares a table header.</summary>
    public TableHeaderElement TableHeader() =>
        new TableHeaderElement(this, _arena.AddNode(NativeProtocol.ComponentTableHeader));

    /// <summary>Declares a table body.</summary>
    public TableBodyElement TableBody() =>
        new TableBodyElement(this, _arena.AddNode(NativeProtocol.ComponentTableBody));

    /// <summary>Declares a table footer.</summary>
    public TableFooterElement TableFooter() =>
        new TableFooterElement(this, _arena.AddNode(NativeProtocol.ComponentTableFooter));

    /// <summary>Declares a table row.</summary>
    public TableRowElement TableRow() =>
        new TableRowElement(this, _arena.AddNode(NativeProtocol.ComponentTableRow));

    /// <summary>Declares a table header cell.</summary>
    public TableHeadElement TableHead() =>
        new TableHeadElement(this, _arena.AddNode(NativeProtocol.ComponentTableHead));

    /// <summary>Declares a table data cell.</summary>
    public TableCellElement TableCell() =>
        new TableCellElement(this, _arena.AddNode(NativeProtocol.ComponentTableCell));

    /// <summary>Declares a table caption.</summary>
    public TableCaptionElement TableCaption() =>
        new TableCaptionElement(this, _arena.AddNode(NativeProtocol.ComponentTableCaption));

    /// <summary>Declares a Command palette item.</summary>
    public CommandItemElement CommandItem(string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentCommandItem);
        _arena.SetNodeData(index, label);
        return new CommandItemElement(this, index);
    }

    /// <summary>Declares a Command palette group.</summary>
    public CommandGroupElement CommandGroup(string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentCommandGroup);
        _arena.SetNodeData(index, label);
        return new CommandGroupElement(this, index);
    }

    /// <summary>Declares a Command palette separator.</summary>
    public CommandSeparatorElement CommandSeparator() =>
        new CommandSeparatorElement(this, _arena.AddNode(NativeProtocol.ComponentCommandSeparator));

    /// <summary>Declares a retained native Command palette.</summary>
    public CommandElement Command(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentCommand);
        _arena.SetNodeData(index, id);
        return new CommandElement(this, index);
    }

    /// <summary>Declares a file or image attachment.</summary>
    public AttachmentElement Attachment(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentAttachment);
        _arena.SetNodeData(index, id);
        return new AttachmentElement(this, index);
    }

    /// <summary>Declares a message bubble.</summary>
    public BubbleElement Bubble() =>
        new BubbleElement(this, _arena.AddNode(NativeProtocol.ComponentBubble));

    /// <summary>Declares a conversation status marker.</summary>
    public MarkerElement Marker(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentMarker);
        _arena.SetNodeData(index, id);
        return new MarkerElement(this, index);
    }

    /// <summary>Declares a message row.</summary>
    public MessageElement Message() =>
        new MessageElement(this, _arena.AddNode(NativeProtocol.ComponentMessage));

    /// <summary>Declares animated loading text.</summary>
    public ShimmerTextElement ShimmerText(string text)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentShimmerText);
        _arena.SetNodeData(index, text);
        return new ShimmerTextElement(this, index);
    }

    /// <summary>Declares a virtualized message transcript.</summary>
    public MessageScrollerElement MessageScroller(string id, int itemCount)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentMessageScroller);
        _arena.SetNodeData(
            index,
            id
                + NativeProtocol.ConstructorArgSeparator
                + itemCount.ToString(System.Globalization.CultureInfo.InvariantCulture)
        );
        return new MessageScrollerElement(this, index);
    }

    /// <summary>Declares a vertical controlled radio set.</summary>
    public RadioGroupElement RadioGroup(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentRadioGroup);
        _arena.SetNodeData(
            index,
            "RadioGroup" + NativeProtocol.ConstructorArgSeparator + id
        );
        return new RadioGroupElement(this, index);
    }

    /// <summary>Declares a horizontal controlled radio set.</summary>
    public RadioGroupElement HorizontalRadioGroup(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentRadioGroup);
        _arena.SetNodeData(
            index,
            "HorizontalRadioGroup" + NativeProtocol.ConstructorArgSeparator + id
        );
        return new RadioGroupElement(this, index);
    }

    /// <summary>Declares a button that opens a native dialog.</summary>
    public DialogElement Dialog(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentDialog);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + label);
        return new DialogElement(this, index);
    }

    /// <summary>Declares a button that opens a native alert dialog.</summary>
    public AlertDialogElement AlertDialog(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentAlertDialog);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + label);
        return new AlertDialogElement(this, index);
    }

    /// <summary>Declares a button that opens a native sheet.</summary>
    public SheetElement Sheet(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentSheet);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + label);
        return new SheetElement(this, index);
    }

    /// <summary>Declares a button that posts a native notification.</summary>
    public NotificationElement Notification(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentNotification);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + label);
        return new NotificationElement(this, index);
    }

    /// <summary>Declares a retained native source editor.</summary>
    public EditorElement Editor(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentEditor);
        _arena.SetNodeData(index, id);
        return new EditorElement(this, index);
    }

    /// <summary>Declares a typed native-menu item.</summary>
    public NativeMenuItemElement NativeMenuItem(string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentNativeMenuItem);
        _arena.SetNodeData(index, label);
        return new NativeMenuItemElement(this, index);
    }

    /// <summary>Declares a typed native-menu separator.</summary>
    public NativeMenuSeparatorElement NativeMenuSeparator() =>
        new NativeMenuSeparatorElement(
            this,
            _arena.AddNode(NativeProtocol.ComponentNativeMenuSeparator)
        );

    /// <summary>Declares a button that shows an OS native menu.</summary>
    public NativeMenuTriggerElement NativeMenuTrigger(string id, string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentNativeMenuTrigger);
        _arena.SetNodeData(index, id + NativeProtocol.ConstructorArgSeparator + label);
        return new NativeMenuTriggerElement(this, index);
    }

    /// <summary>Declares a typed context-menu item.</summary>
    public ContextMenuItemElement ContextMenuItem(string label)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentContextMenuItem);
        _arena.SetNodeData(index, label);
        return new ContextMenuItemElement(this, index);
    }

    /// <summary>Declares a typed context-menu separator.</summary>
    public ContextMenuSeparatorElement ContextMenuSeparator() =>
        new ContextMenuSeparatorElement(
            this,
            _arena.AddNode(NativeProtocol.ComponentContextMenuSeparator)
        );

    /// <summary>Attaches a right-click menu to its target children.</summary>
    public ContextMenuElement ContextMenu(string id)
    {
        var index = _arena.AddNode(NativeProtocol.ComponentContextMenu);
        _arena.SetNodeData(index, id);
        return new ContextMenuElement(this, index);
    }

    /// <summary>A column container.</summary>
    public DivElement VStack(params Element[] children) => Div(children).Flex().FlexColumn();

    /// <summary>A row container.</summary>
    public DivElement HStack(params Element[] children) => Div(children).Flex().FlexRow();
}
