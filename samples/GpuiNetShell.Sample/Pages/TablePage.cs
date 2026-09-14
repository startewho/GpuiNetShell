using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TablePage : GalleryPage
{
    // The structural `Table` is not virtualized: every row and cell becomes a
    // native layout/paint element, so its footprint grows ~linearly with rows
    // (about 0.1 MB per row in a release build). Keep this demo small; the
    // `DataTable` and `VirtualList` pages show large datasets virtualized.
    private const int RowCount = 20;

    public override string Title => "Table";

    public override Element Render(ref RenderContext ui)
    {
        var body = new List<TableRowElement>(RowCount);
        for (var i = 0; i < RowCount; i++)
        {
            var index = i;
            body.Add(
                ui.TableRow()
                    .Add(
                        ui.TableCell().Add(ui.Text($"Person {index + 1}")),
                        ui.TableCell().Add(ui.Text(IndexRole(index))),
                        ui.TableCell().TextRight().Add(ui.Text((50 + ((index * 7) % 50)).ToString()))
                    )
            );
        }

        return Page(
            ref ui,
            "Table",
            $"A structural table built from typed parts with {RowCount} rows.",
            ui.Table()
                .AccessibilityLabel("Team scores")
                .Size(ControlSize.Medium)
                .Add(
                    ui.TableHeader()
                        .Add(
                            ui.TableRow()
                                .Add(
                                    ui.TableHead().Add(ui.Text("Name")),
                                    ui.TableHead().Add(ui.Text("Role")),
                                    ui.TableHead().TextRight().Add(ui.Text("Score"))
                                )
                        ),
                    ui.TableBody().Add(body.ToArray()),
                    ui.TableFooter()
                        .Add(
                            ui.TableRow()
                                .Add(
                                    ui.TableCell().ColSpan(2).Add(ui.Text("Average")),
                                    ui.TableCell().TextRight().Add(ui.Text("74.5"))
                                )
                        ),
                    ui.TableCaption().Add(ui.Text("Generated rows for the current quarter"))
                )
        );
    }

    private static string IndexRole(int index) =>
        (index % 5) switch
        {
            0 => "Engineer",
            1 => "Designer",
            2 => "Manager",
            3 => "Analyst",
            _ => "Support",
        };
}
