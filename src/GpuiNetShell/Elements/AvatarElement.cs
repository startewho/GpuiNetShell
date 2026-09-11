using GpuiNetShell.Rendering;

namespace GpuiNetShell.Elements;

/// <summary>A circular avatar with a name-derived initials fallback.</summary>
public sealed class AvatarElement : Element
{
    internal AvatarElement(RenderContext ui, int index)
        : base(ui, index) { }

    /// <summary>Sets the person's name and generated initials fallback.</summary>
    public AvatarElement Name(string name)
    {
        Arena.AddMethodString(Index, "name", name);
        return this;
    }

    /// <summary>Sets the avatar's semantic size.</summary>
    public AvatarElement Size(ControlSize size)
    {
        Arena.AddMethodEnum(Index, "size", SemanticSize.Name(size));
        return this;
    }
}
