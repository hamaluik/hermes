# Command Execution

The `command/execute` request is sent from Hermes to an extension when a user triggers a command, typically by clicking a toolbar button.

## Direction

**Hermes → Extension**

## Request

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

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "command/execute",
  "params": {
    "command": "myExtension/search"
  }
}
```

## Response

### Result

```typescript
interface CommandExecuteResult {
  /** Whether the command completed successfully */
  success: boolean;

  /** Optional message to display to the user */
  message?: string;
}
```

### Success Response

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "success": true
  }
}
```

### Success with Message

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "success": true,
    "message": "Patient data loaded successfully"
  }
}
```

### Error Response

For command failures, you can either return a success response with `success: false`:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "success": false,
    "message": "Patient not found in database"
  }
}
```

Or return a JSON-RPC error for unexpected failures:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -32000,
    "message": "Command execution failed",
    "data": {
      "command": "myExtension/search",
      "reason": "Database connection lost"
    }
  }
}
```

### When to Use Each Error Style

| Scenario                              | Response Style            |
|---------------------------------------|---------------------------|
| User input invalid                    | `success: false` + message|
| Expected failure (item not found)     | `success: false` + message|
| User cancelled operation              | `success: false` + message|
| Unexpected error (crash, timeout)     | JSON-RPC error            |
| Infrastructure failure                | JSON-RPC error            |

## Behaviour

### Timeout

Hermes waits **30 seconds** for a command response. This allows time for the following:
- User interaction in web UI windows
- Database queries
- Network operations

If the timeout is exceeded, Hermes:
1. Returns an error to the user
2. Does not terminate the extension (it may still be processing)

### During Execution

While handling a command, the extension can make requests to Hermes:

```
Hermes                                    Extension
  │                                           │
  │──── command/execute (id:5) ──────────────>│
  │     {command: "ext/wizard"}               │
  │                                           │
  │                             Open a web UI │
  │<──── ui/openWindow (id:1) ────────────────│
  │                                           │
  │──── openWindow result ───────────────────>│
  │     {windowId: "abc123"}                  │
  │                                           │
  │          ... user interacts with UI ...   │
  │                                           │
  │<──── editor/getMessage (id:2) ────────────│
  │      {format: "hl7"}                      │
  │                                           │
  │──── getMessage result ───────────────────>│
  │     {message: "MSH|^~\\&|..."}            │
  │                                           │
  │<──── editor/patchMessage (id:3) ──────────│
  │      {patches: [...]}                     │
  │                                           │
  │──── patchMessage result ─────────────────>│
  │     {success: true}                       │
  │                                           │
  │<──── command/execute result (id:5) ───────│
  │      {success: true}                      │
```

### Concurrent Commands

Extensions may receive multiple `command/execute` requests concurrently. Each has a unique request ID. Handle them independently.

```python
# Example: concurrent command handling
pending_commands = {}

def handle_message(msg):
    if msg.get("method") == "command/execute":
        command_id = msg["id"]
        # start async handling
        pending_commands[command_id] = asyncio.create_task(
            execute_command(msg["params"]["command"], command_id)
        )
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

def handle_command(request_id, command, args=None):
    """Handle a command/execute request."""

    if command == "myExtension/search":
        return handle_search(request_id)
    elif command == "myExtension/create":
        return handle_create(request_id)
    else:
        # unknown command
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": f"Unknown command: {command}"
            }
        }

def handle_search(request_id):
    """Handle the search command."""

    # 1. Open a web UI for user input
    window_response = send_request("ui/openWindow", {
        "url": "http://localhost:8080/search",
        "title": "Patient Search",
        "width": 600,
        "height": 400
    })

    if "error" in window_response:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "success": False,
                "message": "Failed to open search window"
            }
        }

    # 2. Wait for user to complete search (your web UI handles this)
    # ... web UI sends results via HTTP callback ...

    # 3. Get the current message
    msg_response = send_request("editor/getMessage", {"format": "json"})
    current_message = json.loads(msg_response["result"]["message"])

    # 4. Update the message with search results
    patches = [
        {"path": "PID.5.1", "value": "DOE"},
        {"path": "PID.5.2", "value": "JOHN"},
        {"path": "PID.3.1", "value": "12345"}
    ]

    patch_response = send_request("editor/patchMessage", {"patches": patches})

    if patch_response["result"]["success"]:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "success": True,
                "message": "Patient data loaded"
            }
        }
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "success": False,
                "message": "Failed to update message"
            }
        }
```

## User Feedback

### Message Display

When you include a `message` in the result, Hermes displays it to the user:

- **Success with message:** Shown as a brief notification
- **Failure with message:** Shown as an error dialog

### Progress Indication

While a command is executing:
- The toolbar button that triggered it becomes disabled
- Hermes shows a loading indicator

### No Message

If you don't include a `message`:
- Success: No notification shown
- Failure: Generic "Command failed" message

## Related Documentation

- [Initialize](initialize.md) - Registering commands via toolbar buttons
- [Editor Messages](editor.md) - Reading and modifying messages
- [UI Messages](ui.md) - Opening web UI windows
- [Errors](../errors.md) - Error codes
