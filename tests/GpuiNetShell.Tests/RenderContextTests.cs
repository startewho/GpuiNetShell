using GpuiNetShell.Elements;
using GpuiNetShell.Events;
using GpuiNetShell.Interop;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Tests;

public sealed unsafe class RenderContextTests
{
    [Fact]
    public void ButtonBuildsIdentityLabelVariantAndClick()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });
        var calls = 0;

        ui.Button("save").Label("Save").Primary().OnClick(() => calls++);

        var descriptor = arena.Publish();
        Assert.Equal(1u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentButton, descriptor.Nodes[0].Component);
        Assert.Equal(3u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpMethod, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.ArgString, descriptor.Ops[0].Flags);
        Assert.Equal(NativeProtocol.OpMethod, descriptor.Ops[1].Code);
        Assert.Equal(NativeProtocol.ArgNone, descriptor.Ops[1].Flags);
        Assert.Equal(NativeProtocol.OpCallback, descriptor.Ops[2].Code);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        Assert.Equal("label", DecodePacked(descriptor.Ops[0].A, utf8));
        Assert.Equal("Save", DecodePacked(descriptor.Ops[0].B, utf8));
        Assert.Equal("primary", DecodePacked(descriptor.Ops[1].A, utf8));
        Assert.Equal("on_click", DecodePacked(descriptor.Ops[2].A, utf8));
        Assert.Equal(0, calls);
    }

    [Fact]
    public void ContainerRecordsChildEdgesAndStyleCalls()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.VStack(ui.Text("one"), ui.Text("two")).Gap(8);

        var descriptor = arena.Publish();
        Assert.Equal(3u, descriptor.NodesLen);
        Assert.Equal(2u, descriptor.ChildrenLen);
        Assert.Equal(NativeProtocol.ComponentText, descriptor.Nodes[1].Component);
        Assert.Equal(2u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpNullaryStyle, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.OpParamStyle, descriptor.Ops[1].Code);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        Assert.Equal("flex_col", DecodePacked(descriptor.Ops[0].A, utf8));
        Assert.Equal("gap", DecodePacked(descriptor.Ops[1].A, utf8));
        Assert.Equal(8.0f, BitConverter.UInt32BitsToSingle((uint)descriptor.Ops[1].B));
    }

    [Fact]
    public void StyleCallsRecordTheGpuiMethodNameAndArgument()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Div().P(12).Bg("#112233").Full();

        var descriptor = arena.Publish();
        Assert.Equal(3u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpParamStyle, descriptor.Ops[0].Code);
        Assert.Equal(12.0f, BitConverter.UInt32BitsToSingle((uint)descriptor.Ops[0].B));
        Assert.Equal(NativeProtocol.OpParamStyle, descriptor.Ops[1].Code);
        Assert.Equal(NativeProtocol.OpNullaryStyle, descriptor.Ops[2].Code);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        Assert.Equal("p", DecodePacked(descriptor.Ops[0].A, utf8));
        Assert.Equal("bg", DecodePacked(descriptor.Ops[1].A, utf8));
        Assert.Equal("#112233", DecodePacked(descriptor.Ops[1].B, utf8));
        Assert.Equal("size_full", DecodePacked(descriptor.Ops[2].A, utf8));
    }

    [Fact]
    public void LabelBadgeAndProgressRecordTheirIdentitiesAndMethods()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Label("Name");
        ui.Badge().Count(7).Dot();
        ui.Progress("bar").Value(50).Loading(false);

        var descriptor = arena.Publish();
        Assert.Equal(3u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentLabel, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentBadge, descriptor.Nodes[1].Component);
        Assert.Equal((uint)NativeProtocol.ComponentProgress, descriptor.Nodes[2].Component);
        // badge count + dot; progress value + loading
        Assert.Equal(4u, descriptor.OpsLen);
        Assert.Equal(NativeProtocol.OpMethod, descriptor.Ops[0].Code);
        Assert.Equal(NativeProtocol.ArgNumber, descriptor.Ops[0].Flags);
    }

    [Fact]
    public void ComboboxRecordsOptionsSelectionAndTokens()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Combobox("theme").Options("Light", "Dark").Selected(1).OnChange(_ => { });

        var descriptor = arena.Publish();
        Assert.Equal(1u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentCombobox, descriptor.Nodes[0].Component);
        // options + selected + tokens
        Assert.Equal(3u, descriptor.OpsLen);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        Assert.Equal("options", DecodePacked(descriptor.Ops[0].A, utf8));
        Assert.Equal("Light\nDark", DecodePacked(descriptor.Ops[0].B, utf8));
        Assert.Equal("tokens", DecodePacked(descriptor.Ops[2].A, utf8));
    }

    [Fact]
    public void NotifyInvokesTheInvalidator()
    {
        using var arena = new RenderArena();
        var calls = 0;
        var ui = new RenderContext(arena, new EventRegistry(), () => calls++);

        ui.Notify();

        Assert.Equal(1, calls);
    }

    [Fact]
    public void NotifyDuringRenderThrowsAndWorksOutsideIt()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.BeginRender();
        Assert.Throws<InvalidOperationException>(() => ui.Notify());
        ui.EndRender();

        ui.Notify();
    }

    [Fact]
    public void RadioAndTabsRecordTheirState()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Radio("r").Label("Light").Checked().OnChange(_ => { });
        ui.Tabs("t").Options("A", "B").Selected(1).OnChange(_ => { });

        var descriptor = arena.Publish();
        Assert.Equal(2u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentRadio, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentTabs, descriptor.Nodes[1].Component);
        // radio: label + checked + on_change; tabs: options + selected + tokens
        Assert.Equal(6u, descriptor.OpsLen);
    }

    [Fact]
    public void ScrollResizableAndPopoverRecordTheirState()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Popover("p", "Open").Content(ui.Label("Body")).DefaultOpen();
        ui.Scroll("s").Axis(ScrollAxis.Vertical);
        ui.Resizable("r").Axis(ResizeAxis.Horizontal).Sizes("100", "*");

        var descriptor = arena.Publish();
        Assert.Equal(4u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentPopover, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentScroll, descriptor.Nodes[2].Component);
        Assert.Equal((uint)NativeProtocol.ComponentResizable, descriptor.Nodes[3].Component);
        // popover content + default_open; scroll axis; resizable axis + sizes
        Assert.Equal(5u, descriptor.OpsLen);
        var codes = Enumerable
            .Range(0, (int)descriptor.OpsLen)
            .Select(index => descriptor.Ops[index].Code)
            .ToArray();
        Assert.Contains(NativeProtocol.OpSlot, codes);
    }

    [Fact]
    public void FeedbackComponentsRecordTheirConstructorsAndMethods()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Spinner()
            .Size(ControlSize.Large)
            .Icon(SpinnerIcon.LoaderCircle)
            .Color("blue-600")
            .Ease(SpinnerEase.EaseOutQuint);
        ui.VerticalDashedSeparator().Label("Account").Color("red-500");
        ui.Skeleton().Secondary();
        ui.Tag().Variant(TagVariant.Danger).Outline().RoundedFull().Size(ControlSize.Small);

        var descriptor = arena.Publish();
        Assert.Equal(4u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentSpinner, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentSeparator, descriptor.Nodes[1].Component);
        Assert.Equal((uint)NativeProtocol.ComponentSkeleton, descriptor.Nodes[2].Component);
        Assert.Equal((uint)NativeProtocol.ComponentTag, descriptor.Nodes[3].Component);

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        var separatorData =
            ((ulong)descriptor.Nodes[1].DataOffset << 32) | descriptor.Nodes[1].DataLen;
        Assert.Equal("VerticalDashedSeparator", DecodePacked(separatorData, utf8));
        Assert.Equal(NativeProtocol.ArgEnum, descriptor.Ops[0].Flags);
    }

    [Fact]
    public void NavigationComponentsRecordTheirConstructorsAndMethods()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Link("docs").Href("https://example.com").OnClick(() => { });
        ui.Kbd("ctrl-k").Outline();
        ui.Avatar().Name("Ada Lovelace").Size(ControlSize.Large);
        ui.Icon("icons/check.svg").Size(ControlSize.Small).Rotate(0.5);

        var descriptor = arena.Publish();
        Assert.Equal(4u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentLink, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentKbd, descriptor.Nodes[1].Component);
        Assert.Equal((uint)NativeProtocol.ComponentAvatar, descriptor.Nodes[2].Component);
        Assert.Equal((uint)NativeProtocol.ComponentIcon, descriptor.Nodes[3].Component);
        Assert.Contains(
            Enumerable.Range(0, (int)descriptor.OpsLen),
            index => descriptor.Ops[index].Code == NativeProtocol.OpCallback
        );
    }

    [Fact]
    public void InteractiveComponentsRecordTheirConstructorsAndMethods()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Collapsible()
            .Open()
            .Content(ui.Label("Body"))
            .Add(ui.Button("toggle").Label("Toggle"));
        ui.Pagination("pages").TotalPages(10).CurrentPage(2).VisiblePages(5).OnChange(_ => { });
        ui.Rating("quality").Max(5).Value(3).OnChange(_ => { });
        ui.Clipboard("copy").Value("text").Tooltip("Copy");

        var descriptor = arena.Publish();
        Assert.Equal(6u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentCollapsible, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentPagination, descriptor.Nodes[3].Component);
        Assert.Equal((uint)NativeProtocol.ComponentRating, descriptor.Nodes[4].Component);
        Assert.Equal((uint)NativeProtocol.ComponentClipboard, descriptor.Nodes[5].Component);
        Assert.Contains(NativeProtocol.OpSlot, Enumerable
            .Range(0, (int)descriptor.OpsLen)
            .Select(index => descriptor.Ops[index].Code));
    }

    [Fact]
    public void DisplayComponentsRecordTheirConstructorsAndMethods()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Breadcrumb("Home", "Settings", "Profile");
        ui.GroupBox().Title("Options").Variant(GroupBoxVariant.Outline).Add(ui.Text("Body"));
        ui.StatusBar().LeftContent(ui.Label("Ready")).RightContent(ui.Label("v1"));
        ui.WarningAlert("net", "Offline").Title("Connection").Banner();

        var descriptor = arena.Publish();
        Assert.Equal(7u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentBreadcrumb, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentGroupBox, descriptor.Nodes[1].Component);
        Assert.Equal((uint)NativeProtocol.ComponentStatusBar, descriptor.Nodes[3].Component);
        Assert.Equal((uint)NativeProtocol.ComponentAlert, descriptor.Nodes[6].Component);
        Assert.Equal(1u, descriptor.ChildrenLen);
        Assert.Contains(
            Enumerable.Range(0, (int)descriptor.OpsLen),
            index => descriptor.Ops[index].Flags == NativeProtocol.ArgElement
        );

        var utf8 = new ReadOnlySpan<byte>(descriptor.Utf8, checked((int)descriptor.Utf8Len));
        var breadcrumbData =
            ((ulong)descriptor.Nodes[0].DataOffset << 32) | descriptor.Nodes[0].DataLen;
        Assert.Equal("Home\nSettings\nProfile", DecodePacked(breadcrumbData, utf8));
    }

    [Fact]
    public void MenuComponentsRecordElementAndStringCallbackMethods()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Tooltip("tip", "Help", "Shows help");
        ui.HoverCard("card")
            .TriggerElement(ui.Button("hover").Label("Hover"))
            .Content(ui.Label("Body"))
            .OpenDelay(250);
        ui.DropdownMenu("menu", "Actions").Item("Copy", () => { }).Item("Paste", () => { });
        ui.DropdownButton("split", "Run").Variant(DropdownVariant.Primary).MenuItem("Once", () => { });

        var descriptor = arena.Publish();
        Assert.Equal(6u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentTooltip, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentHoverCard, descriptor.Nodes[1].Component);
        Assert.Equal((uint)NativeProtocol.ComponentDropdownMenu, descriptor.Nodes[4].Component);
        Assert.Equal((uint)NativeProtocol.ComponentDropdownButton, descriptor.Nodes[5].Component);

        var flags = Enumerable
            .Range(0, (int)descriptor.OpsLen)
            .Select(index => descriptor.Ops[index].Flags)
            .ToArray();
        Assert.Contains(NativeProtocol.ArgElement, flags);
        Assert.Contains(NativeProtocol.ArgStringCallback, flags);
    }

    [Fact]
    public void CollectionComponentsRecordTheirConstructors()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.TabBar("tabs")
            .SelectedIndex(1)
            .Variant(TabVariantKind.Pill)
            .OnChange(_ => { })
            .Add(ui.Tab().Label("One"), ui.Tab().Label("Two"));
        ui.List("list", () => "a\tAlpha").Full();
        ui.Select("select", () => "light\tLight", _ => { }).Placeholder("Pick");
        ui.DataTable("table", () => "Ada\tEngineer")
            .Columns("Name", "Role")
            .RenderCell((ctx, args) => ctx.Label(args[0]));

        var descriptor = arena.Publish();
        // tabbar(0), tab(1), tab(2), list(3), select(4), datatable(5)
        Assert.Equal(6u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentTabBar, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentTab, descriptor.Nodes[1].Component);
        Assert.Equal((uint)NativeProtocol.ComponentList, descriptor.Nodes[3].Component);
        Assert.Equal((uint)NativeProtocol.ComponentSelect, descriptor.Nodes[4].Component);
        Assert.Equal((uint)NativeProtocol.ComponentDataTable, descriptor.Nodes[5].Component);
        Assert.Contains(NativeProtocol.OpCallback, Enumerable
            .Range(0, (int)descriptor.OpsLen)
            .Select(index => descriptor.Ops[index].Code));
    }

    [Fact]
    public void TypedCompoundComponentsRecordTheirChildren()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.Accordion("acc")
            .Multiple()
            .Add(
                ui.AccordionItem().Title(ui.Label("One")).Add(ui.Text("Body")),
                ui.AccordionItem().Title(ui.Label("Two")).Open()
            );
        ui.Stepper("steps")
            .SelectedIndex(1)
            .OnChange(_ => { })
            .Add(ui.StepperItem().Add(ui.Text("Choose")), ui.StepperItem().Disabled());

        var descriptor = arena.Publish();
        // accordion(0), item(1), label(2), text(3), item(4), label(5), stepper(6), item(7), text(8), item(9)
        Assert.Equal(10u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentAccordion, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentAccordionItem, descriptor.Nodes[1].Component);
        Assert.Equal((uint)NativeProtocol.ComponentStepper, descriptor.Nodes[6].Component);
        Assert.Equal((uint)NativeProtocol.ComponentStepperItem, descriptor.Nodes[7].Component);
        Assert.Contains(NativeProtocol.ArgElement, Enumerable
            .Range(0, (int)descriptor.OpsLen)
            .Select(index => descriptor.Ops[index].Flags));
    }

    [Fact]
    public void StructuredComponentsRecordTheirChildren()
    {
        using var arena = new RenderArena();
        var ui = new RenderContext(arena, new EventRegistry(), () => { });

        ui.DescriptionList()
            .Columns(2)
            .Add(
                ui.DescriptionItem("Name").Value("Ada"),
                ui.DescriptionItem("Role").Value("Engineer").Span(2)
            );
        ui.HForm()
            .Columns(2)
            .Add(
                ui.Field().Label("Name").Add(ui.Text("...")),
                ui.Field().Label("Role").Required()
            );

        var descriptor = arena.Publish();
        // dlist(0), item(1), item(2), form(3), field(4), text(5), field(6)
        Assert.Equal(7u, descriptor.NodesLen);
        Assert.Equal((uint)NativeProtocol.ComponentDescriptionList, descriptor.Nodes[0].Component);
        Assert.Equal((uint)NativeProtocol.ComponentDescriptionItem, descriptor.Nodes[1].Component);
        Assert.Equal((uint)NativeProtocol.ComponentForm, descriptor.Nodes[3].Component);
        Assert.Equal((uint)NativeProtocol.ComponentField, descriptor.Nodes[4].Component);
    }

    private static string DecodePacked(ulong packed, ReadOnlySpan<byte> utf8)
    {
        var offset = (int)(packed >> 32);
        var length = (int)(packed & 0xFFFF_FFFF);
        return System.Text.Encoding.UTF8.GetString(utf8.Slice(offset, length));
    }
}

