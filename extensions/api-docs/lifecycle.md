# Extension Lifecycle

This document describes the complete lifecycle of a Hermes extension, from startup through shutdown.

## Lifecycle Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                       Extension Lifecycle                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────┐    ┌────────────┐    ┌─────────┐    ┌──────────┐ │
│  │  Start   │───>│ Initialize │───>│ Running │───>│ Shutdown │ │
│  │ Process  │    │ Handshake  │    │  State  │    │          │ │
│  └──────────┘    └────────────┘    └─────────┘    └──────────┘ │
│                        │               │                        │
│                        │               │                        │
│                   On failure      On error                      │
│                        │               │                        │
│                        v               v                        │
│                   ┌──────────────────────┐                      │
│                   │   Failed / Killed    │                      │
│                   └──────────────────────┘                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Phase 1: Process Start

When Hermes launches, it starts each enabled extension as a subprocess.

### Process Configuration

Extensions are configured in Hermes settings:

```json
{
  "extensions": [
    {
      "path": "/usr/local/bin/my-extension",
      "args": ["--config", "/path/to/config.json"],
      "env": {
        "MY_EXTENSION_DEBUG": "1"
      },
      "enabled": true
    }
  ]
}
```

| Field     | Type       | Required | Description                              |
|-----------|------------|----------|------------------------------------------|
| `path`    | `string`   | Yes      | Absolute path to the extension executable|
| `args`    | `string[]` | No       | Command-line arguments                   |
| `env`     | `object`   | No       | Additional environment variables         |
| `enabled` | `boolean`  | No       | Whether to start this extension (default: true) |

### Process Environment

Hermes sets the following environment variables for all extensions:

| Variable              | Description                               |
|-----------------------|-------------------------------------------|
| `HERMES_VERSION`      | Version of Hermes (e.g., `1.0.0`)         |
| `HERMES_API_VERSION`  | Extension API version (e.g., `1.0.0`)     |
| `HERMES_DATA_DIR`     | Path to Hermes data directory             |

### What the Extension Should Do

At startup, your extension should:

1. Set up stdin/stdout message handling (see [Protocol](protocol.md))
2. Redirect all logging to stderr
3. Wait for the `initialize` request from Hermes
4. **Do not** send any messages before receiving `initialize`

## Phase 2: Initialize Handshake

Immediately after starting the extension process, Hermes sends an `initialize` request.

### Initialize Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "hermesVersion": "1.0.0",
    "apiVersion": "1.0.0",
    "dataDirectory": "/Users/user/.hermes"
  }
}
```

See [messages/initialize.md](messages/initialize.md) for the complete specification.

### Initialize Response

Your extension must respond with its metadata and capabilities:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "My Extension",
    "version": "1.0.0",
    "description": "Does useful things with HL7 messages",
    "authors": ["Jane Developer <jane@example.com>"],
    "capabilities": {
      "commands": true,
      "schemaProvider": true
    },
    "toolbarButtons": [
      {
        "id": "my-button",
        "label": "My Action",
        "icon": "<svg viewBox=\"0 0 24 24\">...</svg>",
        "command": "myExtension/action"
      }
    ],
    "schema": {
      "segments": {
        "PID": {
          "fields": [
            {
              "field": 5,
              "component": 1,
              "note": "Custom note for this field"
            }
          ]
        }
      }
    }
  }
}
```

### Handshake Timeout

Hermes waits **10 seconds** for the initialize response. If the extension does not respond in time:

1. Hermes marks the extension as **failed**
2. The extension process is terminated
3. Toolbar buttons are not added
4. An error is logged

### Handshake Failure

If the extension responds with an error:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Extension failed to initialize",
    "data": "Missing required configuration file"
  }
}
```

Hermes will:
1. Log the error
2. Terminate the extension process
3. Continue without this extension

## Phase 3: Running State

After successful initialization, the extension enters the **running state**.

### Available Operations

In the running state:

| Direction           | What Can Happen                                           |
|---------------------|-----------------------------------------------------------|
| Hermes → Extension  | `command/execute` when user triggers a command            |
| Extension → Hermes  | `editor/getMessage`, `editor/patchMessage`, etc.          |

### Command Execution Flow

```
User clicks         Hermes sends           Extension          Extension
toolbar button  →  command/execute  →  handles command  →  sends response
                                              │
                                              ├── editor/getMessage
                                              ├── editor/patchMessage
                                              └── ui/openWindow
```

A typical command execution:

1. User clicks a toolbar button
2. Hermes sends `command/execute` with the command ID
3. Extension receives the request
4. Extension may make requests to Hermes (getMessage, patchMessage, etc.)
5. Extension sends the command result back to Hermes
6. Hermes displays success/error to user

### Concurrent Commands

Extensions may receive multiple `command/execute` requests concurrently. Each command has a unique request ID. Your extension should:

- Handle commands independently
- Not assume sequential execution
- Use request IDs to correlate responses

### Extension State

Extensions are responsible for managing their own state. Hermes does not provide state persistence for extensions. To persist state:

- Use the filesystem (write to a known location)
- Store in your own configuration files
- Use the `HERMES_DATA_DIR` environment variable for a suitable location

## Phase 4: Shutdown

Hermes sends a `shutdown` request when:

- Hermes is closing
- The user disables the extension
- The user changes extension configuration

### Shutdown Request

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "method": "shutdown",
  "params": {}
}
```

### Shutdown Response

Extensions should clean up resources and respond:

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "result": {
    "success": true
  }
}
```

### Shutdown Timeout

Hermes waits **5 seconds** for the shutdown response. If the extension does not respond:

1. Hermes forcibly terminates the process (SIGKILL)
2. An error is logged

### Graceful Shutdown Checklist

When your extension receives `shutdown`:

- [ ] Stop accepting new work
- [ ] Complete or cancel in-flight operations
- [ ] Close any open windows (send close requests)
- [ ] Flush any buffered data
- [ ] Release resources (file handles, network connections)
- [ ] Send the shutdown response
- [ ] Exit the process

## Error States

### Extension Crash

If an extension process exits unexpectedly:

1. Hermes detects the process termination
2. Any pending commands receive an error response
3. Toolbar buttons become disabled
4. Hermes logs the crash

**Hermes does not automatically restart crashed extensions.** Users must manually restart or reload extensions.

### Communication Errors

If communication with an extension fails (broken pipe, invalid JSON, etc.):

1. Hermes marks the extension as **failed**
2. Toolbar buttons become disabled
3. An error notification is shown to the user

### Invalid Responses

If an extension sends an invalid response:

1. The corresponding request fails with an error
2. The extension remains running (single bad response doesn't kill it)
3. Repeated errors may indicate a bug in the extension

## Best Practices

### Startup

```python
# Good: Log to stderr, wait for initialize
import sys
sys.stderr.write("Extension starting...\n")

# Wait for initialize request
request = read_message()
if request["method"] != "initialize":
    sys.stderr.write(f"Expected initialize, got {request['method']}\n")
    sys.exit(1)

# Respond to initialize
write_message({
    "jsonrpc": "2.0",
    "id": request["id"],
    "result": { ... }
})
```

### Long-Running Operations

For operations that take a long time, consider:

1. **Immediate acknowledgment** with a "processing" status
2. **Progress updates** via stderr logging (not visible to user, but useful for debugging)
3. **Timeout handling** in your extension

### Resource Management

```python
# Good: Clean up on shutdown
def handle_shutdown(request_id):
    # close HTTP server if running
    if http_server:
        http_server.shutdown()

    # close database connections
    if db_connection:
        db_connection.close()

    # respond to shutdown
    write_message({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    })

    sys.exit(0)
```

## State Diagram

```
┌─────────┐  process   ┌─────────────┐  success   ┌─────────┐
│ Stopped │──started──>│ Initializing│───────────>│ Running │
└─────────┘            └─────────────┘            └─────────┘
     ^                       │                         │
     │                timeout│                         │shutdown
     │                       │                         │
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

## Related Documentation

- [Protocol](protocol.md) - Message framing and transport
- [Initialize Message](messages/initialize.md) - Handshake details
- [Shutdown Message](messages/shutdown.md) - Shutdown details
