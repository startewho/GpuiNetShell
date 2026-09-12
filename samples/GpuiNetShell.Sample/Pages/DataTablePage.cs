using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class DataTablePage : GalleryPage
{
    private static readonly string[] Roles = ["Engineer", "Designer", "Manager", "Analyst", "Support"];
    private static readonly string[] Teams = ["Platform", "Growth", "Infra", "Design", "Data"];
    private static readonly string[] Statuses = ["Active", "Away", "Offline"];

    private readonly List<Person> _rows = CreateRows(2000);

    private string _status = "(no row action yet)";

    public override string Title => "DataTable";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "DataTable",
            $"The {_rows.Count} row objects stay in C#; the native table asks for one row index at a time.",
            ui.DataTable("people", _rows.Count)
                .Columns("Name", "Role", "Team", "Status", "Score")
                .Stripe()
                .Bordered()
                .H(440)
                .RenderCell(
                    (ctx, row, column) =>
                    {
                        var person = _rows[row];
                        return column switch
                        {
                            "Status" => ctx
                                .Tag()
                                .Variant(
                                    person.Status == "Active"
                                        ? TagVariant.Success
                                        : person.Status == "Away"
                                            ? TagVariant.Warning
                                            : TagVariant.Secondary
                                )
                                .Add(ctx.Text(person.Status)),
                            "Score" => ctx
                                .Label(person.Score.ToString())
                                .FontSemibold()
                                .TextColor(person.Score >= 80 ? "green-600" : "gray-600"),
                            "Name" => ctx.Label(person.Name).FontMedium(),
                            "Role" => ctx.Label(person.Role),
                            "Team" => ctx.Label(person.Team),
                            _ => ctx.Label(string.Empty),
                        };
                    }
                )
                .RowMenu(
                    ui.ContextMenuItem("Copy name")
                        .OnSelect(row =>
                        {
                            _status = $"copy {_rows[row].Name}";
                            Invalidate();
                        }),
                    ui.ContextMenuSeparator(),
                    ui.ContextMenuItem("Delete row")
                        .OnSelect(row =>
                        {
                            _status = $"delete {_rows[row].Name}";
                            Invalidate();
                        })
                ),
            ui.Label($"Last row action: {_status}")
        );

    private static List<Person> CreateRows(int count)
    {
        var rows = new List<Person>(count);
        for (var i = 0; i < count; i++)
        {
            rows.Add(
                new Person(
                    $"Person {i + 1}",
                    Roles[i % Roles.Length],
                    Teams[i % Teams.Length],
                    Statuses[i % Statuses.Length],
                    50 + ((i * 7) % 50)
                )
            );
        }
        return rows;
    }

    private readonly record struct Person(string Name, string Role, string Team, string Status, int Score);
}
