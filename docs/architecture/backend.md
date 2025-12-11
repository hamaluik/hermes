# Backend Architecture

The Rust backend provides high-performance HL7 parsing and network operations.

## Unified Managed State

The backend maintains a single `AppData` struct as Tauri managed state. This
unified approach simplifies access patterns and keeps related fields consistent.
Commands that need state receive it through Tauri's dependency injection rather
than managing their own connections to various subsystems.

The state includes the schema cache, which holds HL7 definitions loaded at
startup. Storing the cache here avoids repeated file reads during editing.
Commands query this cache when they need message or segment metadata.

The listener task handle also lives in managed state. Wrapping it in a mutex
allows start and stop commands to coordinate safely. The mutex ensures only one
command modifies the handle at a time, preventing race conditions when users
rapidly toggle listening.

## Long-Running Task Pattern

Operations that run indefinitely, like the MLLP listener, use a
`Mutex<Option<JoinHandle>>` pattern. The `Option` represents whether a task is
running. When the listener starts, it stores `Some(handle)`. When it stops, it
takes the handle out and aborts it, leaving `None`.

This pattern enforces a single-listener constraint at the type level. Starting a
new listener first checks for an existing handle and aborts it, ensuring only
one listener runs at a time. The mutex prevents concurrent modification while
the option provides a clean representation of the "not running" state.

## Menu Item References

Tauri's native menus become immutable after creation, but Hermes needs to
enable and disable menu items based on application state. For instance, the
Save item should be disabled when there are no unsaved changes.

The backend stores references to menu items that need dynamic state. When the
frontend detects a state change, it calls a command to update the menu item.
This keeps menu state synchronised with toolbar buttons, ensuring users see
consistent affordances regardless of whether they use menus or toolbar.

## Extension Host Integration

The extension host manages third-party extensions that communicate over stdin
and stdout using JSON-RPC. Extensions need access to the current editor content
to provide features like custom validation or transformation.

Rather than having extensions request the editor content repeatedly through IPC,
the backend maintains a copy of the editor content in managed state. The
frontend syncs this copy whenever the message changes, using a debounce to avoid
excessive updates during rapid typing. Extensions can then read the content
directly without additional round trips.

The debounce interval of 500 milliseconds protects extensions from keystroke
spam while keeping them reasonably current. Extensions receive notifications
after the debounce, allowing them to react to changes without processing every
character.

## Plugin Selection

Tauri plugins provide specific capabilities that the frontend needs. The
clipboard plugin enables copy and paste operations. The file system plugin
allows reading and writing message files. The dialog plugin provides native
file open and save dialogs. The store plugin persists user settings to
platform-appropriate locations.

Logging helps with debugging, capturing backend events that the frontend can't
observe directly. The persisted scope plugin remembers file access permissions
across sessions, avoiding repeated permission prompts. The opener plugin
launches external applications and URLs when needed. The updater plugin handles
automatic version checking and installation, running background checks
periodically without user intervention.

## Error Handling

Commands return `Result<T, String>` so errors propagate to the frontend for
display. The string type simplifies serialisation across the IPC boundary while
providing human-readable messages.

Internally, the backend uses `color-eyre` for detailed error context. Errors
wrap with context about what operation was being attempted, making debugging
easier. Before crossing the IPC boundary, these rich errors convert to strings
for frontend display.

## Related Documentation

- [Frontend Architecture](frontend.md) — TypeScript bridges that invoke commands
- [Communication Patterns](communication.md) — Command-response and event
  patterns
- [Application Updates](updater.md) — Update detection and installation flow
- [Rust Coding Standards](../development/coding-standards/rust.md) — Error
  handling and defensive patterns
