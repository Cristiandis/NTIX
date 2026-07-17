using CliFx;

var app = new CommandLineApplicationBuilder()
    .AddCommandsFromThisAssembly()
    .SetExecutableName("ntix")
    .SetVersion(typeof(Program).Assembly.GetName().Version?.ToString() ?? "1.0.0")
    .Build();

return await app.RunAsync();