# Quick Start

## Quick Start

{% stepper %}
{% step %}
### Install NTIX

Download the latest release from [GitHub Releases](https://github.com/Cristiandis/NTIX/releases), install via Scoop, or build from source:

```bash
# Scoop (recommended)
scoop bucket add ntix https://github.com/Cristiandis/scoop-ntix
scoop install ntix
```

```bash
# Build from source
git clone https://github.com/Cristiandis/NTIX.git
cd NTIX
cargo build --release
```

The release artifacts (ZIP and MSI) are built by `cargo-dist`. The binary is at `target/x86_64-pc-windows-gnu/release/ntix.exe`.

Verify the install:

```bash
ntix --version
```
{% endstep %}

{% step %}
### Set up your config

Just run `ntix diff` - NTIX will create a default config at `~/ntix/config.lua`:

```
Created default config at C:\Users\you\ntix\config.lua
Edit it to add your packages, then run ntix diff again.
```

Edit the file to add your packages:

```lua
local options = {
    winget = { enable = true, acceptAgreements = true },
    chocolatey = { enable = true, yes = true },
    scoop = { enable = true, buckets = { "main", "extras" } }
}

local pkgs = {}
pkgs.winget = { "Google.Chrome", "7zip.7zip" }
pkgs.chocolatey = { "ripgrep" }
pkgs.scoop = { "fd", "bat" }

return { options = options, pkgs = pkgs }
```

Only enable the package managers you have installed.
{% endstep %}

{% step %}
### Preview changes

Run `ntix diff` to see what would change without installing anything:

```bash
ntix diff
```

You'll see output like:

```
To install:
  winget: Google.Chrome (latest)
  winget: 7zip.7zip (latest)
  chocolatey: ripgrep (latest)
  scoop: fd (latest)
  scoop: bat (latest)
```

You can also pass a custom config path: `ntix diff ./my-config.lua`
{% endstep %}

{% step %}
### Apply changes

Run `ntix apply` with administrator privileges to install the packages:

```bash
ntix apply
```

{% hint style="warning" %}
`ntix apply` requires administrator privileges. Right-click your terminal and select "Run as Administrator".
{% endhint %}

NTIX will install all listed packages and create a state file to track what it managed.
{% endstep %}
{% endstepper %}

### What's next?

* Pin specific versions to keep critical packages stable
* Manage config files (dotfiles, settings) with `configFiles`
* Set up multiple environments for work and personal use
* Learn about the diff output colors and sections
