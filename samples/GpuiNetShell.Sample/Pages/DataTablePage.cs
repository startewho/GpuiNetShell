using System.Globalization;
using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class DataTablePage : GalleryPage
{
    private static readonly string[] Roles = ["Engineer", "Designer", "Manager", "Analyst", "Support"];
    private static readonly string[] Teams = ["Platform", "Growth", "Infra", "Design", "Data"];
    private static readonly string[] Statuses = ["Active", "Away", "Offline"];

    private readonly List<Person> _rows = CreateRows(2000);

    private string _status = "(no row action yet)";

    public override string Title => "DataTable";

    public override Element Render(ref RenderContext ui)
    {
        // The source generator registered `RenderRowCell` and produced
        // `RenderCellToken`; the table asks managed code for one row index at a
        // time and this page's list stays in C#.
        RegisterGeneratedCallbacks(ref ui);
        return Page(
            ref ui,
            "DataTable",
            $"The {_rows.Count} row objects stay in C#; the native table asks for one row index at a time.",
            ui.DataTable("people", _rows.Count)
                .Columns("Name", "Role", "Team", "Status", "Score")
                .Stripe()
                .Bordered()
                .H(440)
                .RenderCell(RenderCellToken)
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
    }

    [GpuiCallback("RenderCell")]
    private Element RenderRowCell(RenderContext ui, IReadOnlyList<string> arguments)
    {
        var row = int.Parse(arguments[0], CultureInfo.InvariantCulture);
        var column = arguments[1];
        var person = _rows[row];
        return column switch
        {
            "Status" => ui.Tag()
                .Variant(
                    person.Status == "Active"
                        ? TagVariant.Success
                        : person.Status == "Away"
                            ? TagVariant.Warning
                            : TagVariant.Secondary
                )
                .Add(ui.Text(person.Status)),
            "Score" => ui.Label(person.Score.ToString())
                .FontSemibold()
                .TextColor(person.Score >= 80 ? "green-600" : "gray-600"),
            "Name" => ui.Label(person.Name).FontMedium(),
            "Role" => ui.Label(person.Role),
            "Team" => ui.Label(person.Team),
            _ => ui.Label(string.Empty),
        };
    }

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
