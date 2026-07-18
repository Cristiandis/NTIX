using System.Runtime.Versioning;
using System.Security.Principal;

namespace NTIX.Core;

public static class ProcessHelper
{
    [SupportedOSPlatform("windows")]
    public static bool IsRunningAsAdmin()
    {
        try
        {
            var identity = WindowsIdentity.GetCurrent();
            var principal = new WindowsPrincipal(identity);
            return principal.IsInRole(WindowsBuiltInRole.Administrator);
        }
        catch
        {
            return false;
        }
    }
}