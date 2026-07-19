using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using NTIX.Core.PackageManager;

namespace NTIX.Tests;

public class MockCommandRunner : ICommandRunner
{
    public List<string> CapturedCommands { get; } = new();
    public Func<string, int>? RunAsyncHandler { get; set; }
    public Dictionary<string, string> OutputResponses { get; } = new(StringComparer.OrdinalIgnoreCase);

    public Task<int> RunAsync(string command, Action<string>? onOutput = null, Action<string>? onError = null)
    {
        CapturedCommands.Add(command);

        if (RunAsyncHandler != null)
            return Task.FromResult(RunAsyncHandler(command));

        foreach (var kvp in OutputResponses)
        {
            if (command.Contains(kvp.Key, StringComparison.OrdinalIgnoreCase))
                return Task.FromResult(0);
        }

        return Task.FromResult(0);
    }

    public Task<string> RunOutputAsync(string command, bool combineStderr = false)
    {
        CapturedCommands.Add(command);

        if (OutputResponses.TryGetValue(command, out var output))
            return Task.FromResult(output);

        foreach (var kvp in OutputResponses)
        {
            if (command.Contains(kvp.Key, StringComparison.OrdinalIgnoreCase))
                return Task.FromResult(kvp.Value);
        }

        return Task.FromResult(string.Empty);
    }
}
