using System.Globalization;

namespace GpuiNetShell.Elements;

/// <summary>
/// A CSS-like length, mirroring GPUI's <c>Length</c>/<c>AbsoluteLength</c>/
/// <c>DefiniteLength</c> grammar. A bare number is pixels; the other units are
/// spelled explicitly.
/// </summary>
/// <remarks>
/// The style methods keep their <see cref="double"/> overloads (pixels). A
/// <see cref="Length"/> argument selects the unit variant, so
/// <c>.W(200)</c> is pixels and <c>.W(Length.Percent(100))</c> is relative.
/// </remarks>
public readonly struct Length
{
    private readonly string _wire;

    private Length(string wire) => _wire = wire;

    internal string Wire => _wire;

    /// <summary>Sizes to content: the length is omitted.</summary>
    public static Length Auto => new("auto");

    /// <summary>An absolute number of logical pixels.</summary>
    public static Length Px(double pixels) => new(Format(pixels) + "px");

    /// <summary>A percentage of the parent's corresponding dimension.</summary>
    public static Length Percent(double percent) => new(Format(percent) + "%");

    /// <summary>A fraction of the parent (0–1), GPUI's relative length.</summary>
    public static Length Relative(double fraction) => new(Format(fraction * 100.0) + "%");

    /// <summary>An absolute number of root ems.</summary>
    public static Length Rems(double rems) => new(Format(rems) + "rem");

    /// <summary>Interprets a raw GPUI length string ("50%", "12px", "1rem", "auto").</summary>
    public static Length Raw(string value) => new(value);

    internal static string Format(double value) =>
        value.ToString("0.################", CultureInfo.InvariantCulture);

    public override string ToString() => _wire;

    public static implicit operator Length(double pixels) => Px(pixels);
}
