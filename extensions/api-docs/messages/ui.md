# UI Messages

These messages allow extensions to create user interface elements in Hermes.

## Direction

**Extension → Hermes**

---

## ui/openWindow

Opens a new browser window that loads a URL served by the extension. This enables extensions to provide rich, interactive user interfaces, such as wizards, forms, or dashboards, using web technologies.

### Request

#### Method

```
ui/openWindow
```

#### Parameters

```typescript
interface OpenWindowParams {
  /** URL to load in the window (must be http:// or https://) */
  url: string;

  /** Window title shown in the title bar */
  title: string;

  /** Window width in pixels (optional, default: 800) */
  width?: number;

  /** Window height in pixels (optional, default: 600) */
  height?: number;

  /** Whether the window should be modal (optional, default: false) */
  modal?: boolean;

  /** Whether the window is resizable (optional, default: true) */
  resizable?: boolean;
}
```

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ui/openWindow",
  "params": {
    "url": "http://localhost:9876/wizard",
    "title": "Patient Lookup Wizard",
    "width": 600,
    "height": 400,
    "modal": true
  }
}
```

### Response

#### Result

```typescript
interface OpenWindowResult {
  /** Unique identifier for the opened window */
  windowId: string;
}
```

#### Success Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "windowId": "ext-window-abc123"
  }
}
```

#### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Failed to open window",
    "data": "Invalid URL scheme: must be http or https"
  }
}
```

### URL Requirements

| Requirement                     | Details                                    |
|---------------------------------|--------------------------------------------|
| Scheme                          | Must be `http://` or `https://`            |
| Host                            | Any valid hostname or IP                   |
| Typical usage                   | `localhost` with extension-provided server |

**Security:** Hermes does not restrict which URLs can be loaded. Extensions are trusted to serve appropriate content.

### Window Behaviour

#### Modal Windows

When `modal: true`:
- The window appears above the main Hermes window
- The main window is dimmed/disabled
- User must close the extension window to return to Hermes

#### Non-Modal Windows

When `modal: false` (default):
- The window opens independently
- User can interact with both windows
- Multiple windows can be open simultaneously

#### Window Lifecycle

1. Window opens and loads the URL
2. User interacts with the web content
3. Extension's web server handles requests
4. User closes the window (or extension closes it)

### Serving Content

Extensions typically run a local HTTP server to serve their UI:

```python
from http.server import HTTPServer, SimpleHTTPRequestHandler
import threading

class WizardHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/wizard":
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.end_headers()
            self.wfile.write(b"""
                <!DOCTYPE html>
                <html>
                <head><title>Wizard</title></head>
                <body>
                    <h1>Patient Lookup</h1>
                    <input type="text" id="mrn" placeholder="Enter MRN">
                    <button onclick="search()">Search</button>
                    <script>
                        function search() {
                            // POST results back to extension
                            fetch('/api/search', {
                                method: 'POST',
                                body: JSON.stringify({mrn: document.getElementById('mrn').value})
                            });
                        }
                    </script>
                </body>
                </html>
            """)

def start_server():
    server = HTTPServer(("localhost", 9876), WizardHandler)
    thread = threading.Thread(target=server.serve_forever)
    thread.daemon = True
    thread.start()
    return server
```

### Communication Patterns

The window content (web page) communicates with the extension via HTTP:

```
┌───────────────────────────────────────────────────────────┐
│                          Hermes                           │
│  ┌─────────────────────────────────────────────────────┐  │
│  │             Extension Window (WebView)              │  │
│  │                                                     │  │
│  │    [MRN: 12345    ]  [Search]  [Cancel]             │  │
│  │                                                     │  │
│  └──────────────────────────┬──────────────────────────┘  │
└─────────────────────────────┼─────────────────────────────┘
                              │ HTTP (fetch/XHR)
                              ▼
                   ┌────────────────────┐
                   │  Extension Process │
                   │    (HTTP Server)   │
                   │   localhost:9876   │
                   └────────────────────┘
```

### Complete Example

Here's a typical workflow:

```python
import json
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading

# store for communication between HTTP handler and main extension
search_result = None
search_complete = threading.Event()

class WizardHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/wizard":
            # serve the wizard HTML
            html = """
            <!DOCTYPE html>
            <html>
            <head>
                <title>Patient Search</title>
                <style>
                    body { font-family: system-ui; padding: 20px; }
                    input { padding: 8px; margin: 8px 0; width: 200px; }
                    button { padding: 8px 16px; margin: 4px; }
                </style>
            </head>
            <body>
                <h2>Search Patient</h2>
                <div>
                    <input type="text" id="mrn" placeholder="Enter MRN">
                </div>
                <div>
                    <button onclick="search()">Search</button>
                    <button onclick="cancel()">Cancel</button>
                </div>
                <script>
                    async function search() {
                        const mrn = document.getElementById('mrn').value;
                        await fetch('/api/search', {
                            method: 'POST',
                            headers: {'Content-Type': 'application/json'},
                            body: JSON.stringify({mrn: mrn})
                        });
                        window.close();
                    }
                    function cancel() {
                        fetch('/api/cancel', {method: 'POST'});
                        window.close();
                    }
                </script>
            </body>
            </html>
            """
            self.send_response(200)
            self.send_header("Content-Type", "text/html")
            self.end_headers()
            self.wfile.write(html.encode())

    def do_POST(self):
        global search_result
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)

        if self.path == "/api/search":
            data = json.loads(body)
            # simulate database lookup
            search_result = {
                "found": True,
                "patient": {
                    "mrn": data["mrn"],
                    "lastName": "DOE",
                    "firstName": "JOHN",
                    "dob": "19800101",
                    "sex": "M"
                }
            }
            search_complete.set()
            self.send_response(200)
            self.end_headers()

        elif self.path == "/api/cancel":
            search_result = {"found": False, "cancelled": True}
            search_complete.set()
            self.send_response(200)
            self.end_headers()

    def log_message(self, format, *args):
        pass  # suppress HTTP logging

def handle_search_command(request_id):
    global search_result
    search_result = None
    search_complete.clear()

    # start HTTP server
    server = HTTPServer(("localhost", 9876), WizardHandler)
    server_thread = threading.Thread(target=server.serve_forever)
    server_thread.daemon = True
    server_thread.start()

    # open window
    response = send_request("ui/openWindow", {
        "url": "http://localhost:9876/wizard",
        "title": "Patient Search",
        "width": 400,
        "height": 200,
        "modal": True
    })

    if "error" in response:
        server.shutdown()
        return error_response(request_id, "Failed to open window")

    # wait for user interaction
    search_complete.wait(timeout=300)  # 5 minute timeout
    server.shutdown()

    if not search_result or search_result.get("cancelled"):
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {"success": False, "message": "Search cancelled"}
        }

    if not search_result.get("found"):
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {"success": False, "message": "Patient not found"}
        }

    # update the message with patient data
    patient = search_result["patient"]
    send_request("editor/patchMessage", {
        "patches": [
            {"path": "PID.3.1", "value": patient["mrn"]},
            {"path": "PID.5.1", "value": patient["lastName"]},
            {"path": "PID.5.2", "value": patient["firstName"]},
            {"path": "PID.7", "value": patient["dob"]},
            {"path": "PID.8", "value": patient["sex"]}
        ]
    })

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True, "message": "Patient data loaded"}
    }
```

### Security Considerations

| Concern                    | Recommendation                                    |
|----------------------------|---------------------------------------------------|
| Port conflicts             | Use dynamic port allocation or check availability |
| Localhost only             | Bind to `127.0.0.1`, not `0.0.0.0`               |
| Input validation           | Validate all data from the web UI                 |
| CORS                       | Not needed for same-origin localhost requests     |

### Best Practices

1. **Use modal windows for wizards** - Prevents the user from making conflicting changes
2. **Provide a cancel button** - Always let users abort
3. **Clean up on completion** - Shut down the HTTP server when done
4. **Handle window close** - The user may close the window without using your buttons
5. **Set reasonable timeouts** - Avoid waiting indefinitely for user input

### Framework Suggestions

For richer UIs, consider using:

| Framework     | Notes                                      |
|---------------|--------------------------------------------|
| Vanilla HTML  | Simple, no dependencies                    |
| Alpine.js     | Lightweight reactivity                     |
| htmx          | Server-driven interactions                 |
| React/Vue     | Complex UIs (requires build step)          |

---

## ui/closeWindow

Closes a window that was previously opened by the extension.

### Request

#### Method

```
ui/closeWindow
```

#### Parameters

```typescript
interface CloseWindowParams {
  /** The window ID returned from ui/openWindow */
  windowId: string;
}
```

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "ui/closeWindow",
  "params": {
    "windowId": "ext-window-abc123"
  }
}
```

### Response

#### Result

```typescript
interface CloseWindowResult {
  /** Whether the window was closed */
  success: boolean;
}
```

#### Success Response

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "success": true
  }
}
```

#### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32008,
    "message": "Window error",
    "data": "Window not found: ext-window-abc123"
  }
}
```

### Behaviour

| Scenario                    | Result                                  |
|-----------------------------|-----------------------------------------|
| Window exists and is open   | Window closes, returns `success: true`  |
| Window already closed       | No-op, returns `success: true`          |
| Window ID not recognised    | Returns error -32008                    |

### Use Cases

- **Timeout handling:** Close wizard window after inactivity
- **Error recovery:** Close window when backend operation fails
- **Workflow completion:** Close window after "Apply and Close" action

---

## window/closed Notification

**Direction: Hermes → Extension**

Hermes sends this notification when a window opened by the extension is closed, either by user action or programmatically.

### Notification

```typescript
interface WindowClosedParams {
  /** The window ID that was closed */
  windowId: string;

  /** How the window was closed */
  reason: "user" | "extension" | "shutdown";
}
```

| Reason      | Description                                      |
|-------------|--------------------------------------------------|
| `user`      | User clicked the close button or pressed Cmd+W   |
| `extension` | Extension called `ui/closeWindow`                |
| `shutdown`  | Hermes is shutting down                          |

### Example Notification

```json
{
  "jsonrpc": "2.0",
  "method": "window/closed",
  "params": {
    "windowId": "ext-window-abc123",
    "reason": "user"
  }
}
```

### Handling

Since this is a notification (no `id` field), extensions should not send a response. Use this to:

- Clean up resources associated with the window
- Cancel pending operations that depended on user input
- Update internal state

```python
def handle_notification(msg):
    if msg["method"] == "window/closed":
        window_id = msg["params"]["windowId"]
        reason = msg["params"]["reason"]
        log(f"Window {window_id} closed: {reason}")

        # signal any waiting operations to abort
        if window_id in pending_wizards:
            pending_wizards[window_id].cancel()
```

---

## Security Considerations

### Trust Model

Extensions are **trusted code**. Hermes does not restrict:

- Which URLs extensions can open in windows
- What content is loaded in extension windows
- Network requests made by extension web UIs

**Users should only install extensions from trusted sources.**

### Recommendations for Extension Authors

| Practice                    | Why                                              |
|-----------------------------|--------------------------------------------------|
| Bind to `127.0.0.1` only    | Prevents external access to your HTTP server     |
| Use dynamic port allocation | Avoids conflicts with other services             |
| Validate all web UI input   | Protects against injection attacks               |
| Avoid storing credentials   | Prevents persisting sensitive data               |

### Recommendations for Users

- Review extension source code before installing
- Only install extensions from known developers
- Be cautious of extensions that request unusual permissions

---

## Future Extensions

Future API versions may add:

- `ui/showNotification` - Show toast notifications
- `ui/updateWindow` - Change window title, size, or position
- Additional window events (resized, focused, etc.)

## Related Documentation

- [Commands](commands.md) - Opening windows from commands
- [Examples: Wizard](../examples/wizard.md) - Complete wizard example
