# Contributing to Hermes

This document provides guidelines for developing and contributing to the Hermes
project.

## Table of Contents

- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Project Conventions](#project-conventions)
- [Adding Features](#adding-features)
- [Testing](#testing)
- [Common Development Tasks](#common-development-tasks)
- [Troubleshooting Development Issues](#troubleshooting-development-issues)
- [Best Practices](#best-practices)

## Development Setup

### Prerequisites

Ensure you have all required tools installed (see
[README.md](./README.md#prerequisites)):
- Node.js (v18+)
- pnpm (v8+)
- Rust (latest stable)
- Platform-specific build tools

### Initial Setup

```bash
# Clone and install
git clone git@github.com:hamaluik/hermes.git
cd hermes
pnpm install

# Verify installation
pnpm check                   # TypeScript
cd src-tauri && cargo check  # Rust
```

## Development Workflow

### Starting Development

```bash
# Start the application in dev mode (hot-reload enabled)
pnpm tauri dev
```

This command:
1. Starts Vite dev server (frontend)
2. Launches Tauri application (desktop wrapper)
3. Enables hot-reload for frontend changes
4. Automatically rebuilds Rust on backend changes

### Type Checking

Run TypeScript checks without building:

```bash
pnpm check          # One-time check
pnpm check:watch    # Continuous checking
```

### Building for Production

```bash
pnpm tauri build
```

Outputs to `src-tauri/target/release/bundle/`

## Coding Standards

### TypeScript/JavaScript

- Use TypeScript for all new code
- Prefer `const` over `let`, avoid `var`
- Use arrow functions for callbacks
- Use async/await over raw Promises
- Add JSDoc comments for exported functions

**Example:**
```typescript
/**
 * Sends an HL7 message via MLLP protocol
 * @param host - Target host address
 * @param port - Target port number
 * @param message - HL7 message content
 * @param timeout - Connection timeout in milliseconds
 */
export async function sendMessage(
  host: string,
  port: number,
  message: string,
  timeout: number
): Promise<void> {
  await invoke('send_message', { host, port, message, timeout });
}
```

#### Naming Conventions

- Variables/functions: `camelCase`
- Types/interfaces: `PascalCase`
- Constants: `UPPER_SNAKE_CASE`
- Files: `snake_case.ts` or `kebab-case.svelte`

### Svelte

```svelte
<script lang="ts">
  // 1. Imports
  import { invoke } from '@tauri-apps/api/core';

  // 2. Props (if any)
  interface Props {
    title: string;
    onClose?: () => void;
  }
  let { title, onClose }: Props = $props();

  // 3. State
  let isOpen = $state(false);

  // 4. Derived state
  let displayTitle = $derived(title.toUpperCase());

  // 5. Functions
  function handleClick() {
    isOpen = true;
  }
</script>

<!-- 6. Template -->
<div>
  <h1>{displayTitle}</h1>
  <button onclick={handleClick}>Open</button>
</div>

<!-- 7. Styles -->
<style>
  div {
    padding: 1rem;
  }
</style>
```

- Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Keep components focused and single-purpose
- Extract reusable logic into separate modules
- Use TypeScript for props interfaces

### Rust

- Follow Rust standard style guidelines
- Use `rustfmt` for automatic formatting
- Run `cargo clippy` for linting
- Add doc comments for public items

**Example:**
```rust
/// Highlights syntax in an HL7 message for display
///
/// # Arguments
/// * `message` - The raw HL7 message text
/// * `schema_cache` - Reference to the schema cache
///
/// # Returns
/// A vector of styled text segments with color information
#[tauri::command]
pub fn syntax_highlight(
    message: String,
    state: State<'_, AppData>,
) -> Result<Vec<TextSegment>, String> {
    let schema_cache = &state.schema_cache;

    // Parse message
    let parsed = hl7_parser::parse(&message)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Apply highlighting
    let segments = apply_syntax_highlighting(&parsed, schema_cache);

    Ok(segments)
}
```

#### Naming Conventions

- Variables/functions: `snake_case`
- Types/traits: `PascalCase`
- Constants: `UPPER_SNAKE_CASE`
- Modules: `snake_case`

#### Error Handling

- Use `Result<T, E>` for fallible operations
- Convert internal errors to `String` for Tauri commands
- Use `color-eyre` for detailed error context in complex functions
- Propagate errors with `?` operator

## Project Conventions

### File Organization

#### Frontend

```
src/
├── routes/              # SvelteKit pages
├── lib/                 # Feature-based organisation
│   ├── communication/   # MLLP send/receive (components + Tauri bridges)
│   ├── editor/          # Message editor (components + bridges)
│   ├── diff/            # Message comparison
│   ├── find_replace/    # Search functionality
│   ├── validation/      # Validation UI and logic
│   ├── settings/        # Settings UI
│   ├── modals/          # Standalone modals
│   ├── shared/          # Cross-feature utilities
│   ├── tabs/            # Tab navigation
│   ├── toolbar/         # Toolbar components
│   ├── forms/           # Form inputs
│   ├── components/      # Generic UI primitives
│   └── icons/           # SVG icon components
└── settings.ts          # Settings persistence
```

#### Backend

```
src-tauri/src/
├── lib.rs               # Entry point
├── menu/                # Native menu system
├── commands/            # Tauri commands (organised by feature)
│   ├── communication/   # send.rs, listen.rs
│   ├── editor/          # cursor.rs, data.rs, syntax_highlight.rs
│   ├── validation/      # validate.rs, diff.rs
│   └── support/         # field_description.rs, schema.rs
├── schema/              # Schema caching system
└── spec/                # HL7 specifications
```

### Module Organization

TypeScript modules should export related functions together, use named exports
(avoid default exports), and stay focused on a single responsibility.

Rust modules should have one module per major feature, with the public API in
`mod.rs` and internal helpers in separate files.

### Naming Conventions for Features

When adding a new feature, create a `src-tauri/src/commands/feature_name/`
directory on the backend and a corresponding `src/lib/feature_name/` directory
on the frontend containing both components and Tauri bridges.

Use `snake_case` for Tauri commands (e.g., `get_field_description`) and
`camelCase` for the TypeScript wrapper functions (e.g., `getFieldDescription`).
For imports, use `$lib/` paths for cross-directory references and relative paths
within the same feature.

## Adding Features

### Adding a New Tauri Command

#### 1. Create Rust command

```rust
// src-tauri/src/commands/my_feature/my_command.rs
#[tauri::command]
pub async fn my_feature_command(
    param: String,
    state: State<'_, AppData>,
) -> Result<String, String> {
    // Implementation
    Ok(format!("Result: {}", param))
}
```

#### 2. Export and register

```rust
// src-tauri/src/commands/my_feature/mod.rs
mod my_command;
pub use my_command::my_feature_command;

// src-tauri/src/commands/mod.rs
pub mod my_feature;
pub use my_feature::my_feature_command;

// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    // ... existing
    my_feature_command,
])
```

#### 3. Create TypeScript bridge (co-located with feature)

```typescript
// src/lib/my_feature/my_feature.ts
import { invoke } from '@tauri-apps/api/core';

export async function myFeatureCommand(param: string): Promise<string> {
  return await invoke('my_feature_command', { param });
}
```

#### 4. Use in component

```svelte
<script lang="ts">
  // relative import within same feature directory
  import { myFeatureCommand } from './my_feature';
  // or use $lib for cross-directory imports
  // import { myFeatureCommand } from '$lib/my_feature/my_feature';

  async function handleClick() {
    const result = await myFeatureCommand('test');
    console.log(result);
  }
</script>
```

### Adding a New UI Component

1. Create component file in appropriate feature directory under `src/lib/`
2. Define props interface using TypeScript
3. Use Svelte 5 runes for state
4. Add styles scoped to component
5. Use relative imports within feature, `$lib/` for cross-directory

### Adding New HL7 Message Types

1. Update `messages.toml` with message structure
2. Schema cache will automatically load on next run
3. Add any custom field descriptions to `src-tauri/src/spec/`

## Testing

### Manual Testing

Currently, Hermes relies on manual testing. When adding features:

1. **Test the happy path**: Verify feature works as expected
2. **Test edge cases**: Empty inputs, invalid data, network failures
3. **Test error handling**: Ensure errors display appropriately
4. **Test cross-platform**: If possible, test on multiple OSes

### Testing Checklist for New Features

- [ ] Feature works with valid inputs
- [ ] Error messages are clear and helpful
- [ ] UI updates reactively
- [ ] No console errors or warnings
- [ ] Performance is acceptable

## Common Development Tasks

### Updating Dependencies

```bash
pnpm update                # frontend
cd src-tauri && cargo update   # backend
```

### Adding a New Dependency

```bash
# frontend
pnpm add package-name
pnpm add -D dev-package-name

# backend
cd src-tauri
cargo add crate-name
cargo add --dev dev-crate-name
```

### Debugging

For frontend debugging, use the browser DevTools (accessible in dev mode), add
`console.log()` statements, or use the Svelte DevTools extension.

For backend debugging, add `log::info!()` or `log::debug!()` statements and view
them in the terminal where `pnpm tauri dev` is running. You can also use the
`dbg!()` macro for quick debugging or attach VS Code's debugger with
rust-analyzer.

### Viewing Logs

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

### Modifying the Schema

1. Edit `messages.toml`
2. Restart application (schema loads at startup)
3. Verify changes with schema query commands

### Working with Settings

Settings are stored in `settings.json` via Tauri store plugin.

| Platform | Location                                       |
|----------|------------------------------------------------|
| macOS    | `~/Library/Application Support/com.hermes.app/`|
| Windows  | `%APPDATA%\com.hermes.app\`                    |
| Linux    | `~/.local/share/com.hermes.app/`               |

To reset settings, delete `settings.json` and restart app.

## Troubleshooting Development Issues

### Hot Reload Not Working

1. Restart dev server
2. Clear browser cache
3. Check for syntax errors in console

### Rust Compilation Errors

```bash
cd src-tauri
cargo clean
cargo build
```

### TypeScript Errors

```bash
pnpm check
```

Fix reported errors before committing.

### Schema Not Loading

- Verify `messages.toml` syntax
- Check logs for parse errors
- Ensure file is in correct location

## Best Practices

### Code Quality

Take advantage of the type systems in both TypeScript and Rust—they're there to
help catch bugs early. When errors happen (and they will), handle them
gracefully rather than letting things crash unexpectedly.

Sprinkle in log statements where they'd help with debugging later, and document
any complex logic or business rules that aren't immediately obvious from reading
the code. Try to keep functions focused on doing one thing well.

### Performance

For any I/O operations, stick with async/await to avoid blocking. Be thoughtful
about Svelte's reactivity—unnecessary re-renders add up. The schema cache in
this project is a good example of caching done right, and you should apply
similar thinking elsewhere: don't load resources until you actually need them.

### Security

Always validate user inputs before trusting them. On the network side, make sure
to validate host and port values before attempting any connections.

### Maintainability

Stick with the naming conventions already established in the project. When you
notice yourself repeating logic, pull it out into a shared function. Keep the
documentation current as you add features, and take a moment to review your own
code before committing—you'll often catch things you missed.

### Git Workflow

Write commit messages that actually describe what changed and why. Something
like feat: add MLLP listen server for receiving messages tells a story, while
update stuff leaves everyone guessing.

Commit in logical chunks—each commit should represent a coherent unit of work.
Work out a branching strategy with your team based on what fits your workflow.

Before you commit, run the checks:

```bash
pnpm check                     # TypeScript checks
cd src-tauri && cargo clippy   # Rust linting
```

### Documentation

When adding features, update README.md if it affects setup or usage,
ARCHITECTURE.md for architectural changes, and CLAUDE.md if it impacts Claude
Code workflows. Add code comments for complex logic.

---

## Questions or Issues?

- Check existing documentation (README, ARCHITECTURE, HELP)
- Search GitHub issues (if applicable)
- Review Tauri docs: https://tauri.app/
- Review Svelte docs: https://svelte.dev/

Remember: Good documentation and clean code are gifts to your future self!
