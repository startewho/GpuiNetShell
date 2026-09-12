using System.Text;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class DataTablePage : GalleryPage
{
    private const int RowCount = 2000;

    public override string Title => "DataTable";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "DataTable",
            $"A retained table over a large custom dataset ({RowCount} rows) with managed cell rendering.",
            ui.DataTable("people", Rows)
                .Columns("Name", "Role", "Team", "Status", "Score")
                .Stripe()
                .Bordered()
                .H(440)
                .RenderCell(
                    (ctx, args) =>
                    {
                        var cells = args[0].Split('\t');
                        var column = args[1];
                        var value = column switch
                        {
                            "Name" => cells.ElementAtOrDefault(0) ?? "",
                            "Role" => cells.ElementAtOrDefault(1) ?? "",
                            "Team" => cells.ElementAtOrDefault(2) ?? "",
                            "Status" => cells.ElementAtOrDefault(3) ?? "",
                            "Score" => cells.ElementAtOrDefault(4) ?? "",
                            _ => "",
                        };

                        return column switch
                        {
                            "Status" => ctx
                                .Tag()
                                .Variant(
                                    value == "Active"
                                        ? TagVariant.Success
                                        : value == "Away"
                                            ? TagVariant.Warning
                                            : TagVariant.Secondary
                                )
                                .Add(ctx.Text(value)),
                            "Score" => ctx.Label(value).FontSemibold().TextColor(ScoreColor(value)),
                            "Name" => ctx.Label(value).FontMedium(),
                            _ => ctx.Label(value),
                        };
                    }
                )
        );

    private static string ScoreColor(string value) =>
        int.TryParse(value, out var score) && score >= 80 ? "green-600" : "gray-600";

    private static string Rows()
    {
        var roles = new[] { "Engineer", "Designer", "Manager", "Analyst", "Support" };
        var teams = new[] { "Platform", "Growth", "Infra", "Design", "Data" };
        var statuses = new[] { "Active", "Away", "Offline" };
        var builder = new StringBuilder(RowCount * 40);
        for (var i = 0; i < RowCount; i++)
        {
            if (i > 0)
            {
                builder.Append('\n');
            }
            builder
                .Append("Person ")
                .Append(i + 1)
                .Append('\t')
                .Append(roles[i % roles.Length])
                .Append('\t')
                .Append(teams[i % teams.Length])
                .Append('\t')
                .Append(statuses[i % statuses.Length])
                .Append('\t')
                .Append(50 + ((i * 7) % 50));
        }
        return builder.ToString();
    }
}
