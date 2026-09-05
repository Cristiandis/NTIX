# Contributing

## Contributing

### Prerequisites

* [Rust toolchain](https://rustup.rs) (stable) with `cargo`
* Windows 10/11 (required to run the CLI for real; see notes below)
* One or more package managers installed (winget, Chocolatey, Scoop)

### Build

```bash
git clone https://github.com/Cristiandis/NTIX.git
cd NTIX
cargo build --release
```

The binary is produced at `target/x86_64-pc-windows-gnu/release/ntix.exe` (the project targets `x86_64-pc-windows-gnu` by default via `.cargo/config.toml`).

All pull requests should target the `master` branch.

### Run

```bash
cargo run -- diff examples/simple/config.lua
```

Or run the built binary directly:

```bash
./target/x86_64-pc-windows-gnu/release/ntix.exe diff examples/simple/config.lua
```

### Test

```bash
cargo test
```

The test suite has 245 tests covering config loading, diff computation, execution, state management, locking, command building, package manager detection, table parsing, and the real `cmd.exe` command runner.

### Project structure

```
NTIX/
├── src/
│   ├── main.rs                 CLI entry point (clap subcommands, version in help)
│   ├── commands.rs             CLI command handlers and diff rendering
│   ├── lib.rs                  Crate root, public modules
│   ├── config/                 Lua config loading (mlua), incl. config files
│   ├── diff/                   Diff computation (packages + config files)
│   ├── execution/              Package execution and config-file application
│   ├── lock/                   File locking
│   ├── models/                 Data models, options, and config structs
│   ├── package_manager/        Per-manager ops (winget/choco/scoop), command building, table parsing, detection
│   ├── state_management/       State persistence
│   ├── paths.rs                Default paths
│   ├── process_helper.rs       Admin / token membership checks
│   └── hash.rs                 File content hashing
└── tests/                      Integration tests (one file per module)
```

### CI

GitHub Actions runs on pull requests to `master`:

1. Build and run the test suite for `x86_64-pc-windows-gnu` (mingw)
2. Generate code coverage with `cargo llvm-cov` on `x86_64-pc-windows-msvc`
3. Upload coverage to Codecov

Releases are produced with `cargo-dist`, building ZIP and MSI artifacts and publishing the Scoop bucket manifest on tag.

### License

LGPLv2.1. See [LICENSE](https://github.com/Cristiandis/NTIX/blob/master/LICENSE).

Branding assets are CC BY 4.0. See [branding/LICENSE](https://github.com/Cristiandis/NTIX/blob/master/branding/LICENSE).

### Support the Project

If you find this useful, here are a few ways to help out - pick whichever fits:

1. **Contribute** - open a PR, fix a bug, improve the docs, or tackle an [open issue](https://github.com/Cristiandis/NTIX/issues).
2. **Donate** - if the project saves you time or money, consider [sponsoring](https://ko-fi.com/S6S11IXK2X) to help cover dev time.
3. **Spread the word** - star the repo, share it, or mention it to someone who might find it useful.

Every bit helps keep this maintained. Thanks for using it!
