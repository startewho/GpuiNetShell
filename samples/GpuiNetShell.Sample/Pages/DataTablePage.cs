using System.Globalization;
using GpuiNetShell.Elements;
using GpuiNetShell.Entities;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

[GpuiCallbacks]
internal sealed partial class DataTablePage : GalleryPage<DataTablePage.State>
{
    private static readonly string[] Roles = ["Engineer", "Designer", "Manager", "Analyst", "Support"];
    private static readonly string[] Teams = ["Platform", "Growth", "Infra", "Design", "Data"];
    private static readonly string[] Statuses = ["Active", "Away", "Offline"];

    private static readonly List<Person> Rows = CreateRows(200_000);

    internal sealed class State
    {
        public string Status { get; set; } = "(no row action yet)";
    }

    public override string Title => "DataTable";

    protected override ulong RegisterPageCallbacks(ref RenderContext ui)
    {
        RegisterGeneratedCallbacks(ref ui);
        return PageToken;
    }

    // The source generator produced `RenderCellToken` and `PageToken`; the table
    // asks managed code for one row index at a time and this page's list stays
    // in C#.
    [GpuiCallback("Page")]
    private Element RenderPage(State state, RenderContext ui, Context<State> cx)
    {
        return Page(
            ref ui,
            "DataTable",
            $"The {Rows.Count} row objects stay in C#; the native table asks for one row index at a time.",
            ui.DataTable("people", Rows.Count)
                .Columns("Name", "Role", "Team", "Status", "Score")
                .Stripe()
                .Bordered()
                .H(440)
                .RenderCell(RenderCellToken)
                .RowMenu(
                    ui.ContextMenuItem("Copy name")
                        .OnSelect(row =>
                            Update(
                                (s, c) =>
                                {
                                    s.Status = $"copy {Rows[row].Name}";
                                    c.Notify();
                                }
                            )
                        ),
                    ui.ContextMenuSeparator(),
                    ui.ContextMenuItem("Delete row")
                        .OnSelect(row =>
                            Update(
                                (s, c) =>
                                {
                                    s.Status = $"delete {Rows[row].Name}";
                                    c.Notify();
                                }
                            )
                        )
                ),
            ui.Label($"Last row action: {state.Status}")
        );
    }

    [GpuiCallback("RenderCell")]
    private Element RenderRowCell(RenderContext ui, IReadOnlyList<string> arguments)
    {
        var row = int.Parse(arguments[0], CultureInfo.InvariantCulture);
        var column = arguments[1];
        var person = Rows[row];
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
