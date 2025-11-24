# Extension Examples

This section contains complete, working examples of Hermes extensions.

## Available Examples

| Example                     | Description                                    | Complexity |
|-----------------------------|------------------------------------------------|------------|
| [Minimal](minimal.md)       | Hello World extension with basic structure     | Beginner   |
| [Wizard](wizard.md)         | Full wizard with web UI and database lookup    | Advanced   |

## Running Examples

### 1. Create the Extension

Save the example code as an executable file:

```bash
# Python example
chmod +x extension.py

# Or compile for Rust/Go/etc.
cargo build --release
```

### 2. Configure in Hermes

Add to your Hermes settings:

```json
{
  "extensions": [
    {
      "path": "/path/to/extension.py",
      "enabled": true
    }
  ]
}
```

### 3. Restart Hermes

The extension will start automatically and add its toolbar buttons.

## Example Structure

All examples follow this general structure:

```
1. Message handling (read/write JSON-RPC messages)
2. Initialize handler (respond to Hermes handshake)
3. Command handlers (respond to toolbar button clicks)
4. Shutdown handler (clean up resources)
5. Main loop (dispatch messages to handlers)
```

## Language Recommendations

| Language   | Best For                               | Notes                          |
|------------|----------------------------------------|--------------------------------|
| Python     | Quick prototypes, simple extensions    | Easiest way to get started     |
| Rust       | Performance-critical, production use   | Best for complex extensions    |
| Go         | Network-heavy extensions               | Good concurrency support       |
| Node.js    | Web UI extensions, existing JS code    | Async I/O                      |

## Common Patterns

### Logging

Always log to stderr, never stdout:

```python
import sys
sys.stderr.write("Debug: processing command\n")
```

### Request/Response Correlation

Track pending requests by ID:

```python
pending_requests = {}
next_id = 1

def send_request(method, params):
    global next_id
    request_id = next_id
    next_id += 1

    pending_requests[request_id] = asyncio.Event()
    write_message({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params
    })

    # wait for response
    pending_requests[request_id].wait()
    result = pending_requests.pop(request_id)
    return result
```

### Error Handling

Always handle errors gracefully:

```python
try:
    result = do_something()
    return {"success": True}
except Exception as e:
    sys.stderr.write(f"Error: {e}\n")
    return {"success": False, "message": str(e)}
```

## Debugging Tips

### View Extension Output

Extension stderr output appears in Hermes logs. Log locations by platform:

| Platform | Log Location                            |
|----------|-----------------------------------------|
| macOS    | `~/Library/Logs/Hermes/`                |
| Linux    | `$XDG_DATA_HOME/hermes/logs/` or `~/.local/share/hermes/logs/` |
| Windows  | `%APPDATA%\Hermes\logs\`                |

**Note:** The `HERMES_DATA_DIR` environment variable points to the data directory, not the log directory. Logs are stored separately, following platform conventions.

### Test Protocol Manually

You can test your extension manually:

```bash
# start extension
./extension.py

# send initialize (paste into stdin)
Content-Length: 85

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"hermesVersion":"1.0.0","apiVersion":"1.0.0","dataDirectory":"/tmp"}}
```

### Validate JSON

Use `jq` to validate your JSON:

```bash
echo '{"jsonrpc":"2.0","id":1}' | jq .
```

## Next Steps

1. Start with the [Minimal Example](minimal.md) to understand the basics
2. Progress to the [Wizard Example](wizard.md) for a complete real-world extension
3. Read the [Protocol](../protocol.md) documentation for deeper understanding
