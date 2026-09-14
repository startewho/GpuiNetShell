using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Text;

namespace GpuiNetShell.SourceGen;

/// <summary>
/// Generates the managed side of the native callback bridge.
///
/// For every partial class marked <c>[GpuiCallbacks]</c>, each method marked
/// <c>[GpuiCallback("Name")]</c> gets a token property and one entry in a
/// generated <c>RegisterGeneratedCallbacks(ref RenderContext)</c> method. The
/// callback kind is inferred from the method signature, so rendering a row is
/// just a method instead of a hand-written <c>Events.RegisterElement</c> lambda.
/// </summary>
[Generator(LanguageNames.CSharp)]
public sealed class GpuiCallbackGenerator : IIncrementalGenerator
{
    private const string CallbacksAttribute = "GpuiNetShell.GpuiCallbacksAttribute";
    private const string CallbackAttribute = "GpuiNetShell.GpuiCallbackAttribute";

    public void Initialize(IncrementalGeneratorInitializationContext context)
    {
        IncrementalValuesProvider<CallbackClass> classes = context.SyntaxProvider
            .ForAttributeWithMetadataName(
                CallbacksAttribute,
                static (node, _) => node is ClassDeclarationSyntax,
                static (ctx, _) => Transform(ctx)
            )
            .Where(static model => model is not null)
            .Select(static (model, _) => model!.Value);

        context.RegisterSourceOutput(classes, static (production, model) => Emit(production, model));
    }

    private static CallbackClass? Transform(GeneratorAttributeSyntaxContext context)
    {
        if (context.TargetSymbol is not INamedTypeSymbol type)
        {
            return null;
        }
        if (!type.ContainingTypeIsNullOrGlobal())
        {
            return null;
        }

        var methods = ImmutableArray.CreateBuilder<CallbackMethod>();
        foreach (ISymbol member in type.GetMembers())
        {
            if (member is not IMethodSymbol method)
            {
                continue;
            }
            AttributeData? attribute = FindAttribute(method, CallbackAttribute);
            if (attribute is null)
            {
                continue;
            }
            string name =
                attribute.ConstructorArguments.Length == 1
                && attribute.ConstructorArguments[0].Value is string text
                    ? text
                    : method.Name;
            CallbackKind? kind = InferKind(method);
            if (kind is null)
            {
                continue;
            }
            string stateType =
                kind == CallbackKind.EntityView
                    ? method.Parameters[0].Type.ToDisplayString()
                    : string.Empty;
            methods.Add(new CallbackMethod(method.Name, name + "Token", kind.Value, stateType));
        }

        string accessibility = type.DeclaredAccessibility switch
        {
            Accessibility.Public => "public ",
            Accessibility.Internal => "internal ",
            _ => "internal ",
        };
        string typeParameters = type.TypeParameters.Length == 0
            ? string.Empty
            : "<" + string.Join(", ", TypeParameterNames(type.TypeParameters)) + ">";

        return new CallbackClass(
            type.ContainingNamespace.IsGlobalNamespace
                ? string.Empty
                : type.ContainingNamespace.ToDisplayString(),
            type.Name,
            accessibility,
            typeParameters,
            methods.ToImmutable()
        );
    }

    private static IEnumerable<string> TypeParameterNames(ImmutableArray<ITypeParameterSymbol> parameters)
    {
        foreach (ITypeParameterSymbol parameter in parameters)
        {
            yield return parameter.Name;
        }
    }

    private static AttributeData? FindAttribute(ISymbol symbol, string metadataName)
    {
        foreach (AttributeData attribute in symbol.GetAttributes())
        {
            if (attribute.AttributeClass?.ToDisplayString() == metadataName)
            {
                return attribute;
            }
        }
        return null;
    }

    private static CallbackKind? InferKind(IMethodSymbol method)
    {
        string returnType = method.ReturnType.ToDisplayString();
        if (returnType == "void")
        {
            if (method.Parameters.Length == 0)
            {
                return CallbackKind.Action;
            }
            if (method.Parameters.Length == 1)
            {
                return method.Parameters[0].Type.SpecialType switch
                {
                    SpecialType.System_Boolean => CallbackKind.ActionBool,
                    SpecialType.System_Double => CallbackKind.ActionNumber,
                    SpecialType.System_String => CallbackKind.ActionString,
                    _ => null,
                };
            }
            return null;
        }
        if (returnType == "string" && method.Parameters.Length == 0)
        {
            return CallbackKind.Rows;
        }
        if (
            returnType == "string"
            && method.Parameters.Length == 2
            && method.Parameters[0].Type.SpecialType == SpecialType.System_Double
            && method.Parameters[1].Type.SpecialType == SpecialType.System_Double
        )
        {
            return CallbackKind.Measure;
        }
        if (
            returnType == "GpuiNetShell.Elements.Element"
            && method.Parameters.Length == 3
            && method.Parameters[1].Type.ToDisplayString() == "GpuiNetShell.Rendering.RenderContext"
            && IsContextOf(method.Parameters[2].Type, method.Parameters[0].Type)
        )
        {
            return CallbackKind.EntityView;
        }
        if (
            returnType == "GpuiNetShell.Elements.Element"
            && method.Parameters.Length == 2
            && method.Parameters[1].Type.ToDisplayString()
                == "System.Collections.Generic.IReadOnlyList<string>"
        )
        {
            return CallbackKind.Element;
        }
        return null;
    }

    /// <summary>
    /// True when <paramref name="type"/> is <c>Context&lt;T&gt;</c> instantiated
    /// with the same state type as the entity view's first parameter.
    /// </summary>
    private static bool IsContextOf(ITypeSymbol type, ITypeSymbol state)
    {
        if (type is not INamedTypeSymbol named || named.TypeArguments.Length != 1)
        {
            return false;
        }
        if (
            named.Name != "Context"
            || named.ContainingNamespace.ToDisplayString() != "GpuiNetShell.Entities"
        )
        {
            return false;
        }
        return SymbolEqualityComparer.Default.Equals(named.TypeArguments[0], state);
    }

    private static void Emit(SourceProductionContext context, CallbackClass model)
    {
        var builder = new StringBuilder();
        builder.AppendLine("// <auto-generated/>");
        builder.AppendLine("#nullable enable");
        if (model.Namespace.Length != 0)
        {
            builder.Append("namespace ").Append(model.Namespace).AppendLine();
            builder.AppendLine("{");
        }
        builder
            .Append(model.Accessibility)
            .Append("partial class ")
            .Append(model.ClassName)
            .Append(model.TypeParameters)
            .AppendLine();
        builder.AppendLine("{");
        foreach (CallbackMethod method in model.Methods)
        {
            builder
                .Append("    /// <summary>Callback token for <c>")
                .Append(method.MethodName)
                .Append("</c>.</summary>")
                .AppendLine();
            builder
                .Append("    internal ulong ")
                .Append(method.TokenName)
                .Append(" { get; private set; }")
                .AppendLine();
        }
        builder.AppendLine();
        builder.AppendLine("    /// <summary>Registers every [GpuiCallback] method. Call at the top of Render.</summary>");
        builder.AppendLine(
            "    private void RegisterGeneratedCallbacks(ref global::GpuiNetShell.Rendering.RenderContext ui)"
        );
        builder.AppendLine("    {");
        foreach (CallbackMethod method in model.Methods)
        {
            builder
                .Append("        ")
                .Append(method.TokenName)
                .Append(" = ")
                .Append(Registration(method, Key(model, method)))
                .AppendLine(";");
        }
        builder.AppendLine("    }");
        builder.AppendLine("}");
        if (model.Namespace.Length != 0)
        {
            builder.AppendLine("}");
        }

        context.AddSource(
            HintName(model),
            SourceText.From(builder.ToString(), Encoding.UTF8)
        );
    }

    private static string Registration(CallbackMethod method, string key) =>
        method.Kind switch
        {
            CallbackKind.Action => $"ui.RegisterCallback({method.MethodName})",
            CallbackKind.ActionBool =>
                $"ui.RegisterCallback(value => {method.MethodName}(value.Boolean))",
            CallbackKind.ActionNumber =>
                $"ui.RegisterCallback(value => {method.MethodName}(value.Number))",
            CallbackKind.ActionString =>
                $"ui.RegisterCallback(value => {method.MethodName}(value.String ?? string.Empty))",
            CallbackKind.Rows => $"ui.RegisterRows({method.MethodName})",
            CallbackKind.Element => $"ui.RegisterElement({method.MethodName})",
            CallbackKind.Measure => $"ui.RegisterMeasure({method.MethodName})",
            CallbackKind.EntityView =>
                $"ui.RegisterEntityView<{method.StateType}>({Literal(key)}, {method.MethodName})",
            _ => "0",
        };

    /// <summary>A stable, session-unique key for one entity-view method.</summary>
    private static string Key(CallbackClass model, CallbackMethod method) =>
        (model.Namespace.Length == 0 ? string.Empty : model.Namespace + ".")
        + model.ClassName
        + "."
        + method.MethodName;

    /// <summary>A C# string literal for <paramref name="value"/>.</summary>
    private static string Literal(string value) =>
        "\"" + value.Replace("\\", "\\\\").Replace("\"", "\\\"") + "\"";

    private static string HintName(CallbackClass model) =>
        model.Namespace.Length == 0
            ? model.ClassName + ".GpuiCallbacks.g.cs"
            : model.Namespace.Replace('.', '_') + "_" + model.ClassName + ".GpuiCallbacks.g.cs";
}

internal enum CallbackKind
{
    Action,
    ActionBool,
    ActionNumber,
    ActionString,
    Rows,
    Element,
    /// <summary>A canvas measure: <c>string M(double availableWidth, double availableHeight)</c>.</summary>
    Measure,
    /// <summary>An entity-view renderer: <c>Element M(TState, RenderContext, Context&lt;TState&gt;)</c>.</summary>
    EntityView,
}

internal readonly record struct CallbackMethod(
    string MethodName,
    string TokenName,
    CallbackKind Kind,
    string StateType
);

internal readonly record struct CallbackClass(
    string Namespace,
    string ClassName,
    string Accessibility,
    string TypeParameters,
    ImmutableArray<CallbackMethod> Methods
);

internal static class SymbolExtensions
{
    public static bool ContainingTypeIsNullOrGlobal(this INamedTypeSymbol type) =>
        type.ContainingType is null;
}
