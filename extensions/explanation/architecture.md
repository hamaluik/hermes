# Architecture

This document explains the architectural decisions behind the Hermes extension
system, including why extensions run as separate processes, why JSON-RPC 2.0
was chosen, and the trade-offs involved.

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                           Hermes                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐  ┌───────────┐  │
│  │ Toolbar │  │ Editor  │  │ Schema Cache│  │  Windows  │  │
│  └────┬────┘  └────┬────┘  └──────┬──────┘  └─────┬─────┘  │
│       │            │              │               │         │
│       └────────────┴──────────────┴───────────────┘         │
│                            │                                │
│                    ┌───────┴───────┐                        │
│                    │Extension Host │                        │
│                    └───────┬───────┘                        │
└────────────────────────────┼────────────────────────────────┘
                             │ stdio (JSON-RPC 2.0)
              ┌──────────────┼──────────────┐
              │              │              │
         ┌────┴────┐    ┌────┴────┐    ┌────┴────┐
         │Extension│    │Extension│    │Extension│
         │    A    │    │    B    │    │    C    │
         └─────────┘    └─────────┘    └─────────┘
```

Extensions communicate with Hermes through a clearly defined boundary: stdio
pipes carrying JSON-RPC 2.0 messages. This separation creates fault isolation,
language flexibility, and security boundaries.

## Why Separate Processes?

Extensions run as independent operating system processes rather than as
in-process plugins (like shared libraries or embedded scripting languages).
This architecture provides several critical benefits.

### Fault Isolation

When an extension crashes, panics, or encounters an unrecoverable error, it
cannot take down Hermes. The extension process terminates independently, and
Hermes detects the termination and marks the extension as failed.

This is crucial for stability. A bug in third-party extension code—a null
pointer dereference, an infinite loop, a memory leak—affects only that
extension. The user can continue working with other extensions and core Hermes
functionality.

**In-process alternative:** If extensions were loaded as dynamic libraries
(`.so`, `.dll`, `.dylib` files), a crash in extension code would crash the
entire application. The operating system makes no distinction between a
segfault in your code versus library code loaded into your address space.

### Language Agnostic

Extensions can be written in any programming language that can:
1. Read and write JSON
2. Communicate over stdin/stdout

This includes Python, JavaScript/Node.js, Go, Rust, Ruby, Java, C++, or even
shell scripts. Extension authors choose the tools they know and that best fit
their problem domain.

**Why this matters:** Different tasks favour different languages. A data
transformation extension might be cleanest in Python with its rich ecosystem.
A performance-critical parser might be best in Rust. A quick automation script
might use bash or Ruby. The extension system doesn't constrain these choices.

**In-process alternative:** Plugin systems that load shared libraries restrict
extensions to languages with C FFI compatibility (C, C++, Rust, Go). Scripting
languages like Python or JavaScript would require embedding an interpreter,
significantly increasing complexity.

### Security Boundary

Extensions run with their own operating system permissions and cannot directly
access Hermes internals. They can only interact through the defined API
surface. If an extension is compromised or malicious, it has access only to:

- What it can request through the API (current message, file dialogs, etc.)
- Files and resources available to its own user permissions

Hermes doesn't expose its configuration, schema cache, or internal state
directly. Extensions must make explicit requests, which Hermes can validate,
log, or deny.

**Practical impact:** An extension cannot:
- Read Hermes configuration files directly
- Modify the schema cache except through defined override mechanisms
- Access other extensions' state
- Interfere with other extensions' communication with Hermes

### Independent Lifecycle

Each extension has its own startup, running, and shutdown phases. Extensions
can be started, stopped, enabled, disabled, or reloaded independently without
restarting Hermes.

When a user adds a new extension or updates an extension's configuration,
Hermes can spawn just that extension process. When an extension misbehaves,
the user can disable it without losing work or restarting.

## Why JSON-RPC 2.0?

JSON-RPC 2.0 provides a standardised, bidirectional request-response protocol
that's well-suited for extension communication.

### Bidirectional Communication

Extensions need to:
- Receive commands from Hermes (toolbar button clicks)
- Make requests to Hermes (get current message, patch fields, open windows)

JSON-RPC handles both directions symmetrically. Either side can send requests
and receive responses. This is more flexible than simple message passing or
command-driven protocols.

**Example flow:**
1. User clicks toolbar button
2. Hermes sends `command/execute` notification to extension
3. Extension sends `editor/getMessage` request to Hermes
4. Hermes responds with message content
5. Extension processes data
6. Extension sends `editor/patchMessage` request
7. Hermes responds with success confirmation

### Typed Errors

JSON-RPC defines a standard error structure with error codes, messages, and
optional data. This provides consistent error handling across the entire
API.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "Field 'format' must be one of: hl7, json, yaml, toml"
  }
}
```

Extensions can distinguish between different error conditions programmatically
using error codes, while still providing human-readable messages.

### Request-Response Correlation

The `id` field in JSON-RPC messages allows responses to be matched with
requests, even when multiple requests are in flight simultaneously. This
enables concurrent operations without complex synchronisation.

An extension can send three requests in rapid succession and handle the
responses as they arrive, out of order. The `id` field ensures each response
is routed to the correct handler.

### Well-Established Pattern

JSON-RPC 2.0 is the foundation of the Language Server Protocol (LSP), used by
VS Code, Neovim, and many other editors. Developers familiar with LSP will
recognise the patterns immediately.

This reduces the learning curve and allows reuse of existing libraries.
Many languages have mature JSON-RPC libraries that handle message framing,
request tracking, and error handling.

### Alternatives Considered

**REST over HTTP:** Requires running an HTTP server in Hermes or the
extension, adding complexity and network configuration. Localhost HTTP is
overkill for process-local communication.

**gRPC:** More efficient for high-throughput scenarios, but requires protobuf
definitions and language-specific code generation. The added complexity isn't
justified for extension communication volumes.

**Custom binary protocol:** Maximum efficiency, but requires writing parsers
and serialisers for every language. JSON is universal and debuggable.

## Why stdio for Transport?

Extensions communicate with Hermes through standard input and output streams
rather than network sockets, files, or shared memory.

### Simplicity

Every operating system and programming language provides built-in support for
stdin/stdout. There's no configuration, no port allocation, no file paths to
coordinate. Hermes spawns the extension process with its stdin/stdout
connected to pipes, and communication begins immediately.

**Example (Python):**
```python
import sys
import json

# read from stdin
message = json.loads(sys.stdin.readline())

# write to stdout
sys.stdout.write(json.dumps(response) + "\n")
sys.stdout.flush()
```

### Portability

stdio works identically on macOS, Linux, and Windows. There are no
platform-specific socket APIs, no file system path format differences, no
permission model variations. The same extension code runs everywhere.

### No Network Configuration

Unlike network sockets, stdio requires no port numbers, no localhost
configuration, no firewall rules, no TLS certificates. There's no risk of port
conflicts when running multiple Hermes instances or multiple extensions.

### Separation of Concerns

stdio naturally separates logging from protocol communication:

- **stdout:** JSON-RPC messages only
- **stderr:** Human-readable logs, debugging output, error messages

Hermes captures stderr and displays it in the extension logs modal, making it
easy to debug extensions without polluting the JSON-RPC stream.

### Process Lifetime Binding

When Hermes terminates, the stdio pipes close automatically. Extensions detect
EOF on stdin and know to shut down. When an extension process exits, Hermes
detects the broken pipe and marks the extension as stopped.

This provides automatic cleanup without complex teardown protocols.

### Alternatives Considered

**Unix domain sockets:** Portable across Unix systems but more complex to set
up than stdio. Windows support (named pipes) differs. No meaningful
performance benefit for extension communication volumes.

**TCP sockets:** Requires port allocation, introduces network security
concerns, and creates potential conflicts. Unnecessary when both processes
are local.

**Shared memory:** Maximum performance, but complex to implement correctly
across platforms and languages. Extension communication isn't
performance-critical enough to justify the complexity.

## Fire-and-Forget Commands

When a user clicks a toolbar button, Hermes sends a `command/execute`
notification to the extension. Notifications are JSON-RPC messages without an
`id` field—they don't expect or wait for a response.

### Why Fire-and-Forget?

This design simplifies both sides of the protocol:

**For Hermes:**
- No timeout management needed (no waiting for command completion)
- No result tracking or error propagation required
- No blocking the UI thread while extensions work

**For extensions:**
- Commands can be handled asynchronously at any pace
- Long-running operations don't time out
- No pressure to respond quickly with partial results

### How Extensions Communicate Progress

Since commands don't have responses, how do extensions report errors or
progress?

1. **stderr logging:** Extensions write progress messages to stderr, which
   Hermes captures and displays in the Extension Logs modal. Users can view
   logs to see what extensions are doing.

2. **UI updates:** Extensions can open windows, show dialogs, or modify the
   message. These side effects communicate state changes to the user.

3. **Editor state:** When an extension modifies the message (via
   `editor/patchMessage` or `editor/setMessage`), the user sees the change
   immediately in the editor.

**Example flow:**
```
User clicks button
    ↓
Hermes sends command/execute notification
    ↓
Extension logs to stderr: "Starting analysis..."
    ↓
Extension requests editor/getMessage
    ↓
Extension processes data (30 seconds)
    ↓
Extension logs to stderr: "Analysis complete"
    ↓
Extension requests editor/patchMessage to update fields
    ↓
User sees updated message in editor
```

### Trade-offs

**Benefit:** Simplicity. No timeout tuning, no complex result structures, no
error propagation protocols.

**Cost:** No built-in acknowledgement. Extensions must actively communicate
what they're doing through logs or UI updates. Hermes can't display a progress
bar for command execution without the extension opening a window.

For Hermes extensions, this trade-off is acceptable. Commands typically either
complete quickly (< 1 second) or open a UI that provides its own feedback
(windows, dialogs). Long-running batch operations can log progress to stderr
for users who check the logs.

## Editor State Synchronisation

The backend maintains a copy of the current editor message in
`AppData.editor_message`. This allows editor operations (`getMessage`,
`patchMessage`, `setMessage`) to be handled synchronously by the backend
without round-trips to the frontend.

### Why Backend State?

Extensions run in separate processes and communicate with the backend. Without
backend state, every `editor/getMessage` request would need to:

1. Extension sends `getMessage` to backend
2. Backend sends request to frontend
3. Frontend responds with current message
4. Backend forwards response to extension

This adds latency and complexity. By maintaining state in the backend:

1. Extension sends `getMessage` to backend
2. Backend responds immediately from its copy

### Keeping State Consistent

The frontend calls `sync_editor_message` whenever the editor content changes
(typing, undo, redo, paste, etc.). This keeps the backend copy up to date.

When extensions modify the message:
1. Extension sends `patchMessage` or `setMessage`
2. Backend updates its `editor_message` copy
3. Backend emits `extension-set-message` event to frontend
4. Frontend updates its editor from the event

This maintains a single source of truth while allowing fast backend responses.

### Trade-offs

**Benefit:** Extensions get instant responses to `getMessage` requests. The
extension host remains responsive even with many concurrent requests.

**Cost:** Two copies of the message (frontend and backend) must stay in sync.
Bugs in the sync logic could create inconsistencies. In practice, the sync is
simple and reliable because all updates flow through well-defined paths.

## Conclusion

The architecture prioritises:

1. **Stability:** Extension crashes don't affect Hermes
2. **Flexibility:** Any language, any tool, any approach
3. **Simplicity:** Standard protocols, minimal configuration
4. **Security:** Clear boundaries, explicit API surface

The trade-offs—separate processes, JSON overhead, fire-and-forget commands—are
acceptable costs for these benefits. For Hermes's use case (desktop
application with moderate extension communication), the architecture provides
the right balance.
