# Shutdown

The `shutdown` request is sent from Hermes to the extension when it's time for the extension to terminate gracefully.

## Direction

**Hermes → Extension**

## When Shutdown is Sent

Hermes sends `shutdown` when:

| Trigger                        | Description                              |
|--------------------------------|------------------------------------------|
| Hermes is closing              | User quit the application                |
| Extension is disabled          | User disabled the extension in settings  |
| Extension configuration changed| User modified extension settings         |
| Extension reload requested     | User requested extension reload          |

## Request

### Method

```
shutdown
```

### Parameters

```typescript
interface ShutdownParams {
  /** Reason for shutdown (optional, for logging) */
  reason?: "closing" | "disabled" | "reload" | "error";
}
```

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "method": "shutdown",
  "params": {
    "reason": "closing"
  }
}
```

## Response

### Result

```typescript
interface ShutdownResult {
  /** Whether shutdown completed successfully */
  success: boolean;
}
```

### Success Response

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "result": {
    "success": true
  }
}
```

### Error Response

If cleanup fails, you can respond with an error, but Hermes will still terminate the process:

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "error": {
    "code": -32000,
    "message": "Shutdown failed",
    "data": "Could not save state to disk"
  }
}
```

## Timeout

Hermes waits **5 seconds** for the shutdown response.

| Outcome              | Hermes behaviour                           |
|----------------------|--------------------------------------------|
| Response received    | Process allowed to exit naturally          |
| Timeout exceeded     | Process forcibly terminated (SIGKILL)      |
| Error response       | Logged, then process terminated            |

## Shutdown Checklist

When your extension receives `shutdown`:

### 1. Stop Accepting New Work

```python
def handle_shutdown(request_id, reason):
    global accepting_work
    accepting_work = False
```

### 2. Cancel In-Flight Operations

```python
    # cancel any running async tasks
    for task in pending_tasks:
        task.cancel()
```

### 3. Close Open Windows

If your extension has open windows, they will be closed by Hermes. However, you should clean up any associated resources:

```python
    # stop HTTP servers for UI windows
    for server in http_servers:
        server.shutdown()
```

### 4. Release Resources

```python
    # close database connections
    if db_connection:
        db_connection.close()

    # close file handles
    for handle in open_files:
        handle.close()

    # stop background threads
    for thread in background_threads:
        thread.join(timeout=1.0)
```

### 5. Persist State (If Needed)

```python
    # save any state that needs to survive restart
    state = {"last_query": last_query, "preferences": user_prefs}
    with open(state_file, "w") as f:
        json.dump(state, f)
```

### 6. Send Response

```python
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    }
```

### 7. Exit

After sending the response, exit the process:

```python
    sys.exit(0)
```

## Complete Example

```python
import sys
import json
import threading

# global state
http_server = None
db_connection = None
shutdown_event = threading.Event()

def handle_shutdown(request_id, reason):
    """Handle the shutdown request."""

    sys.stderr.write(f"Shutting down (reason: {reason})\n")

    # signal any waiting operations to abort
    shutdown_event.set()

    # stop HTTP server
    if http_server:
        sys.stderr.write("Stopping HTTP server...\n")
        http_server.shutdown()

    # close database
    if db_connection:
        sys.stderr.write("Closing database connection...\n")
        try:
            db_connection.close()
        except Exception as e:
            sys.stderr.write(f"Warning: failed to close database: {e}\n")

    # send response
    write_message({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    })

    # flush output
    sys.stdout.flush()
    sys.stderr.flush()

    # exit
    sys.exit(0)

def main_loop():
    while True:
        msg = read_message()

        if msg["method"] == "shutdown":
            handle_shutdown(
                msg["id"],
                msg.get("params", {}).get("reason", "unknown")
            )
            # handle_shutdown calls sys.exit(), so we won't reach here

        # handle other messages...
```

## Handling Forced Termination

If your extension does not respond to `shutdown` within 5 seconds, Hermes will forcibly terminate it. To handle this gracefully:

### Use Signal Handlers (Unix)

```python
import signal

def signal_handler(signum, frame):
    sys.stderr.write("Received termination signal, cleaning up...\n")
    # minimal cleanup only - you have very limited time
    if db_connection:
        db_connection.close()
    sys.exit(1)

signal.signal(signal.SIGTERM, signal_handler)
signal.signal(signal.SIGINT, signal_handler)
```

### Use atexit (Python)

```python
import atexit

def cleanup():
    sys.stderr.write("atexit cleanup running...\n")
    # cleanup code here

atexit.register(cleanup)
```

## Best Practices

| Practice                        | Why                                        |
|---------------------------------|--------------------------------------------|
| Respond quickly                 | Avoid forced termination                   |
| Log cleanup steps               | Helps debugging if issues occur            |
| Use timeouts for cleanup        | Avoid hanging while waiting for resources  |
| Exit after responding           | Avoid leaving zombie processes             |
| Handle partial cleanup          | Some resources may already be gone         |

## State Persistence

If your extension needs to remember state across restarts:

### Where to Store State

```python
import os

# use HERMES_DATA_DIR if available
data_dir = os.environ.get("HERMES_DATA_DIR", os.path.expanduser("~/.hermes"))
extension_data_dir = os.path.join(data_dir, "extensions", "my-extension")
os.makedirs(extension_data_dir, exist_ok=True)

state_file = os.path.join(extension_data_dir, "state.json")
```

### What to Persist

| Good to Persist           | Avoid Persisting                |
|---------------------------|---------------------------------|
| User preferences          | Session tokens                  |
| Recent search queries     | Temporary file paths            |
| Window positions/sizes    | In-progress operation state     |
| Cached lookup data        | Sensitive credentials           |

## Related Documentation

- [Lifecycle](../lifecycle.md) - Full lifecycle documentation
- [Initialize](initialize.md) - Startup handshake
