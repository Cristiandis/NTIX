using System.Threading.Tasks;

namespace NTIX.Core.PackageManager;

public interface ICommandRunner
{
    Task<int> RunAsync(string command, Action<string>? onOutput = null, Action<string>? onError = null);
    Task<string> RunOutputAsync(string command, bool combineStderr = false);
}
