# Communication Patterns

Frontend and backend communicate through Tauri's IPC system using commands and
events. The choice of pattern depends on the operation's characteristics.

## Command-Response Pattern

Synchronous operations use command-response when the frontend needs an immediate
answer and can wait for it. The frontend invokes a command, the backend
processes it, and the frontend receives the result directly.

This pattern suits lookups and calculations where the frontend blocks on the
result before continuing. Getting a field description, parsing a segment for
form display, and validating a message all use command-response. These
operations complete quickly and the frontend needs their output before rendering
the next frame.

## Event-Driven Pattern

Asynchronous operations with progress updates use events when the backend may
emit multiple messages before completion. The frontend sets up listeners before
invoking the command, then reacts to events as they arrive.

Sending an HL7 message demonstrates this pattern. The backend emits log events
as it connects, sends, and receives. The frontend updates a progress display
with each event. Finally, the backend emits a response event with the server's
acknowledgment. This stream of events keeps users informed during operations
that take noticeable time.

## Long-Running Tasks

Operations that run indefinitely spawn background tasks. The MLLP listener
exemplifies this pattern: it starts, runs until stopped, and may emit events
throughout its lifetime.

The frontend invokes a start command that spawns a tokio task and returns
immediately. The task runs independently, emitting events whenever messages
arrive. Later, the frontend invokes a stop command that aborts the task. The
task handle stored in managed state makes this coordination possible.

## Choosing a Pattern

Command-response works when the operation completes quickly and the frontend
needs the result to proceed. If the frontend can't render without the data,
command-response keeps the code simple.

Event-driven suits operations where the frontend benefits from progress updates
or where multiple results arrive over time. Users appreciate feedback during
network operations, and events provide that naturally.

Long-running tasks apply when an operation has no natural completion. The
listener waits indefinitely for incoming messages, so it can't use
command-response. Background tasks with stored handles allow the frontend to
control the lifecycle.

## Menu-Event Bridge

Native menus can't call TypeScript directly. Instead, menu clicks emit events
that the frontend listens to. This bridges the native menu system into the
frontend's event handling, keeping file operation logic in one place regardless
of whether triggered by menu or toolbar.

The Save menu item and the Save toolbar button both trigger the same handler.
The menu emits an event, and the frontend's listener calls the same function the
toolbar button uses. This unification prevents divergent behaviour between menu
and toolbar.

## Backend State Sync

The frontend syncs editor content to the backend so extensions can read it
without IPC round trips. Rather than extensions polling for content, the backend
maintains a current copy. Extensions receive notifications when content changes,
after a debounce period that prevents excessive updates during typing.

This pattern reduces latency for extension features. An extension providing
autocomplete can read the current content directly rather than requesting it
through the frontend. The debounce protects both the extension and the IPC
channel from keystroke-level traffic.

## Related Documentation

- [Frontend Architecture](frontend.md) — TypeScript bridges and event listeners
- [Backend Architecture](backend.md) — Command definitions and state management
- [Networking](networking.md) — MLLP protocol details for send/receive
