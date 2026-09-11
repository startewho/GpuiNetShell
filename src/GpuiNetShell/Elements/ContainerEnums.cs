namespace GpuiNetShell.Elements;

/// <summary>Scroll axis. `Both` drives both axes.</summary>
public enum ScrollAxis : uint
{
    Vertical = 0,
    Horizontal = 1,
    Both = 2,
}

/// <summary>Native scrollbar visibility policy.</summary>
public enum ScrollbarMode : uint
{
    Scrolling = 0,
    Hover = 1,
    Always = 2,
}

/// <summary>Resizable axis: 0 is a row, 1 is a column.</summary>
public enum ResizeAxis : uint
{
    Horizontal = 0,
    Vertical = 1,
}

/// <summary>Semantic control size shared by badge, progress, radio, and button.</summary>
public enum ControlSize : uint
{
    ExtraSmall = 0,
    Small = 1,
    Medium = 2,
    Large = 3,
}

/// <summary>Where a <see cref="PopoverElement"/> hangs relative to its trigger.</summary>
public enum PopoverAnchor : uint
{
    TopLeft = 0,
    TopCenter = 1,
    TopRight = 2,
    BottomLeft = 3,
    BottomCenter = 4,
    BottomRight = 5,
    LeftCenter = 6,
    RightCenter = 7,
}

/// <summary>The icon a <see cref="SpinnerElement"/> rotates.</summary>
public enum SpinnerIcon : uint
{
    Loader = 0,
    LoaderCircle = 1,
}

/// <summary>The rotation easing curve of a <see cref="SpinnerElement"/>.</summary>
public enum SpinnerEase : uint
{
    Linear = 0,
    EaseInOut = 1,
    EaseOutQuint = 2,
}

/// <summary>Semantic variant of a <see cref="TagElement"/>.</summary>
public enum TagVariant : uint
{
    Primary = 0,
    Secondary = 1,
    Danger = 2,
    Success = 3,
    Warning = 4,
    Info = 5,
}

/// <summary>Presentation variant of a <see cref="GroupBoxElement"/>.</summary>
public enum GroupBoxVariant : uint
{
    Normal = 0,
    Fill = 1,
    Outline = 2,
}

/// <summary>Visual variant of a <see cref="DropdownButtonElement"/>.</summary>
public enum DropdownVariant : uint
{
    Primary = 0,
    Secondary = 1,
    Danger = 2,
    Ghost = 3,
}

/// <summary>Popup-menu anchor of a <see cref="DropdownButtonElement"/>.</summary>
public enum DropdownAnchor : uint
{
    TopRight = 0,
    BottomRight = 1,
    BottomLeft = 2,
    TopLeft = 3,
}

/// <summary>Visual variant of a <see cref="TabBarElement"/>.</summary>
public enum TabVariantKind : uint
{
    Tab = 0,
    Outline = 1,
    Pill = 2,
    Segmented = 3,
    Underline = 4,
}

/// <summary>Maps a <see cref="ControlSize"/> to the wire's size literal.</summary>
internal static class SemanticSize
{
    public static string Name(ControlSize size) =>
        size switch
        {
            ControlSize.ExtraSmall => "xsmall",
            ControlSize.Small => "small",
            ControlSize.Large => "large",
            _ => "medium",
        };
}
