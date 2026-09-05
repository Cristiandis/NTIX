# Options

## Options

Option types in `ntix_rs::models::options`.

### NTIXOptions

Top-level options container.

```rust
pub struct NTIXOptions {
    pub winget: WingetOptions,
    pub chocolatey: ChocoOptions,
    pub scoop: ScoopOptions,
}
```

Each field defaults to a fully disabled (`Default`) value unless configured.

### WingetOptions

```rust
pub struct WingetOptions {
    pub enable: bool,
    pub accept_agreement: bool,
    pub silent: bool,
    pub disable_interactivity: bool,
}
```

| Field | Config key | Type | Default | Description |
|-------|-----------|------|---------|-------------|
| `enable` | `enable` | `bool` | `false` | Enable winget support |
| `accept_agreement` | `acceptAgreements` | `bool` | `false` | Auto-accept source/license agreements |
| `silent` | `silent` | `bool` | `false` | Pass `--silent` |
| `disable_interactivity` | `disableInteractivity` | `bool` | `false` | Pass `--disable-interactivity` |

### ChocoOptions

```rust
pub struct ChocoOptions {
    pub enable: bool,
    pub yes: bool,
    pub force: bool,
    pub ignore_dependencies: bool,
    pub allow_downgrade: bool,
    pub skip_power_shell: bool,
    pub params: Option<String>,
    pub pre: bool,
}
```

| Field | Config key | Type | Default | Description |
|-------|-----------|------|---------|-------------|
| `enable` | `enable` | `bool` | `false` | Enable Chocolatey support |
| `yes` | `yes` | `bool` | `false` | Pass `-y` to skip prompts |
| `force` | `force` | `bool` | `false` | Pass `--force` |
| `ignore_dependencies` | `ignoreDependencies` | `bool` | `false` | Pass `--ignore-dependencies` |
| `allow_downgrade` | `allowDowngrade` | `bool` | `false` | Pass `--allow-downgrade` |
| `skip_power_shell` | `skipPowerShell` | `bool` | `false` | Pass `--skip-scripts` |
| `params` | `params` | `Option<String>` | `None` | Custom `--params` value |
| `pre` | `pre` | `bool` | `false` | Pass `--pre` for prereleases |

### ScoopBucket

```rust
pub struct ScoopBucket {
    pub name: String,
    pub url: Option<String>,
}

impl ScoopBucket {
    pub fn new(name: impl Into<String>) -> Self;
}
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Bucket name (for example `"main"`, `"extras"`) |
| `url` | `Option<String>` | Optional custom bucket URL |

### ScoopOptions

```rust
pub struct ScoopOptions {
    pub enable: bool,
    pub buckets: Vec<ScoopBucket>,
    pub global: bool,
    pub independent: bool,
    pub no_cache: bool,
    pub skip_hash_check: bool,
    pub arch: Option<String>,
}
```

| Field | Config key | Type | Default | Description |
|-------|-----------|------|---------|-------------|
| `enable` | `enable` | `bool` | `false` | Enable Scoop support |
| `buckets` | `buckets` | `Vec<ScoopBucket>` | `[main, extras, versions]` | Buckets to ensure exist |
| `global` | `global` | `bool` | `false` | Pass `-g` for global installs |
| `independent` | `independent` | `bool` | `false` | Pass `-i` |
| `no_cache` | `noCache` | `bool` | `false` | Pass `-k` |
| `skip_hash_check` | `skipHashCheck` | `bool` | `false` | Pass `-s` |
| `arch` | `arch` | `Option<String>` | `None` | Pass `--arch <value>` |
