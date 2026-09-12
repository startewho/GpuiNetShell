using System.Text;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class ListPage : GalleryPage
{
    private const int RowCount = 500;

    public override string Title => "List";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "List",
            $"A virtualized list backed by a large row snapshot ({RowCount} rows) with a custom row renderer.",
            ui.List("people", Rows)
                .RenderRow(
                    (ctx, fields) =>
                    {
                        var name = fields.ElementAtOrDefault(1) ?? "";
                        var role = fields.ElementAtOrDefault(2) ?? "";
                        var disabled = string.Equals(
                            fields.ElementAtOrDefault(3),
                            "true",
                            StringComparison.Ordinal
                        );
                        return ctx
                            .HStack(
                                ctx.Label(name).FontMedium(),
                                ctx.Label(role).TextSize(12)
                            )
                            .Gap(8)
                            .ItemsCenter()
                            .Opacity(disabled ? 0.5 : 1.0);
                    }
                )
                .Full()
                .H(360)
        );

    private static string Rows()
    {
        var builder = new StringBuilder();
        var roles = new[] { "Engineer", "Designer", "Manager", "Analyst", "Support" };
        for (var i = 0; i < RowCount; i++)
        {
            if (i > 0)
            {
                builder.Append('\n');
            }
            builder
                .Append("row-")
                .Append(i)
                .Append('\t')
                .Append("Person ")
                .Append(i + 1)
                .Append('\t')
                .Append(roles[i % roles.Length])
                .Append('\t')
                .Append(i % 37 == 0 ? "true" : "false");
        }
        return builder.ToString();
    }
}
