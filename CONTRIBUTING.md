# Contributing to Hermes

This document provides guidelines for developing and contributing to the Hermes project. Whether you're returning to the project after time away or joining for the first time, these guidelines will help you understand the development workflow and coding standards.

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

Ensure you have all required tools installed (see [README.md](./README.md#prerequisites)):
- Node.js (v18+)
- pnpm (v8+)
- Rust (latest stable)
- Platform-specific build tools

### Initial Setup

```bash
# Clone and install
git clone <repository-url>
cd hermes
pnpm install

# Verify installation
pnpm check          # TypeScript type checking
cd src-tauri && cargo check  # Rust compilation
```

### IDE Setup

**Recommended IDEs:**
- **VS Code** with extensions:
  - Svelte for VS Code
  - rust-analyzer
  - Tauri
  - Prettier - Code formatter
  - ESLint

**Configuration:**
- TypeScript strict mode is enabled
- Prettier for code formatting
- Use `rustfmt` for Rust code

## Development Workflow

### Starting Development

```bash
# Start the application in dev mode (hot-reload enabled)
pnpm tauri dev
```

This command:
1. Starts Vite dev server (frontend) on port 5173
2. Launches Tauri application (desktop wrapper)
3. Enables hot-reload for frontend changes
4. Automatically rebuilds Rust on backend changes

### Frontend-Only Development

For UI work that doesn't require backend:

```bash
pnpm dev
```

Note: Tauri commands won't work in this mode, but you can iterate faster on UI/UX.

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

**Style:**
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

**Naming Conventions:**
- Variables/functions: `camelCase`
- Types/interfaces: `PascalCase`
- Constants: `UPPER_SNAKE_CASE`
- Files: `snake_case.ts` or `kebab-case.svelte`

### Svelte

**Component Structure:**
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

**Best Practices:**
- Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Keep components focused and single-purpose
- Extract reusable logic into separate modules
- Use TypeScript for props interfaces

### Rust

**Style:**
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

**Naming Conventions:**
- Variables/functions: `snake_case`
- Types/traits: `PascalCase`
- Constants: `UPPER_SNAKE_CASE`
- Modules: `snake_case`

**Error Handling:**
- Use `Result<T, E>` for fallible operations
- Convert internal errors to `String` for Tauri commands
- Use `color-eyre` for detailed error context in complex functions
- Propagate errors with `?` operator

## Project Conventions

### File Organization

**Frontend:**
```
src/
├── routes/              # SvelteKit pages
├── lib/                 # Components
│   ├── components/      # Generic UI components
│   ├── forms/          # Form-specific components
│   └── icons/          # SVG icon components
├── backend/            # Tauri command wrappers
└── settings.ts         # Settings management
```

**Backend:**
```
src-tauri/src/
├── lib.rs              # Entry point
├── commands/           # Tauri commands (one file per feature)
├── schema/             # Schema caching system
└── spec/               # HL7 specifications
```

### Module Organization

**TypeScript modules should:**
- Export related functions together
- Use named exports (avoid default exports)
- Keep modules focused on single responsibility

**Rust modules should:**
- One module per major feature
- Public API in `mod.rs`
- Internal helpers in separate files

### Naming Conventions for Features

When adding a new feature:
1. **Backend**: Create `src-tauri/src/commands/feature_name.rs`
2. **Frontend bridge**: Create `src/backend/feature_name.ts`
3. **UI component**: Create `src/lib/feature_name.svelte`
4. **Tauri command**: Use `snake_case` (e.g., `get_field_description`)
5. **TypeScript function**: Use `camelCase` (e.g., `getFieldDescription`)

## Adding Features

### Adding a New Tauri Command

**Step 1: Create Rust command**

```rust
// src-tauri/src/commands/my_feature.rs
#[tauri::command]
pub async fn my_feature_command(
    param: String,
    state: State<'_, AppData>,
) -> Result<String, String> {
    // Implementation
    Ok(format!("Result: {}", param))
}
```

**Step 2: Export and register**

```rust
// src-tauri/src/commands/mod.rs
pub mod my_feature;
pub use my_feature::my_feature_command;

// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    // ... existing
    my_feature_command,
])
```

**Step 3: Create TypeScript bridge**

```typescript
// src/backend/my_feature.ts
import { invoke } from '@tauri-apps/api/core';

export async function myFeatureCommand(param: string): Promise<string> {
  return await invoke('my_feature_command', { param });
}
```

**Step 4: Use in component**

```svelte
<script lang="ts">
  import { myFeatureCommand } from '$backend/my_feature';

  async function handleClick() {
    const result = await myFeatureCommand('test');
    console.log(result);
  }
</script>
```

### Adding a New UI Component

1. Create component file in appropriate directory
2. Define props interface using TypeScript
3. Use Svelte 5 runes for state
4. Add styles scoped to component
5. Export from `$lib` if needed elsewhere

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
- [ ] Works after application restart (persistence)

### Automated Testing (Future)

Contributions of automated tests are welcome:
- **Frontend**: Vitest for TypeScript logic, Testing Library for components
- **Backend**: Rust unit tests and integration tests

## Common Development Tasks

### Updating Dependencies

**Frontend:**
```bash
pnpm update
```

**Backend:**
```bash
cd src-tauri
cargo update
```

### Adding a New Dependency

**Frontend:**
```bash
pnpm add package-name
pnpm add -D dev-package-name
```

**Backend:**
```bash
cd src-tauri
cargo add crate-name
cargo add --dev dev-crate-name
```

### Debugging

**Frontend:**
- Use browser DevTools (accessible in dev mode)
- Add `console.log()` statements
- Use Svelte DevTools extension

**Backend:**
- Add `log::info!()`, `log::debug!()` statements
- View logs in terminal where `pnpm tauri dev` is running
- Use `dbg!()` macro for quick debugging
- Use VS Code debugger with rust-analyzer

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

**Location:**
- macOS: `~/Library/Application Support/com.hermes.app/`
- Windows: `%APPDATA%\com.hermes.app\`
- Linux: `~/.local/share/com.hermes.app/`

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

1. **Type Safety**: Leverage TypeScript and Rust's type systems
2. **Error Handling**: Always handle errors gracefully
3. **Logging**: Add appropriate log statements for debugging
4. **Comments**: Document complex logic and business rules
5. **Keep Functions Small**: Aim for single responsibility

### Performance

1. **Avoid Blocking**: Use async/await for I/O operations
2. **Minimize Re-renders**: Use Svelte's reactivity efficiently
3. **Cache When Possible**: Schema cache is a good example
4. **Lazy Load**: Don't load resources until needed

### Security

1. **Sanitize Inputs**: Validate all user inputs
2. **Network**: Validate host/port before connecting

### Maintainability

1. **Consistent Naming**: Follow project conventions
2. **DRY Principle**: Extract common logic into functions
3. **Documentation**: Update docs when adding features
4. **Code Reviews**: Review your own code before committing

### Git Workflow

1. **Commit Messages**: Use clear, descriptive messages
   - Good: `feat: add MLLP listen server for receiving messages`
   - Bad: `update stuff`

2. **Commit Frequency**: Commit logical units of work

3. **Branch Strategy**: (Define based on team preferences)

4. **Before Committing**:
   ```bash
   pnpm check              # TypeScript checks
   cd src-tauri && cargo clippy  # Rust linting
   ```

### Documentation

When adding features:

1. Update README.md if it affects setup or usage
2. Update ARCHITECTURE.md for architectural changes
3. Update HELP.md for user-facing features
4. Add code comments for complex logic
5. Update CLAUDE.md if it affects Claude Code workflows

---

## Questions or Issues?

- Check existing documentation (README, ARCHITECTURE, HELP)
- Search GitHub issues (if applicable)
- Review Tauri docs: https://tauri.app/
- Review Svelte docs: https://svelte.dev/

Remember: Good documentation and clean code are gifts to your future self!
