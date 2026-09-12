using GpuiNetShell.Elements;
using GpuiNetShell.Rendering;

namespace GpuiNetShell.Sample.Pages;

internal sealed class TablePage : GalleryPage
{
    public override string Title => "Table";

    public override Element Render(ref RenderContext ui) =>
        Page(
            ref ui,
            "Table",
            "A simple structural table built from typed header/body/footer/cell parts.",
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
                    ui.TableBody()
                        .Add(
                            ui.TableRow()
                                .Add(
                                    ui.TableCell().Add(ui.Text("Ada Lovelace")),
                                    ui.TableCell().Add(ui.Text("Engineer")),
                                    ui.TableCell().TextRight().Add(ui.Text("98"))
                                ),
                            ui.TableRow()
                                .Add(
                                    ui.TableCell().Add(ui.Text("Grace Hopper")),
                                    ui.TableCell().Add(ui.Text("Admiral")),
                                    ui.TableCell().TextRight().Add(ui.Text("95"))
                                ),
                            ui.TableRow()
                                .Add(
                                    ui.TableCell().Add(ui.Text("Linus Torvalds")),
                                    ui.TableCell().Add(ui.Text("Maintainer")),
                                    ui.TableCell().TextRight().Add(ui.Text("91"))
                                )
                        ),
                    ui.TableFooter()
                        .Add(
                            ui.TableRow()
                                .Add(
                                    ui.TableCell().ColSpan(2).Add(ui.Text("Average")),
                                    ui.TableCell().TextRight().Add(ui.Text("94.7"))
                                )
                        ),
                    ui.TableCaption().Add(ui.Text("Team scores for the current quarter"))
                )
        );
}
