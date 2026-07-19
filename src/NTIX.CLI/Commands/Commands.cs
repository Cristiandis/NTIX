using CliFx;
using CliFx.Binding;
using CliFx.Infrastructure;
using NTIX.Core.Config;
using NTIX.Core.StateManagement;
using NTIX.Core.Diff;
using NTIX.Core.Execution;
using NTIX.Core.Lock;
using NTIX.Core;
using NTIX.Core.Models;
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

    [CommandOption("adopt", 'a', Description = "Adopt already-installed packages into NTIX state")]
    public bool Adopt { get; set; }

    [CommandOption("upgrade", 'u', Description = "Check for and apply available upgrades")]
    public bool Upgrade { get; set; }

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

        DiffResult diff = null!;
        var configFileName = Path.GetFileName(ConfigPath);
        await AnsiConsole.Status()
            .Spinner(Spinner.Known.Dots)
            .SpinnerStyle(Style.Parse("yellow"))
            .StartAsync($"[bold]{configFileName}[/]", async ctx =>
            {
                var progress = new Progress<string>(s => ctx.Status($"[dim]{s}[/]"));
                diff = await DiffEngine.ComputeDiffAsync(config, state, progress: progress, adoptMode: Adopt, upgradeMode: Upgrade);
            });

        var tree = CommandsHelper.BuildDiffTree(configFileName, config, diff);
        AnsiConsole.Write(tree);

        foreach (var w in diff.Warnings)
            AnsiConsole.MarkupLine($"[yellow]Warning: {Markup.Escape(w)}[/]");

        if (NoGc)
            diff.ToRemove.Clear();

        if (DryRun)
        {
            AnsiConsole.MarkupLine("\n[yellow](Dry run - no changes made)[/]");
            return;
        }

        if (diff.IsEmpty)
            return;

        using var lockFile = new LockFile();
        var statePath = StateService.GetStatePath();
        var success = await ExecutionEngine.ApplyDiffAsync(
            diff, config.Options, state, statePath,
            stopOnFailure: StopOnFailure,
            onOutput: Console.WriteLine,
            onError: msg => AnsiConsole.MarkupLine($"[red]{Markup.Escape(msg)}[/]"));

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

    [CommandOption("adopt", 'a', Description = "Show packages that would be adopted")]
    public bool Adopt { get; set; }

    [CommandOption("upgrade", 'u', Description = "Check for and apply available upgrades")]
    public bool Upgrade { get; set; }

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

        DiffResult diff = null!;
        var configFileName = Path.GetFileName(ConfigPath);
        await AnsiConsole.Status()
            .Spinner(Spinner.Known.Dots)
            .SpinnerStyle(Style.Parse("yellow"))
            .StartAsync($"[bold]{configFileName}[/]", async ctx =>
            {
                var progress = new Progress<string>(s => ctx.Status($"[dim]{s}[/]"));
                diff = await DiffEngine.ComputeDiffAsync(config, state, progress: progress, adoptMode: Adopt, upgradeMode: Upgrade);
            });

        var tree = CommandsHelper.BuildDiffTree(configFileName, config, diff);
        AnsiConsole.Write(tree);

        foreach (var w in diff.Warnings)
            AnsiConsole.MarkupLine($"[yellow]Warning: {Markup.Escape(w)}[/]");
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

internal static class CommandsHelper
{
    private static readonly Dictionary<string, string> SourceMarkup = new(StringComparer.OrdinalIgnoreCase)
    {
        ["winget"] = "darkmagenta",
        ["chocolatey"] = "blue",
        ["scoop"] = "magenta",
    };

    public static Tree BuildDiffTree(string configFileName, NTIXConfig config, DiffResult diff)
    {
        var tree = new Tree($"[bold]{configFileName}[/]");

        if (config.Imports.Count > 0)
        {
            var importsNode = tree.AddNode("[dim]imports[/]");
            AddImportChildren(importsNode, config.Imports);
        }

        if (diff.ToInstall.Count > 0)
        {
            var node = tree.AddNode($"[green]\u2191 To install ({diff.ToInstall.Count})[/]");
            AddGroupedBySource(node, diff.ToInstall, "green", showVersion: true);
        }

        if (diff.ToUpgrade.Count > 0)
        {
            var node = tree.AddNode($"[yellow]\u2191 To upgrade ({diff.ToUpgrade.Count})[/]");
            AddGroupedBySourceWithVersions(node, diff.ToUpgrade);
        }

        if (diff.ToAdopt.Count > 0)
        {
            var node = tree.AddNode($"[cyan]\u271a To adopt ({diff.ToAdopt.Count})[/]");
            AddGroupedBySource(node, diff.ToAdopt, "cyan", showVersion: true);
        }

        if (diff.ToSkip.Count > 0)
            tree.AddNode($"[dim]\u2713 Already managed ({diff.ToSkip.Count})[/]");

        if (diff.ToRemove.Count > 0)
        {
            var node = tree.AddNode($"[red]\u2717 Orphans ({diff.ToRemove.Count})[/]");
            AddGroupedBySource(node, diff.ToRemove, "red", showVersion: false);
        }

        if (diff.IsEmpty && !diff.HasError)
            tree.AddNode("[dim]Nothing to do.[/]");

        return tree;
    }

    private static void AddImportChildren(TreeNode parent, List<ImportNode> imports)
    {
        foreach (var import in imports)
        {
            if (import.Children.Count > 0)
            {
                var node = parent.AddNode($"[dim]{import.Path}[/]");
                AddImportChildren(node, import.Children);
            }
            else
            {
                parent.AddNode($"[dim]{import.Path}[/]");
            }
        }
    }

    private static void AddGroupedBySource(TreeNode parent, List<PackageSpec> packages, string color, bool showVersion)
    {
        var grouped = packages.GroupBy(p => p.Source).OrderBy(g => g.Key);

        foreach (var group in grouped)
        {
            var sourceColor = SourceMarkup.TryGetValue(group.Key, out var c) ? c : "white";
            var count = group.Count();

            if (count == 1)
            {
                var pkg = group.First();
                var version = showVersion && pkg.Version != null ? $" ({pkg.Version})" : "";
                parent.AddNode($"[{sourceColor}]{group.Key}: {pkg.Id}{version}[/]");
            }
            else
            {
                var sourceNode = parent.AddNode($"[{sourceColor}]{group.Key} ({count})[/]");
                foreach (var pkg in group.OrderBy(p => p.Id))
                {
                    var version = showVersion && pkg.Version != null ? $" ({pkg.Version})" : "";
                    sourceNode.AddNode($"[{sourceColor}]{pkg.Id}{version}[/]");
                }
            }
        }
    }

    private static void AddGroupedBySourceWithVersions(TreeNode parent, List<PackageSpec> packages)
    {
        var grouped = packages.GroupBy(p => p.Source).OrderBy(g => g.Key);

        foreach (var group in grouped)
        {
            var sourceColor = SourceMarkup.TryGetValue(group.Key, out var c) ? c : "white";
            var count = group.Count();

            if (count == 1)
            {
                var pkg = group.First();
                parent.AddNode($"[{sourceColor}]{group.Key}: {pkg.Id} \u2192 {pkg.Version}[/]");
            }
            else
            {
                var sourceNode = parent.AddNode($"[{sourceColor}]{group.Key} ({count})[/]");
                foreach (var pkg in group.OrderBy(p => p.Id))
                {
                    sourceNode.AddNode($"[{sourceColor}]{pkg.Id} \u2192 {pkg.Version}[/]");
                }
            }
        }
    }
}
