# Roadmap

Released features and future plans for NTIX.

## Released

### Arbitrary Config Files

Manage dotfiles, shell configurations, and system settings declaratively - similar to NixOS Home Manager. Define the desired state of files on disk and NTIX will create or update them:

```lua
configFiles = {
    ["C:/Users/you/.gitconfig"] = "configs/gitconfig"
}
```

Declare them under `configFiles` in any config, preview with `ntix diff -c` and apply with `ntix apply -c`. See [Configuration](../configuration/README.md).

## Upcoming

### Optional Windows Features

Enable and disable Windows optional features like Hyper-V, OpenSSH Server, WSL, and .NET Framework directly from your config:

```lua
features = { "Hyper-V", "OpenSSH" }
```

### Nix-Shells

Temporary environments with specific packages available for your current session. Run `ntix shell config.lua`, get a subshell with your declared packages in PATH, and clean up on exit:

```bash
ntix shell dev-config.lua
# packages available here
exit  # packages cleaned up
```
