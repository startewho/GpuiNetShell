namespace GpuiNetShell;

/// <summary>Severity of a notification posted through <see cref="GpuiApplication.PushNotification"/>.</summary>
public enum NotificationLevel : uint
{
    Info = 0,
    Success = 1,
    Warning = 2,
    Error = 3,
}
