# Lifecycle

This document explains the conceptual model behind extension lifecycle phases,
state transitions, error handling, and the design decisions around timeouts
and command execution.

## The Four Phases

Every extension moves through four distinct phases during its lifetime. Each
phase has different responsibilities and behaviours.

### 1. Spawn

**Purpose:** Bring the extension process into existence.

Hermes starts the extension as a child process, configuring its environment
and connecting stdio pipes. At this point, the extension is just a running
programme—it hasn't yet communicated with Hermes or declared its capabilities.

**What's happening:**
- Operating system creates a new process
- stdin/stdout/stderr are connected to pipes
- Environment variables are set (HERMES_VERSION, HERMES_API_VERSION, HERMES_DATA_DIR)
- Extension process begins executing

**Extension responsibilities:**
- Set up stdin/stdout handling for JSON-RPC messages
- Redirect all logging to stderr (not stdout)
- Wait for the `initialize` request
- **Do not** send any messages before receiving `initialize`

**Why wait for initialize?** The extension doesn't know what version of Hermes
is running it, what API version is available, or where to store data. The
initialize request provides this context. Sending messages prematurely could
cause protocol errors.

### 2. Initialize (Handshake)

**Purpose:** Establish capabilities and metadata through mutual agreement.

Hermes sends an `initialize` request containing:
- Hermes version
- API version
- Data directory path

The extension responds with:
- Name and version
- Capabilities (command list, schema provider flag)
- Toolbar buttons
- Optional schema overrides

This handshake is a negotiation. Hermes says "I'm version X with API version
Y," and the extension says "I provide these capabilities and need these UI
elements."

**Why a handshake instead of configuration?** Configuration files would
require coordination between Hermes and extensions, file format versioning,
and a discovery mechanism. The handshake is self-describing—everything Hermes
needs to know is in the response.

**Success:** Extension transitions to Running state. Toolbar buttons appear,
schema overrides are merged, commands are registered.

**Failure:** If the extension responds with an error or doesn't respond within
10 seconds, Hermes terminates it and marks it as Failed.

### 3. Running

**Purpose:** Perform actual work in response to user actions.

In the running state, the extension is fully operational. It can receive
command notifications and make requests to Hermes.

**Bidirectional communication:**

| Direction           | Message Types                                        |
|---------------------|------------------------------------------------------|
| Hermes → Extension  | `command/execute` notifications                      |
| Extension → Hermes  | `editor/getMessage`, `editor/patchMessage`, `editor/setMessage`, `ui/openWindow`, `ui/closeWindow`, `ui/showMessage`, etc. |

**Fire-and-forget commands:** When a user clicks a toolbar button, Hermes
sends a `command/execute` notification (no `id` field). The extension handles
it asynchronously without responding. See the "Command Execution Model"
section below.

**State management:** Extensions are responsible for their own state. Hermes
doesn't provide persistence or state synchronisation. Extensions can use the
filesystem, databases, or in-memory structures to track state across command
invocations.

**Concurrent commands:** Extensions may receive multiple `command/execute`
notifications before completing previous ones. Extensions should either:
- Handle commands concurrently (threads, async/await)
- Queue commands and process them sequentially
- Reject new commands while busy (via stderr logging)

The choice depends on the extension's design and the nature of its operations.

### 4. Shutdown

**Purpose:** Clean up resources and terminate gracefully.

Hermes sends a `shutdown` request when:
- Hermes is closing
- The user disables the extension
- The user changes extension configuration (triggering reload)

The extension should:
1. Stop accepting new work
2. Complete or cancel in-flight operations
3. Close any open windows
4. Flush buffered data
5. Release resources (file handles, network connections, database connections)
6. Respond with `{"success": true}`
7. Exit the process

**Timeout:** Hermes waits 5 seconds for the response. If the extension doesn't
respond, Hermes sends SIGKILL to forcibly terminate the process.

**Why 5 seconds?** Most cleanup operations complete quickly (< 100ms). Five
seconds is generous for network connection teardown, file flushing, or
database commits. Longer timeouts delay application shutdown unnecessarily.

**Graceful vs forced:** Graceful shutdown gives extensions a chance to:
- Save work in progress
- Close database transactions cleanly
- Notify remote services of disconnection
- Avoid corrupting files

Forced shutdown (SIGKILL) provides no cleanup opportunity. Extensions should
respond promptly to shutdown requests to avoid being killed.

## State Transitions

```
┌─────────┐  process   ┌──────────────┐  success   ┌─────────┐
│ Stopped │──started──>│ Initialising │───────────>│ Running │
└─────────┘            └──────────────┘            └─────────┘
     ^                       │                         │
     │                timeout│                         │shutdown
     │                 error │                         │
     │                       v                         v
     │                 ┌──────────┐              ┌──────────┐
     │                 │  Failed  │              │ Shutting │
     │                 └──────────┘              │   Down   │
     │                       │                   └──────────┘
     │                       │                         │
     │   crash/error         │                         │response
     ├───────────────────────┘                         │
     │                                                 │
     └─────────────────────────────────────────────────┘
```

### Stopped → Initialising

**Trigger:** User enables extension, Hermes starts up, or user reloads
extensions.

**What happens:** Hermes spawns the extension process and sends the
`initialize` request.

**Timeout:** 10 seconds for the initialize response.

### Initialising → Running

**Trigger:** Extension responds successfully to `initialize`.

**What happens:**
- Hermes registers the extension's commands
- Toolbar buttons are added to the UI
- Schema overrides are merged into the schema cache
- Extension can now receive commands and make requests

### Initialising → Failed

**Trigger:** Extension returns an error response, doesn't respond within 10
seconds, or crashes during initialisation.

**What happens:**
- Hermes logs the error
- Extension process is terminated (if still running)
- No toolbar buttons are added
- User sees error in extension settings

**Recovery:** User must fix the extension and reload. Hermes doesn't
automatically retry.

### Running → Shutting Down

**Trigger:** Hermes sends `shutdown` request.

**What happens:**
- Extension stops accepting new commands
- Extension cleans up resources
- Extension responds to shutdown request
- Extension exits

**Timeout:** 5 seconds for the shutdown response.

### Running → Stopped

**Trigger:** Extension responds to `shutdown` successfully, or shutdown times
out and extension is killed.

**What happens:**
- Extension process terminates
- Toolbar buttons are removed
- Pending requests receive errors
- Schema overrides are removed from cache

### Running → Failed

**Trigger:** Extension process crashes or exits unexpectedly without a
shutdown request.

**What happens:**
- Hermes detects process termination
- Toolbar buttons become disabled
- User sees error notification
- Extension logs show crash details (if available)

**Recovery:** User must restart Hermes or reload extensions.

### Shutting Down → Stopped

**Trigger:** Extension responds to shutdown or timeout expires.

**What happens:**
- Process terminates normally (exit) or forcibly (SIGKILL)
- Resources are released
- Extension returns to Stopped state

## Error Handling Philosophy

### Errors Don't Cascade

A single error doesn't kill the extension. If an extension returns an invalid
response to `editor/getMessage`, that request fails but the extension remains
in the Running state. It can continue handling other commands.

**Why?** Extensions might have bugs that affect specific operations. A bug in
window handling shouldn't break message editing. Hermes isolates errors to the
operation that failed.

**Exception:** Repeated errors, process crashes, or complete communication
breakdown cause Hermes to mark the extension as Failed.

### Errors Are Logged

All errors—protocol errors, request failures, timeouts, crashes—are logged to
the extension logs. Users can view the logs to diagnose issues.

**Why visible logging?** Extension developers need to debug their code.
Users need to understand why an extension isn't working. Silent failures
create support burdens. Explicit logs make troubleshooting tractable.

### No Automatic Restart

Hermes doesn't automatically restart crashed extensions. The user must
manually reload extensions or restart Hermes.

**Why not auto-restart?** If an extension crashes due to a bug, restarting it
will likely trigger the same crash. This creates a crash loop that wastes
resources and spams logs. Manual intervention gives the user control and
prevents runaway processes.

## Timeout Rationale

### Initialize: 10 Seconds

**Why 10 seconds?** Extensions might need to:
- Connect to a database
- Load configuration files
- Perform initial HTTP requests
- Validate credentials
- Start an HTTP server

These operations can take a few seconds, especially on slow disks or networks.
Ten seconds provides ample time while still bounding startup latency.

**Too short:** Extensions that perform network lookups or database migrations
at startup would fail.

**Too long:** Hermes startup would be sluggish if every extension took the
full timeout.

### Shutdown: 5 Seconds

**Why 5 seconds?** Shutdown operations are typically faster than startup:
- Close database connections (< 100ms)
- Stop HTTP servers (< 1s)
- Flush buffers (< 100ms)
- Close file handles (instant)

Five seconds is generous for these operations while ensuring Hermes doesn't
hang when closing.

**Too short:** Extensions might not finish writing buffered data or closing
transactions cleanly.

**Too long:** Users waiting for Hermes to close would experience noticeable
delays.

### Commands: No Timeout

Commands use a fire-and-forget model, so there's no timeout. Extensions can
take as long as they need.

**Why no timeout?** Different commands have different performance
characteristics:

- Simple field extraction: < 100ms
- Message validation: < 1s
- Database query: 1-5s
- External API call: 5-30s
- Batch processing: minutes

A single timeout value can't accommodate this range. Too short penalises
legitimate use cases; too long delays error detection.

**How users know progress:** Extensions log to stderr and update the UI
(windows, dialogs, message edits). Users see activity even if commands take
time.

## Command Execution Model

Commands use **fire-and-forget** notifications: Hermes sends `command/execute`
without an `id` field, and the extension doesn't respond.

### Why Fire-and-Forget?

**Traditional request-response:**
```
User clicks button
    ↓
Hermes sends command/execute (request with id)
    ↓
Hermes waits for response (with timeout)
    ↓
Extension processes command
    ↓
Extension sends result (success or error)
    ↓
Hermes displays result to user
```

**Problems:**
1. **Timeout selection:** How long should Hermes wait? 5 seconds? 30 seconds?
   Different commands have vastly different performance characteristics.

2. **Progress tracking:** If a command takes 30 seconds, how does Hermes show
   progress? It would need complex result structures with partial updates.

3. **Error propagation:** What does an error response mean? The command failed
   entirely? Partially? Should Hermes show an error dialog? Log it? Both?

**Fire-and-forget:**
```
User clicks button
    ↓
Hermes sends command/execute (notification, no id)
    ↓
Extension handles asynchronously
    ↓
Extension logs progress to stderr
    ↓
Extension updates UI (windows, dialogs, message edits)
```

**Benefits:**
1. **No timeout needed:** Extensions work at their own pace.
2. **Natural progress communication:** Extensions use logs, windows, and
   dialogs to show what's happening.
3. **Simpler protocol:** No result tracking, no complex error structures.

### How Extensions Communicate Results

Since commands don't have responses, extensions use side effects:

**1. stderr logging:**
```python
sys.stderr.write("Starting patient lookup...\n")
# ... perform lookup ...
sys.stderr.write("Found 3 matching patients\n")
```

Users can view logs in the Extension Logs modal to see what happened.

**2. Message edits:**
```python
# update message with lookup results
hermes.patch_message([
    {"path": "PID.5.1", "value": "DOE"},
    {"path": "PID.5.2", "value": "JOHN"}
])
```

The user sees the editor update immediately.

**3. Dialogs:**
```python
# show completion message
hermes.show_message(
    message="Patient lookup complete. Found 3 matches.",
    level="info"
)
```

The user receives explicit feedback.

**4. Windows:**
```python
# open a window with results
window_id = hermes.open_window("http://localhost:8000/results")
```

The extension shows a rich UI in a dedicated window.

### Concurrent Command Handling

Extensions may receive `command/execute` notifications while previous commands
are still running. How should extensions handle this?

**Option 1: Queue commands**
```python
command_queue = []

def handle_command_execute(command):
    command_queue.append(command)
    if not currently_processing:
        process_next_command()
```

**When to use:** Commands modify shared state or resources, and concurrent
execution would cause conflicts.

**Option 2: Process concurrently**
```python
def handle_command_execute(command):
    threading.Thread(target=execute_command, args=[command]).start()
```

**When to use:** Commands are independent and can run in parallel.

**Option 3: Reject duplicates**
```python
def handle_command_execute(command):
    if currently_processing:
        sys.stderr.write("Extension busy, ignoring command\n")
        return
    execute_command(command)
```

**When to use:** Commands are resource-intensive and running multiple
instances would degrade performance.

The choice depends on the extension's design. Hermes doesn't enforce a
particular model.

## Conclusion

The lifecycle phases—spawn, initialize, running, shutdown—provide a clear
structure for extension development. Each phase has well-defined
responsibilities and transitions.

The design prioritises:

1. **Clarity:** Extensions know what state they're in and what's expected.
2. **Fault tolerance:** Errors are isolated and don't cascade.
3. **Flexibility:** Extensions control their own state and concurrency models.
4. **Responsiveness:** Timeouts are bounded but generous.

Understanding these concepts helps extension developers build robust,
well-behaved extensions that integrate smoothly with Hermes.
