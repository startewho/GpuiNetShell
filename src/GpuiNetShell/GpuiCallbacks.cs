namespace GpuiNetShell;

/// <summary>
/// Marks a partial class whose <see cref="GpuiCallbackAttribute"/> methods should
/// have callback registration generated. The class is usually the application's
/// <see cref="View"/>.
/// </summary>
/// <remarks>
/// The generator produces a token property per annotated method and a
/// <c>RegisterGeneratedCallbacks(ref RenderContext)</c> method that registers
/// them. Call it at the top of <c>Render</c>; then hand the token to the element
/// that should invoke the callback.
/// </remarks>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class GpuiCallbacksAttribute : Attribute { }

/// <summary>
/// Marks a method as a managed callback the native host may invoke. The
/// generator infers the callback kind from the signature:
/// <list type="bullet">
/// <item><c>void M()</c> — parameterless action.</item>
/// <item><c>void M(bool|double|string)</c> — typed action.</item>
/// <item><c>string M()</c> — row-snapshot provider.</item>
/// <item><c>string M(double, double)</c> — canvas measure; returns <c>"width\theight"</c>.</item>
/// <item><c>Element M(RenderContext, IReadOnlyList&lt;string&gt;)</c> — element renderer
/// (used as a <c>Canvas.Prepaint</c> callback).</item>
/// <item><c>Element M(TState, RenderContext, Context&lt;TState&gt;)</c> — entity view;
/// register with <c>RegisterEntityView</c> and render through
/// <c>ui.Child(entity, token)</c>.</item>
/// </list>
/// </summary>
[AttributeUsage(AttributeTargets.Method, Inherited = false)]
public sealed class GpuiCallbackAttribute : Attribute
{
    public GpuiCallbackAttribute(string name) => Name = name;

    /// <summary>The generated token property name (before the <c>Token</c> suffix).</summary>
    public string Name { get; }
}
