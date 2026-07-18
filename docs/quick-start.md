# Quick Start

## Quick Start

{% stepper %}
{% step %}
### Install NTIX

Download the latest release from [GitHub Releases](https://github.com/cristianizzo/NTIX/releases), or build from source:

```bash
git clone https://github.com/cristianizzo/NTIX.git
cd NTIX
dotnet publish src/NTIX.Cli -c Release -r win-x64
```

The published binary will be in `src/NTIX.Cli/bin/Release/net10.0/win-x64/publish/`.
{% endstep %}

{% step %}
### Create a config file

Create a file called `config.lua`:

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
ntix diff config.lua
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
{% endstep %}

{% step %}
### Apply changes

Run `ntix apply` with administrator privileges to install the packages:

```bash
ntix apply config.lua
```

{% hint style="warning" %}
`ntix apply` requires administrator privileges. Right-click your terminal and select "Run as Administrator".
{% endhint %}

NTIX will install all listed packages and create a state file to track what it managed.
{% endstep %}
{% endstepper %}

### What's next?

* Pin specific versions to keep critical packages stable
* Set up multiple environments for work and personal use
* Learn about the diff output colors and sections
