using CliFx;
using CliFx.Binding;
using CliFx.Infrastructure;
using NTIX.Core.Config;
using NTIX.Core.StateManagement;
using NTIX.Core.Diff;
using NTIX.Core.Execution;
using NTIX.Core.Lock;
using NTIX.Core;
using Spectre.Console;
using System.Runtime.Versioning;

namespace NTIX.CLI.Commands;

[Command("apply", Description = "Apply desired state (install/remove packages)")]
public partial class ApplyCommand : ICommand
{
    [CommandParameter(0, Name = "config-path", Description = "Path to configuration file (default: ~/ntix/config.lua)")]
    public string? ConfigPath { get; set; }

    [CommandOption("dry-run", 'd', Description = "Show what would change without applying")]
    public bool DryRun { get; set; }

    [CommandOption("no-gc", Description = "Don't remove packages not in config")]
    public bool NoGc { get; set; }

    [CommandOption("stop-on-failure", Description = "Stop on first package failure instead of continuing")]
    public bool StopOnFailure { get; set; }

    [SupportedOSPlatform("windows")]
    public async ValueTask ExecuteAsync(IConsole console)
    {
        if (!ProcessHelper.IsRunningAsAdmin())
        {
            AnsiConsole.MarkupLine("[red]Error: ntix apply requires administrator privileges.[/]");
            AnsiConsole.MarkupLine("Please re-run in an elevated terminal (Run as Administrator).");
            Environment.ExitCode = 1;
            return;
        }

        var isNew = ConfigPath is null && !File.Exists(ConfigLoader.DefaultConfigPath);
        ConfigPath = ConfigLoader.EnsureDefaultConfig(ConfigPath);
        if (isNew)
        {
            AnsiConsole.MarkupLine($"[green]Created default config at {ConfigPath}[/]");
            AnsiConsole.MarkupLine("Edit it to add your packages, then run [bold]ntix diff[/] again.");
            return;
        }
        var config = ConfigLoader.Load(ConfigPath);
        var state = StateService.LoadState() ?? new NTIX.Core.Models.State();
        var diff = DiffEngine.ComputeDiff(config, state);

        if (NoGc)
            diff.ToRemove.Clear();

        DiffEngine.PrintDiff(diff);

        if (DryRun)
        {
            AnsiConsole.MarkupLine("\n[yellow](Dry run - no changes made)[/]");
            return;
        }

        if (diff.IsEmpty)
            return;

        using var lockFile = new LockFile();
        var statePath = StateService.GetStatePath();
        var success = await ExecutionEngine.ApplyDiffAsync(diff, config.Options, state, statePath, stopOnFailure: StopOnFailure);

        if (success)
        {
            AnsiConsole.MarkupLine("\n[green]Done.[/]");
        }
        else
        {
            AnsiConsole.MarkupLine("\n[red]Some operations failed.[/]");
            Environment.ExitCode = 1;
        }
    }
}

[Command("diff", Description = "Show what would change")]
public partial class DiffCommand : ICommand
{
    [CommandParameter(0, Name = "config-path", Description = "Path to configuration file (default: ~/ntix/config.lua)")]
    public string? ConfigPath { get; set; }

    public async ValueTask ExecuteAsync(IConsole console)
    {
        var isNew = ConfigPath is null && !File.Exists(ConfigLoader.DefaultConfigPath);
        ConfigPath = ConfigLoader.EnsureDefaultConfig(ConfigPath);
        if (isNew)
        {
            AnsiConsole.MarkupLine($"[green]Created default config at {ConfigPath}[/]");
            AnsiConsole.MarkupLine("Edit it to add your packages, then run [bold]ntix diff[/] again.");
            return;
        }
        var config = ConfigLoader.Load(ConfigPath);
        var state = StateService.LoadState() ?? new NTIX.Core.Models.State();
        var diff = DiffEngine.ComputeDiff(config, state);
        DiffEngine.PrintDiff(diff);
    }
}

[Command("state", Description = "Show current NTIX state")]
public partial class StateCommand : ICommand
{
    public async ValueTask ExecuteAsync(IConsole console)
    {
        var state = StateService.LoadState();

        if (state == null)
        {
            AnsiConsole.MarkupLine("[yellow]No state file found.[/]");
            return;
        }

        AnsiConsole.MarkupLine("[bold]NTIX State:[/]");

        if (state.Winget.Count == 0 && state.Chocolatey.Count == 0 && state.Scoop.Count == 0)
        {
            AnsiConsole.MarkupLine("  [dim](empty)[/]");
        }
        else
        {
            foreach (var (id, ver) in state.Winget)
                AnsiConsole.MarkupLine($"  [cyan]winget: {id} ({ver})[/]");

            foreach (var (id, ver) in state.Chocolatey)
                AnsiConsole.MarkupLine($"  [magenta]chocolatey: {id} ({ver})[/]");

            foreach (var (id, ver) in state.Scoop)
                AnsiConsole.MarkupLine($"  [blue]scoop: {id} ({ver})[/]");
        }
    }
}