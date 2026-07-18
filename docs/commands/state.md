# state

## ntix state

Display the packages NTIX is currently tracking.

### Usage

```bash
ntix state
```

### Output

```
NTIX State:
  winget: Google.Chrome (latest)
  winget: 7zip.7zip (23.01)
  chocolatey: ripgrep (latest)
  scoop: fd (latest)
```

### State file

Stored at `%LOCALAPPDATA%/ntix/state.json`.

Tracks package ID and version per manager. Used to detect orphans, version drift, and avoid re-installing managed packages.
