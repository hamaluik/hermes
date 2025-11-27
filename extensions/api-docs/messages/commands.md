# Command Execution

The `command/execute` notification is sent from Hermes to an extension when a user
triggers a command, typically by clicking a toolbar button.

Commands use a **fire-and-forget** model: Hermes sends the notification and the extension
handles it asynchronously without sending a response.

## Direction

**Hermes → Extension** (notification)

## Notification

### Method

```
command/execute
```

### Parameters

```typescript
interface CommandExecuteParams {
  /** The command identifier to execute */
  command: string;
}
```

### Example Notification

```json
{
  "jsonrpc": "2.0",
  "method": "command/execute",
  "params": {
    "command": "myExtension/search"
  }
}
```

Note: There is no `id` field because this is a notification, not a request.

## Response

**No response is expected.** Extensions should not send a response to `command/execute`
notifications.

## Behaviour

### Execution Flow

When a user clicks a toolbar button:

```
User clicks         Hermes sends           Extension
toolbar button  →   command/execute    →   handles command
                    (notification)         asynchronously
```

The extension receives the notification and handles the command in the background.
It can make requests to Hermes (like `editor/getMessage`, `editor/patchMessage`) while
executing the command.

### Example Flow

```
Hermes                                    Extension
  │                                           │
  │──── command/execute (notification) ──────>│
  │     {command: "ext/search"}               │
  │                                           │
  │         ... extension does work ...       │
  │                                           │
  │<──── editor/getMessage (id:1) ────────────│
  │                                           │
  │──── getMessage result ───────────────────>│
  │                                           │
  │<──── editor/patchMessage (id:2) ──────────│
  │                                           │
  │──── patchMessage result ─────────────────>│
  │                                           │
```

### Concurrent Commands

Extensions may receive multiple `command/execute` notifications concurrently. Extensions
should be prepared to handle multiple commands in parallel if needed.

### Logging

Since there's no result mechanism, extensions should use stderr for logging command
progress and errors. This output is captured by Hermes and visible in extension logs.

```python
def handle_command(params):
    command = params.get("command")
    log(f"Executing command: {command}")

    # ... do work ...

    if success:
        log("Command completed successfully")
    else:
        log(f"Command failed: {error_message}")
```

## Command Naming

### Conventions

Use a namespace prefix to avoid conflicts:

| Pattern                     | Example                    |
|-----------------------------|----------------------------|
| `{extensionName}/{action}`  | `patientWizard/search`     |
| `{vendor}.{extension}/{action}` | `acme.labs/lookup`     |

### Reserved Prefixes

Do not use the following prefixes:
- `hermes/` - reserved for Hermes internal commands
- `extension/` - reserved for extension management

## Complete Example

Here's a complete command handler in Python:

```python
import json
import sys

def log(message):
    sys.stderr.write(f"[my-ext] {message}\n")
    sys.stderr.flush()

def handle_command(params):
    """Handle a command/execute notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    # check if we recognise this command
    if command not in ["myExtension/search", "myExtension/create"]:
        log(f"Unknown command: {command}")
        return

    # execute the command directly
    if command == "myExtension/search":
        execute_search()
    elif command == "myExtension/create":
        execute_create()

def execute_search():
    """Execute the search command."""
    log("Starting search...")

    # get the current message
    msg_response = send_request("editor/getMessage", {"format": "json"})
    if "error" in msg_response:
        log(f"Failed to get message: {msg_response['error']['message']}")
        return

    current_message = json.loads(msg_response["result"]["message"])

    # do some work with the message...

    # update the message
    patches = [
        {"path": "PID.5.1", "value": "DOE"},
        {"path": "PID.5.2", "value": "JOHN"}
    ]

    patch_response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in patch_response or not patch_response.get("result", {}).get("success"):
        log("Failed to update message")
        return

    log("Search completed successfully")

def handle_message(msg):
    """Route message to appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    # check if this is a notification (no id field)
    if request_id is None:
        # handle notifications
        if method == "command/execute":
            handle_command(params)
        else:
            log(f"Unknown notification: {method}")
        return None

    # handle requests (with id field)
    if method == "initialize":
        return handle_initialize(request_id, params)
    elif method == "shutdown":
        return handle_shutdown(request_id, params)
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }
```

## Related Documentation

- [Initialize](initialize.md) - Registering commands via toolbar buttons
- [Editor Messages](editor.md) - Reading and modifying messages
- [UI Messages](ui.md) - Opening web UI windows
- [Protocol](../protocol.md) - Notification format
- [Errors](../errors.md) - Error handling
