# Manage Windows

This guide shows how to open and manage web UI windows for complex user
interactions.

## Open a Simple Window

```python
response = send_request("ui/openWindow", {
    "url": "http://localhost:9876/wizard",
    "title": "Patient Lookup",
    "width": 600,
    "height": 400
})

if "error" in response:
    log(f"Failed to open window: {response['error']['message']}")
    return

window_id = response["result"]["windowId"]
log(f"Opened window: {window_id}")
```

## Open a Modal Window

Modal windows block interaction with the main Hermes window until closed:

```python
response = send_request("ui/openWindow", {
    "url": "http://localhost:9876/wizard",
    "title": "Patient Search",
    "width": 450,
    "height": 400,
    "modal": True
})

window_id = response["result"]["windowId"]
```

Use modal windows for wizards and workflows where you want the user's full
attention.

## Open a Non-Resizable Window

```python
response = send_request("ui/openWindow", {
    "url": "http://localhost:9876/form",
    "title": "Quick Input",
    "width": 400,
    "height": 300,
    "resizable": False
})
```

## Close a Window Programmatically

**Important:** JavaScript's `window.close()` does not work in Hermes extension
windows. You must use the `ui/closeWindow` method.

```python
# save the window ID when opening
response = send_request("ui/openWindow", {
    "url": "http://localhost:9876/wizard",
    "title": "Wizard"
})

window_id = response["result"]["windowId"]

# later, close the window
send_request("ui/closeWindow", {
    "windowId": window_id
})
```

## Host a Local HTTP Server

Extensions typically run a local HTTP server to serve their UI:

```python
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading
import socket

def find_free_port():
    """Find an available port."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]

class UIHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/" or self.path == "/wizard":
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(b"""
                <!DOCTYPE html>
                <html>
                <head><title>Wizard</title></head>
                <body>
                    <h1>Patient Search</h1>
                    <input type="text" id="mrn" placeholder="Enter MRN">
                    <button onclick="search()">Search</button>
                    <script>
                        async function search() {
                            const mrn = document.getElementById('mrn').value;
                            await fetch('/api/search', {
                                method: 'POST',
                                headers: {'Content-Type': 'application/json'},
                                body: JSON.stringify({mrn: mrn})
                            });
                        }
                    </script>
                </body>
                </html>
            """)
        else:
            self.send_error(404)

    def do_POST(self):
        if self.path == "/api/search":
            content_length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(content_length)
            data = json.loads(body)

            # process the search
            log(f"Searching for MRN: {data['mrn']}")

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"success": True}).encode())

    def log_message(self, format, *args):
        # suppress HTTP logging or redirect to extension logs
        pass

# start the server
port = find_free_port()
server = HTTPServer(("127.0.0.1", port), UIHandler)

# run in background thread
thread = threading.Thread(target=server.serve_forever)
thread.daemon = True
thread.start()

log(f"HTTP server started on port {port}")

# open the window
response = send_request("ui/openWindow", {
    "url": f"http://127.0.0.1:{port}/wizard",
    "title": "Patient Lookup"
})
```

## Complete Wizard Pattern

This pattern shows the typical workflow for a wizard with user interaction:

```python
import threading
import json
from http.server import HTTPServer, BaseHTTPRequestHandler

# global state for communication between HTTP handler and main code
wizard_result = None
wizard_event = threading.Event()
wizard_window_id = None
http_server = None

class WizardHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/wizard":
            # serve wizard HTML
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
                        // extension will close the window
                    }
                    async function cancel() {
                        await fetch('/api/cancel', {method: 'POST'});
                        // extension will close the window
                    }
                </script>
            </body>
            </html>
            """
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(html.encode())

    def do_POST(self):
        global wizard_result

        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)

        if self.path == "/api/search":
            data = json.loads(body)
            # simulate database lookup
            wizard_result = {
                "action": "search",
                "mrn": data["mrn"],
                "patient": {
                    "lastName": "DOE",
                    "firstName": "JOHN"
                }
            }
            wizard_event.set()
            self.send_response(200)
            self.end_headers()

        elif self.path == "/api/cancel":
            wizard_result = {"action": "cancel"}
            wizard_event.set()
            self.send_response(200)
            self.end_headers()

    def log_message(self, format, *args):
        pass

def execute_wizard():
    """Run the wizard and wait for user input."""
    global wizard_result, wizard_window_id, http_server

    # reset state
    wizard_result = None
    wizard_event.clear()

    # start HTTP server
    port = find_free_port()
    http_server = HTTPServer(("127.0.0.1", port), WizardHandler)
    thread = threading.Thread(target=http_server.serve_forever)
    thread.daemon = True
    thread.start()

    try:
        # open window
        response = send_request("ui/openWindow", {
            "url": f"http://127.0.0.1:{port}/wizard",
            "title": "Patient Search",
            "width": 400,
            "height": 200,
            "modal": True
        })

        if "error" in response:
            log(f"Failed to open window: {response['error']['message']}")
            return

        wizard_window_id = response["result"]["windowId"]
        log(f"Opened window: {wizard_window_id}")

        # wait for user interaction (with timeout)
        wizard_event.wait(timeout=300)

        # process result
        if wizard_result is None:
            log("Wizard timed out")
            return

        if wizard_result["action"] == "cancel":
            log("User cancelled wizard")
            return

        if wizard_result["action"] == "search":
            patient = wizard_result["patient"]
            log(f"User selected: {patient['firstName']} {patient['lastName']}")

            # update the message
            send_request("editor/patchMessage", {
                "patches": [
                    {"path": "PID.5.1", "value": patient["lastName"]},
                    {"path": "PID.5.2", "value": patient["firstName"]}
                ]
            })

    finally:
        # always clean up
        if wizard_window_id:
            send_request("ui/closeWindow", {"windowId": wizard_window_id})
        if http_server:
            http_server.shutdown()
```

## Clean Up Resources

Always clean up when finished or on shutdown:

```python
def cleanup():
    """Clean up HTTP server and windows."""
    global wizard_window_id, http_server

    # close any open windows
    if wizard_window_id:
        send_request("ui/closeWindow", {"windowId": wizard_window_id})
        wizard_window_id = None

    # stop HTTP server
    if http_server:
        http_server.shutdown()
        http_server = None

def handle_shutdown(request_id, params):
    """Handle shutdown request from Hermes."""
    log("Shutting down")
    cleanup()

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    }
```

## Security Best Practices

| Practice                    | Why                                          |
| --------------------------- | -------------------------------------------- |
| Bind to `127.0.0.1` only    | Prevents external access to your HTTP server |
| Use dynamic port allocation | Avoids conflicts with other services         |
| Validate all web UI input   | Protects against injection attacks           |
| Close windows explicitly    | Prevents resource leaks                      |

**Never bind to `0.0.0.0`** - this would expose your extension's HTTP server to
the network.

```python
# Good: localhost only
server = HTTPServer(("127.0.0.1", port), Handler)

# Bad: exposed to network
server = HTTPServer(("0.0.0.0", port), Handler)
```

## Framework Suggestions

For richer UIs, consider using:

| Framework    | Notes                             |
| ------------ | --------------------------------- |
| Vanilla HTML | Simple, no dependencies           |
| Alpine.js    | Lightweight reactivity            |
| htmx         | Server-driven interactions        |
| React/Vue    | Complex UIs (requires build step) |

## Common Gotchas

### window.close() Doesn't Work

JavaScript's `window.close()` does not work in Hermes extension windows. Always
use `ui/closeWindow`:

```python
# Wrong: won't work
# <button onclick="window.close()">Close</button>

# Right: close via extension API
if wizard_window_id:
    send_request("ui/closeWindow", {"windowId": wizard_window_id})
```

### Port Conflicts

Always use dynamic port allocation to avoid conflicts:

```python
# Good: find free port
port = find_free_port()
server = HTTPServer(("127.0.0.1", port), Handler)

# Bad: hardcoded port might conflict
server = HTTPServer(("127.0.0.1", 8080), Handler)
```

### Forgetting to Clean Up

Always clean up in `finally` blocks and shutdown handlers:

```python
try:
    # open window and run wizard
    execute_wizard()
finally:
    # always clean up, even on error
    if window_id:
        send_request("ui/closeWindow", {"windowId": window_id})
    if server:
        server.shutdown()
```

## Related Documentation

- [Reference: ui/openWindow](../reference/methods.md#uiopenwindow)
- [Reference: ui/closeWindow](../reference/methods.md#uiclosewindow)
- [Tutorial: Building a Wizard Extension](../tutorials/wizard-with-ui.md)
- [How-To: Show Dialogs](show-dialogs.md)
