using System.Reflection;
using CliFx;

var asm = typeof(Program).Assembly;
var version = asm.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion
    ?? asm.GetName().Version?.ToString()
    ?? "1.0.0";

var app = new CommandLineApplicationBuilder()
    .AddCommandsFromThisAssembly()
    .SetExecutableName("ntix")
    .SetVersion(version)
    .Build();

return await app.RunAsync();