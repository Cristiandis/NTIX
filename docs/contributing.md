# Contributing

## Contributing

### Prerequisites

* [.NET 10 SDK](https://dotnet.microsoft.com/download/dotnet/10.0)
* Windows 10/11
* One or more package managers installed (winget, Chocolatey, Scoop)

### Build

```bash
git clone https://github.com/Cristiandis/NTIX.git
cd NTIX
git checkout dev
dotnet build
```

All pull requests should target the `dev` branch. The `master` branch mirrors `dev` for releases.

### Run

```bash
dotnet run --project src/NTIX.Cli -- diff examples/simple/config.lua
```

### Test

```bash
dotnet test
```

The test suite has 220 tests covering config loading, diff computation, execution, state management, locking, and command building.

### Project structure

```
NTIX/
├── src/
│   ├── NTIX.Cli/          CLI entry point (CliFx commands)
│   └── NTIX.Core/          Core library
│       ├── Config/         Lua config loading
│       ├── Diff/           Diff computation
│       ├── Execution/      Package execution
│       ├── Lock/           File locking
│       ├── Models/         Data models
│       ├── PackageManager/ Package manager abstraction
│       └── StateManagement/ State persistence
└── tests/
    └── NTIX.Tests/         xUnit + FluentAssertions + Moq
```

### CI

GitHub Actions runs on every push:

1. Build (Release)
2. Test with code coverage (AltCover)
3. Upload coverage to Codecov
4. Publish Native AOT binary (`win-x64`)
5. Upload artifact

### License

GPLv3. See [LICENSE](https://github.com/Cristiandis/NTIX/blob/master/LICENSE).

Branding assets are CC BY 4.0. See [branding/LICENSE](https://github.com/Cristiandis/NTIX/blob/master/branding/LICENSE).
