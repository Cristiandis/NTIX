using Spectre.Console.Cli;
using NTIX.CLI.Commands;

var app = new CommandApp();
app.Configure(config =>
{
    config.SetApplicationName("ntix");
    config.SetApplicationVersion("1.0.0");
    config.AddCommand<ApplyCommand>("apply")
        .WithDescription("Apply desired state (install/remove packages)");
    config.AddCommand<DiffCommand>("diff")
        .WithDescription("Show what would change");
    config.AddCommand<StateCommand>("state")
        .WithDescription("Show current NTIX state");
});

return app.Run(args);