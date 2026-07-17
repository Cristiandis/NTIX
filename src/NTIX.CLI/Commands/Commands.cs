using System.Threading;
using System.Threading.Tasks;
using Spectre.Console;
using Spectre.Console.Cli;
using NTIX.Core.Config;
using NTIX.Core.StateManagement;
using NTIX.Core.Diff;
using NTIX.Core.Execution;
using NTIX.Core.Lock;

namespace NTIX.CLI.Commands;

public class ApplySettings : CommandSettings
{
    [CommandArgument(0, "<CONFIG>")]
    public string ConfigPath { get; init; } = "";

    [CommandOption("--dry-run")]
    public bool DryRun { get; init; }

    [CommandOption("--no-gc")]
    public bool NoGc { get; init; }
}

public class ApplyCommand : AsyncCommand<ApplySettings>
{
    protected override async Task<int> ExecuteAsync(CommandContext context, ApplySettings settings, CancellationToken cancellationToken = default)
    {
        try
        {
            var config = ConfigLoader.Load(settings.ConfigPath);
            var state = StateService.LoadState() ?? new NTIX.Core.Models.State();
            var diff = DiffEngine.ComputeDiff(config, state);

            if (settings.NoGc)
                diff.ToRemove.Clear();

            DiffEngine.PrintDiff(diff);

            if (settings.DryRun)
            {
                AnsiConsole.MarkupLine("\n[yellow](Dry run - no changes made)[/]");
                return 0;
            }

            if (diff.IsEmpty)
                return 0;

            using var lockFile = new LockFile();
            var success = ExecutionEngine.ApplyDiff(diff, config.Options, state);
            
            if (success)
            {
                StateService.SaveState(state);
                AnsiConsole.MarkupLine("\n[green]Done.[/]");
                return 0;
            }
            else
            {
                AnsiConsole.MarkupLine("\n[red]Some operations failed.[/]");
                return 1;
            }
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Error: {ex.Message}[/]");
            return 1;
        }
    }
}

public class DiffSettings : CommandSettings
{
    [CommandArgument(0, "<CONFIG>")]
    public string ConfigPath { get; init; } = "";
}

public class DiffCommand : AsyncCommand<DiffSettings>
{
    protected override async Task<int> ExecuteAsync(CommandContext context, DiffSettings settings, CancellationToken cancellationToken = default)
    {
        try
        {
            var config = ConfigLoader.Load(settings.ConfigPath);
            var state = StateService.LoadState() ?? new NTIX.Core.Models.State();
            var diff = DiffEngine.ComputeDiff(config, state);
            DiffEngine.PrintDiff(diff);
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Error: {ex.Message}[/]");
            return 1;
        }
    }
}

public class StateSettings : CommandSettings { }

public class StateCommand : AsyncCommand<StateSettings>
{
    protected override async Task<int> ExecuteAsync(CommandContext context, StateSettings settings, CancellationToken cancellationToken = default)
    {
        try
        {
            var state = StateService.LoadState();
            
            if (state == null)
            {
                AnsiConsole.MarkupLine("[yellow]No state file found.[/]");
                return 0;
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
            return 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Error: {ex.Message}[/]");
            return 1;
        }
    }
}