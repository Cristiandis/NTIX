# Options Reference

## Options Reference

The `options` table controls how NTIX interacts with each package manager.

### Winget

| Option             | Type    | Default | Description                    |
| ------------------ | ------- | ------- | ------------------------------ |
| `enable`           | boolean | `false` | Enable winget                  |
| `acceptAgreements` | boolean | `false` | Auto-accept license agreements |
| `interactive`      | boolean | `false` | Show installer UI              |

### Chocolatey

| Option   | Type    | Default | Description       |
| -------- | ------- | ------- | ----------------- |
| `enable` | boolean | `false` | Enable Chocolatey |
| `yes`    | boolean | `false` | Skip all prompts  |

### Scoop

| Option    | Type    | Default                          | Description       |
| --------- | ------- | -------------------------------- | ----------------- |
| `enable`  | boolean | `false`                          | Enable Scoop      |
| `buckets` | list    | `{"main", "extras", "versions"}` | Buckets to search |
