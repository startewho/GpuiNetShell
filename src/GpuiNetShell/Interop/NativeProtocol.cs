namespace GpuiNetShell.Interop;

/// <summary>
/// The wire vocabulary shared with the native host. Every constant here mirrors
/// <c>crates/gpui-net-shell/src/schema.rs</c>. <see cref="SchemaHash"/> is the
/// value both sides must agree on; a test pins its literal on each side.
/// </summary>
/// <remarks>
/// The operation set mirrors <c>gpui-shell</c>'s <c>SpecOp</c>. Styling is not
/// enumerated per property: a style call carries a GPUI method name and an
/// argument that the native host resolves against GPUI's reflected style table.
/// Component behavior is a generic <see cref="OpMethod"/>, and event bindings
/// are a generic <see cref="OpCallback"/>.
/// </remarks>
public static class NativeProtocol
{
    public const uint AbiVersion = 6;

    /// <summary>Identifies the component/operation vocabulary below.</summary>
    public const ulong SchemaHash = 0x6E65_7473_6865_6C53;

    /// <summary>
    /// Separates the string arguments of a multi-argument constructor inside one
    /// node's identity data, mirroring <c>schema.rs</c>.
    /// </summary>
    public const char ConstructorArgSeparator = '\u001F';

    // Components. Ids are registry indices: the native host resolves them
    // against the registered component catalog.
    public const uint ComponentDiv = 0;
    public const uint ComponentText = 1;
    public const uint ComponentButton = 2;
    public const uint ComponentLabel = 3;
    public const uint ComponentBadge = 4;
    public const uint ComponentProgress = 5;
    public const uint ComponentCombobox = 6;
    public const uint ComponentRadio = 7;
    public const uint ComponentTabs = 8;
    public const uint ComponentScroll = 9;
    public const uint ComponentScrollbar = 10;
    public const uint ComponentResizable = 11;
    public const uint ComponentPopover = 12;
    public const uint ComponentSpinner = 13;
    public const uint ComponentSeparator = 14;
    public const uint ComponentSkeleton = 15;
    public const uint ComponentTag = 16;
    public const uint ComponentLink = 17;
    public const uint ComponentKbd = 18;
    public const uint ComponentAvatar = 19;
    public const uint ComponentIcon = 20;
    public const uint ComponentCollapsible = 21;
    public const uint ComponentPagination = 22;
    public const uint ComponentRating = 23;
    public const uint ComponentClipboard = 24;
    public const uint ComponentBreadcrumb = 25;
    public const uint ComponentGroupBox = 26;
    public const uint ComponentStatusBar = 27;
    public const uint ComponentAlert = 28;
    public const uint ComponentTooltip = 29;
    public const uint ComponentHoverCard = 30;
    public const uint ComponentDropdownMenu = 31;
    public const uint ComponentDropdownButton = 32;
    public const uint ComponentTab = 33;
    public const uint ComponentTabBar = 34;
    public const uint ComponentList = 35;
    public const uint ComponentSelect = 36;
    public const uint ComponentDataTable = 37;
    public const uint ComponentAccordionItem = 38;
    public const uint ComponentAccordion = 39;
    public const uint ComponentStepperItem = 40;
    public const uint ComponentStepper = 41;
    public const uint ComponentDescriptionItem = 42;
    public const uint ComponentDescriptionList = 43;
    public const uint ComponentField = 44;
    public const uint ComponentForm = 45;
    public const uint ComponentInput = 46;
    public const uint ComponentNumberInput = 47;
    public const uint ComponentTextarea = 48;
    public const uint ComponentOtpInput = 49;
    public const uint ComponentSlider = 50;
    public const uint ComponentColorPicker = 51;
    public const uint ComponentCalendar = 52;
    public const uint ComponentDatePicker = 53;
    public const uint ComponentMenuItem = 54;
    public const uint ComponentMenuSeparator = 55;
    public const uint ComponentMenu = 56;
    public const uint ComponentMenuBar = 57;
    public const uint ComponentSidebarMenuItem = 58;
    public const uint ComponentSidebarMenu = 59;
    public const uint ComponentSidebarHeader = 60;
    public const uint ComponentSidebarFooter = 61;
    public const uint ComponentSidebar = 62;
    public const uint ComponentSidebarToggleButton = 63;
    public const uint ComponentSettingItem = 64;
    public const uint ComponentSettingGroup = 65;
    public const uint ComponentSettingPage = 66;
    public const uint ComponentSettings = 67;
    public const uint ComponentTreeItem = 68;
    public const uint ComponentTree = 69;
    public const uint ComponentTableHeader = 70;
    public const uint ComponentTableBody = 71;
    public const uint ComponentTableFooter = 72;
    public const uint ComponentTableRow = 73;
    public const uint ComponentTableHead = 74;
    public const uint ComponentTableCell = 75;
    public const uint ComponentTableCaption = 76;
    public const uint ComponentTable = 77;
    public const uint ComponentCommandItem = 78;
    public const uint ComponentCommandGroup = 79;
    public const uint ComponentCommandSeparator = 80;
    public const uint ComponentCommand = 81;
    public const uint ComponentAttachment = 82;
    public const uint ComponentBubble = 83;
    public const uint ComponentMarker = 84;
    public const uint ComponentMessage = 85;
    public const uint ComponentShimmerText = 86;
    public const uint ComponentMessageScroller = 87;
    public const uint ComponentRadioGroup = 88;
    public const uint ComponentDialog = 89;
    public const uint ComponentAlertDialog = 90;
    public const uint ComponentSheet = 91;
    public const uint ComponentNotification = 92;
    public const uint ComponentEditor = 93;
    public const uint ComponentNativeMenuItem = 94;
    public const uint ComponentNativeMenuSeparator = 95;
    public const uint ComponentNativeMenuTrigger = 96;
    public const uint ComponentContextMenuItem = 97;
    public const uint ComponentContextMenuSeparator = 98;
    public const uint ComponentContextMenu = 99;
    public const uint ComponentVirtualList = 100;
    public const uint ComponentImage = 101;

    // Operations. `a` is the packed UTF-8 range of a method name; `flags`
    // classifies the argument in `b`.
    public const ushort OpNullaryStyle = 1;
    public const ushort OpParamStyle = 2;
    public const ushort OpMethod = 3;
    public const ushort OpCallback = 4;
    /// <summary>A named slot: <c>a</c> is the name, <c>b</c> the child node index.</summary>
    public const ushort OpSlot = 5;

    public const ushort ArgNone = 0;
    public const ushort ArgNumber = 1;
    public const ushort ArgString = 2;
    /// <summary>A closed-set literal for a component method, packed like <see cref="ArgString"/>.</summary>
    public const ushort ArgEnum = 3;
    /// <summary>An element argument: <c>b</c> is the child node index to materialize.</summary>
    public const ushort ArgElement = 4;
    /// <summary>A two-argument method: <c>b</c> is a packed string, <c>c</c> a callback token.</summary>
    public const ushort ArgStringCallback = 5;

    // Input event kinds delivered through the `input_event` callback.
    public const uint InputKeyDown = 0;
    public const uint InputKeyUp = 1;
    public const uint InputMouseDown = 2;
    public const uint InputMouseUp = 3;
    public const uint InputMouseMove = 4;
    public const uint InputScroll = 5;

    // Modifier flags carried in `input_event`'s `flags`.
    public const uint ModifierShift = 1;
    public const uint ModifierControl = 2;
    public const uint ModifierAlt = 4;
    public const uint ModifierPlatform = 8;

    // Callback value kinds delivered through the `invoke` callback.
    public const uint CallbackValueNone = 0;
    public const uint CallbackValueBoolean = 1;
    public const uint CallbackValueNumber = 2;
    public const uint CallbackValueString = 3;

    // Notification severities.
    public const uint NotificationInfo = 0;
    public const uint NotificationSuccess = 1;
    public const uint NotificationWarning = 2;
    public const uint NotificationError = 3;

    public const int StatusOk = 0;
    public const int StatusTruncated = -3;
}
