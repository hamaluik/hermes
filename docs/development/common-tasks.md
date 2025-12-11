# Common Development Tasks

## Updating Dependencies

```bash
pnpm update                    # frontend
cd src-tauri && cargo update   # backend
```

## Adding a New Dependency

```bash
# frontend
pnpm add package-name
pnpm add -D dev-package-name

# backend
cd src-tauri
cargo add crate-name
cargo add --dev dev-crate-name
```

## Debugging

For frontend debugging, use the browser DevTools (accessible in dev mode), add
`console.log()` statements, or use the Svelte DevTools extension.

For backend debugging, add `log::info!()` or `log::debug!()` statements and view
them in the terminal where `pnpm tauri dev` is running. You can also use the
`dbg!()` macro for quick debugging or attach VS Code's debugger with
rust-analyzer.

## Viewing Logs

Logs are written via `tauri-plugin-log`:

```rust
use log::{info, warn, error};

info!("Application started");
warn!("Connection timeout");
error!("Failed to parse message: {}", err);
```

Logs appear in:
- **Dev mode**: Terminal output
- **Production**: Platform-specific log locations
  - macOS: `~/Library/Logs/com.hermes.app/`
  - Windows: `%APPDATA%\com.hermes.app\logs\`
  - Linux: `~/.local/share/com.hermes.app/logs/`

## Modifying the Schema

1. Edit `messages.toml`
2. Restart application (schema loads at startup)
3. Verify changes with schema query commands

## Working with Settings

Settings are stored in `settings.json` via Tauri store plugin.

| Platform | Location                                        |
|----------|-------------------------------------------------|
| macOS    | `~/Library/Application Support/com.hermes.app/` |
| Windows  | `%APPDATA%\com.hermes.app\`                     |
| Linux    | `~/.local/share/com.hermes.app/`                |

To reset settings, delete `settings.json` and restart app.

## Related Documentation

- [Workflow](workflow.md) — Dev mode and building
- [Troubleshooting](troubleshooting.md) — Common issues
