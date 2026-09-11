namespace GpuiNetShell.Elements;

/// <summary>Visual variant of a <see cref="ButtonElement"/>.</summary>
public enum ButtonVariant : ulong
{
    Default = 0,
    Primary = 1,
    Secondary = 2,
    Danger = 3,
    Success = 4,
    Warning = 5,
    Ghost = 6,
    Link = 7,
}

/// <summary>Semantic control size of a <see cref="ButtonElement"/>.</summary>
public enum ButtonSize : ulong
{
    ExtraSmall = 0,
    Small = 1,
    Medium = 2,
    Large = 3,
}
