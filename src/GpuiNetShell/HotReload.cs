using System.Reflection.Metadata;

[assembly: MetadataUpdateHandler(typeof(GpuiNetShell.HotReload))]

namespace GpuiNetShell;

/// <summary>
/// Bridges .NET Hot Reload / edit-and-continue into the running native shell.
///
/// When the debugger applies a metadata update, the framework's caches are
/// dropped and every live session is asked to re-render. Because the managed
/// view rebuilds its element tree every frame, edits to a <c>Render</c> method
/// show up on the next frame without restarting the process.
/// </summary>
public static class HotReload
{
    /// <summary>Called before an update is applied; drops cached managed state.</summary>
    public static void ClearCache(Type[]? updatedTypes)
    {
        GpuiApplication.ClearRenderCaches();
    }

    /// <summary>Called after an update is applied; repaints every live session.</summary>
    public static void UpdateApplication(Type[]? updatedTypes)
    {
        GpuiApplication.InvalidateAll();
    }
}
