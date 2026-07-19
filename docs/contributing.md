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

The test suite has 223 tests covering config loading, diff computation, execution, state management, locking, and command building.

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

### Support the Project

If you find this useful, here are a few ways to help out - pick whichever fits:

1. **Contribute** — open a PR, fix a bug, improve the docs, or tackle an [open issue](https://github.com/Cristiandis/NTIX/issues).
2. **Donate** — if the project saves you time or money, consider [sponsoring](https://ko-fi.com/S6S11IXK2X) to help cover dev time.
3. **Spread the word** — star the repo, share it, or mention it to someone who might find it useful.

Every bit helps keep this maintained. Thanks for using it!
