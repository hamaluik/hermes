# Tutorial: Building a Wizard with a Web UI

In this advanced tutorial, you'll build a complete wizard extension with a web
interface. You'll learn how to create multi-step workflows, manage windows,
run an HTTP server, and provide schema overrides for custom fields.

## What You'll Build

A patient lookup wizard that:
1. Opens a search window when you click the toolbar button
2. Lets users search for patients by MRN or last name
3. Shows matching results in a selectable list
4. Populates the HL7 message with the selected patient's data
5. Closes the window automatically when done

This demonstrates the full power of Hermes extensions—building rich,
interactive workflows that integrate with external data sources.

## What You'll Learn

- Running an embedded HTTP server for the UI
- Opening and closing windows via the API
- Building multi-view interfaces (search → results → loading)
- Handling async operations with threading
- Finding available ports dynamically
- Providing schema overrides for organisation-specific fields
- Proper resource cleanup on shutdown

## Prerequisites

Start with the complete code from [Your First Extension](first-extension.md).
You'll add threading, HTTP server, and window management to that foundation.

You should be comfortable with the concepts from that tutorial:
- Message framing and JSON-RPC
- The initialize handshake
- Sending requests to Hermes
- Handling commands

## Architecture Overview

Here's what we're building:

```
┌─────────────────────────────────────────┐
│ Hermes                                  │
│                                         │
│  Toolbar: [🔍 Patient Lookup]          │
│                    │                    │
│                    │ (click)            │
│                    ▼                    │
│  ┌──────────────────────────────────┐   │
│  │   Wizard Window (WebView)        │   │
│  │                                  │   │
│  │   Search: [DOE         ]         │   │
│  │                                  │   │
│  │   Results:                       │   │
│  │   ☑ DOE, JOHN (MRN: 12345678)    │   │
│  │   ☐ DOE, JANE (MRN: 11112222)    │   │
│  │                                  │   │
│  │   [Apply] [Back] [Cancel]        │   │
│  │                                  │   │
│  └──────────────┬───────────────────┘   │
│                 │ HTTP                  │
└─────────────────┼───────────────────────┘
                  │
         ┌────────┴────────┐
         │ Extension       │
         │                 │
         │  HTTP Server    │
         │  localhost:9876 │
         │                 │
         │  Patient DB     │
         │  (simulated)    │
         │                 │
         └─────────────────┘
```

## Step 1: Set Up Imports and Constants

Create `patient_wizard.py`:

```python
#!/usr/bin/env python3
"""
Patient Lookup Wizard Extension

A complete wizard with web UI for looking up and loading patient data.
"""

import sys
import json
import threading
import socket
from http.server import HTTPServer, BaseHTTPRequestHandler

EXTENSION_NAME = "Patient Lookup Wizard"
EXTENSION_VERSION = "1.0.0"
```

We need `threading` for async operations and `http.server` for the web UI.

## Step 2: Create a Simulated Patient Database

For this tutorial, we'll use a simple in-memory database. In a real extension,
you'd query an actual database or API:

```python
# simulated patient database
PATIENTS = {
    "12345678": {
        "mrn": "12345678",
        "lastName": "DOE",
        "firstName": "JOHN",
        "middleName": "Q",
        "dob": "19800115",
        "sex": "M",
        "address": {
            "street": "123 MAIN ST",
            "city": "ANYTOWN",
            "state": "ON",
            "zip": "A1A 1A1"
        },
        "phone": "5551234567",
        "accountNumber": "ACC001234"
    },
    "11112222": {
        "mrn": "11112222",
        "lastName": "DOE",
        "firstName": "JANE",
        "middleName": "B",
        "dob": "19850310",
        "sex": "F",
        "address": {
            "street": "789 PINE RD",
            "city": "ANYTOWN",
            "state": "ON",
            "zip": "A2A 2A2"
        },
        "phone": "5555551234",
        "accountNumber": "ACC009999"
    }
}
```

## Step 3: Add Global State

Wizards need state to coordinate between the web UI and the extension:

```python
# global state for wizard coordination
http_server = None
http_port = None
wizard_window_id = None
wizard_result = None
wizard_event = threading.Event()
```

- `http_server`: The running HTTP server instance
- `http_port`: Which port the server is listening on
- `wizard_window_id`: ID of the open window (for closing later)
- `wizard_result`: Data collected from the user
- `wizard_event`: Threading event to signal completion

## Step 4: Update send_request() for Thread Safety

The `read_message()`, `write_message()`, and `log()` functions from the first
tutorial work as-is. However, `send_request()` needs to be thread-safe since
commands run in background threads. Replace your `send_request()` with this
version:

```python
_next_id = 1
_pending = {}
_message_lock = threading.Lock()

def send_request(method, params):
    """Send a request to Hermes and wait for response (thread-safe)."""
    global _next_id

    with _message_lock:
        request_id = _next_id
        _next_id += 1
        _pending[request_id] = threading.Event()

        write_message({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params
        })

    # wait for response (outside the lock)
    _pending[request_id].wait(timeout=30)

    with _message_lock:
        if request_id in _pending:
            result = _pending.pop(request_id)
            if isinstance(result, threading.Event):
                raise TimeoutError(f"Request {method} timed out")
            return result

    raise Exception("Request lost")


def handle_response(msg):
    """Handle a response from Hermes (called from main loop)."""
    request_id = msg.get("id")
    with _message_lock:
        if request_id in _pending:
            event = _pending[request_id]
            _pending[request_id] = msg
            event.set()
```

The lock prevents race conditions when multiple threads send requests.

## Step 5: Create the HTML Interface

The wizard's UI is served as HTML. Add this function:

```python
def get_wizard_html():
    """Return the wizard HTML page."""
    return """<!DOCTYPE html>
<html>
<head>
    <title>Patient Lookup</title>
    <style>
        * { box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            max-width: 400px;
            margin: 0 auto;
            background: white;
            padding: 24px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        h1 {
            margin: 0 0 20px 0;
            font-size: 20px;
            color: #333;
        }
        .form-group {
            margin-bottom: 16px;
        }
        label {
            display: block;
            margin-bottom: 4px;
            font-weight: 500;
            color: #555;
        }
        input[type="text"] {
            width: 100%;
            padding: 10px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 16px;
        }
        input[type="text"]:focus {
            outline: none;
            border-color: #007bff;
            box-shadow: 0 0 0 2px rgba(0,123,255,0.25);
        }
        .buttons {
            display: flex;
            gap: 8px;
            margin-top: 20px;
        }
        button {
            flex: 1;
            padding: 10px 16px;
            border: none;
            border-radius: 4px;
            font-size: 14px;
            cursor: pointer;
            transition: background 0.2s;
        }
        .btn-primary {
            background: #007bff;
            color: white;
        }
        .btn-primary:hover:not(:disabled) { background: #0056b3; }
        .btn-primary:disabled {
            background: #ccc;
            cursor: not-allowed;
        }
        .btn-secondary {
            background: #e0e0e0;
            color: #333;
        }
        .btn-secondary:hover { background: #ccc; }
        .error {
            color: #dc3545;
            margin-top: 12px;
            padding: 8px;
            background: #f8d7da;
            border-radius: 4px;
            display: none;
        }
        .view { display: none; }
        .view.active { display: block; }
        .results-list {
            max-height: 200px;
            overflow-y: auto;
            border: 1px solid #ddd;
            border-radius: 4px;
            margin-bottom: 16px;
        }
        .result-item {
            padding: 12px;
            border-bottom: 1px solid #eee;
            cursor: pointer;
            transition: background 0.15s;
        }
        .result-item:last-child { border-bottom: none; }
        .result-item:hover { background: #f0f7ff; }
        .result-item.selected {
            background: #e3f2fd;
            border-left: 3px solid #007bff;
        }
        .result-name {
            font-weight: 500;
            color: #333;
        }
        .result-mrn {
            font-size: 12px;
            color: #666;
            margin-top: 2px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Patient Lookup</h1>

        <!-- Search View -->
        <div id="searchView" class="view active">
            <div class="form-group">
                <label for="query">Search by MRN or Last Name</label>
                <input type="text" id="query" placeholder="Enter MRN or last name">
            </div>
            <div class="buttons">
                <button class="btn-primary" onclick="search()">Search</button>
                <button class="btn-secondary" onclick="cancel()">Cancel</button>
            </div>
            <div id="searchError" class="error"></div>
        </div>

        <!-- Results View -->
        <div id="resultsView" class="view">
            <div id="resultsHeader">Select a patient:</div>
            <div class="results-list" id="resultsList"></div>
            <div class="buttons">
                <button class="btn-primary" id="applyBtn" onclick="applyPatient()" disabled>Apply</button>
                <button class="btn-secondary" onclick="backToSearch()">Back</button>
                <button class="btn-secondary" onclick="cancel()">Cancel</button>
            </div>
        </div>

        <!-- Loading View -->
        <div id="loadingView" class="view">
            <div style="text-align: center; padding: 20px;">Searching...</div>
        </div>
    </div>

    <script>
        const queryInput = document.getElementById('query');
        const searchView = document.getElementById('searchView');
        const resultsView = document.getElementById('resultsView');
        const loadingView = document.getElementById('loadingView');
        const searchError = document.getElementById('searchError');
        const resultsList = document.getElementById('resultsList');
        const applyBtn = document.getElementById('applyBtn');

        let selectedMrn = null;
        let searchResults = [];

        queryInput.focus();
        queryInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') search();
        });

        function showView(view) {
            searchView.classList.remove('active');
            resultsView.classList.remove('active');
            loadingView.classList.remove('active');
            view.classList.add('active');
        }

        async function search() {
            const query = queryInput.value.trim();
            if (!query) {
                showError('Please enter a search term');
                return;
            }

            hideError();
            showView(loadingView);

            try {
                const response = await fetch('/api/search', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ query })
                });
                const data = await response.json();

                if (data.patients && data.patients.length > 0) {
                    searchResults = data.patients;
                    selectedMrn = null;
                    applyBtn.disabled = true;
                    renderResults();
                    showView(resultsView);
                } else {
                    showError('No patients found');
                    showView(searchView);
                }
            } catch (err) {
                showError('Failed to connect to extension');
                showView(searchView);
            }
        }

        function renderResults() {
            resultsList.innerHTML = searchResults.map(p => `
                <div class="result-item" data-mrn="${p.mrn}" onclick="selectPatient('${p.mrn}')">
                    <div class="result-name">${p.lastName}, ${p.firstName}</div>
                    <div class="result-mrn">MRN: ${p.mrn}</div>
                </div>
            `).join('');
        }

        function selectPatient(mrn) {
            selectedMrn = mrn;
            applyBtn.disabled = false;
            document.querySelectorAll('.result-item').forEach(el => {
                el.classList.toggle('selected', el.dataset.mrn === mrn);
            });
        }

        async function applyPatient() {
            if (!selectedMrn) return;
            showView(loadingView);

            try {
                const response = await fetch('/api/apply', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ mrn: selectedMrn })
                });
                const data = await response.json();

                if (!data.success) {
                    showError(data.message || 'Failed to apply patient');
                    showView(resultsView);
                }
            } catch (err) {
                showError('Failed to connect to extension');
                showView(resultsView);
            }
        }

        function backToSearch() {
            selectedMrn = null;
            showView(searchView);
            queryInput.focus();
        }

        async function cancel() {
            try {
                await fetch('/api/cancel', { method: 'POST' });
            } catch (err) {
                // ignore errors on cancel
            }
        }

        function showError(msg) {
            searchError.textContent = msg;
            searchError.style.display = 'block';
        }

        function hideError() {
            searchError.style.display = 'none';
        }
    </script>
</body>
</html>"""
```

This HTML has three views (search, results, loading) that switch based on user
actions.

## Step 6: Add the Search Logic

Create a function to search the patient database:

```python
def search_patients(query):
    """Search patients by MRN or last name."""
    query = query.upper()
    results = []

    for mrn, patient in PATIENTS.items():
        # match by MRN prefix or last name contains
        if mrn.startswith(query) or query in patient["lastName"]:
            results.append({
                "mrn": patient["mrn"],
                "lastName": patient["lastName"],
                "firstName": patient["firstName"]
            })

    return results
```

## Step 7: Create the HTTP Request Handler

The HTTP server needs to handle GET (serve HTML) and POST (API calls):

```python
class WizardHandler(BaseHTTPRequestHandler):
    """HTTP request handler for wizard web UI."""

    def do_GET(self):
        if self.path == "/" or self.path == "/wizard":
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(get_wizard_html().encode("utf-8"))
        else:
            self.send_error(404)

    def do_POST(self):
        global wizard_result

        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length).decode("utf-8")

        if self.path == "/api/search":
            data = json.loads(body) if body else {}
            query = data.get("query", "")
            patients = search_patients(query)
            self.send_json({"patients": patients})

        elif self.path == "/api/apply":
            data = json.loads(body) if body else {}
            mrn = data.get("mrn", "")
            patient = PATIENTS.get(mrn)
            if patient:
                wizard_result = {"action": "apply", "patient": patient}
                wizard_event.set()
                self.send_json({"success": True})
            else:
                self.send_json({"success": False, "message": "Patient not found"})

        elif self.path == "/api/cancel":
            wizard_result = {"action": "cancel"}
            wizard_event.set()
            self.send_json({"success": True})

        else:
            self.send_error(404)

    def send_json(self, data):
        content = json.dumps(data).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", len(content))
        self.end_headers()
        self.wfile.write(content)

    def log_message(self, format, *args):
        # log HTTP requests to extension stderr
        log(f"HTTP: {args[0]}")
```

The handler sets `wizard_result` and signals the event when the user makes a
choice.

## Step 8: Add Server Start/Stop Functions

```python
def find_free_port():
    """Find an available port dynamically."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def start_http_server():
    """Start the HTTP server for the wizard UI."""
    global http_server, http_port

    http_port = find_free_port()
    http_server = HTTPServer(("127.0.0.1", http_port), WizardHandler)

    thread = threading.Thread(target=http_server.serve_forever)
    thread.daemon = True
    thread.start()

    log(f"HTTP server started on port {http_port}")
    return http_port


def stop_http_server():
    """Stop the HTTP server."""
    global http_server
    if http_server:
        log("Stopping HTTP server")
        http_server.shutdown()
        http_server = None


def close_wizard_window():
    """Close the wizard window via Hermes API."""
    global wizard_window_id
    if wizard_window_id:
        log(f"Closing window: {wizard_window_id}")
        try:
            send_request("ui/closeWindow", {"windowId": wizard_window_id})
        except Exception as e:
            log(f"Failed to close window: {e}")
        wizard_window_id = None
```

**Note**: JavaScript's `window.close()` doesn't work in Hermes. Always use the
`ui/closeWindow` API.

## Step 9: Handle Initialize with Schema Overrides

```python
def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": EXTENSION_NAME,
            "version": EXTENSION_VERSION,
            "description": "Look up patients and populate HL7 messages",
            "capabilities": {
                "commands": ["wizard/patientLookup"],
                "schemaProvider": True
            },
            "toolbarButtons": [
                {
                    "id": "wizard-patient-lookup",
                    "label": "Patient Lookup",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="11" cy="11" r="8"/>
                        <path d="M21 21l-4.35-4.35"/>
                        <circle cx="11" cy="8" r="2"/>
                        <path d="M11 10v2"/>
                        <path d="M8 14h6"/>
                    </svg>""",
                    "command": "wizard/patientLookup"
                }
            ],
            "schema": {
                "segments": {
                    "PID": {
                        "fields": [
                            {
                                "field": 3,
                                "component": 1,
                                "note": "8-digit MRN from Patient Master Index"
                            },
                            {
                                "field": 3,
                                "component": 4,
                                "template": "MRN"
                            }
                        ]
                    }
                }
            }
        }
    }
```

The schema section adds organisation-specific field notes and templates.

## Step 10: Execute the Patient Lookup

This is where everything comes together:

```python
def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "wizard/patientLookup":
        # start async work in background thread
        thread = threading.Thread(target=execute_patient_lookup)
        thread.start()
    else:
        log(f"Unknown command: {command}")


def execute_patient_lookup():
    """Execute the patient lookup wizard asynchronously."""
    global wizard_result, wizard_window_id

    # reset state
    wizard_result = None
    wizard_window_id = None
    wizard_event.clear()

    # start HTTP server
    port = start_http_server()

    try:
        # open wizard window
        response = send_request("ui/openWindow", {
            "url": f"http://127.0.0.1:{port}/wizard",
            "title": "Patient Lookup",
            "width": 450,
            "height": 400,
            "modal": True
        })

        if "error" in response:
            log(f"Failed to open wizard: {response['error']['message']}")
            return

        # save window ID for later closing
        wizard_window_id = response.get("result", {}).get("windowId")
        log(f"Opened window: {wizard_window_id}")

        # wait for user interaction (up to 5 minutes)
        wizard_event.wait(timeout=300)

        # process result
        if wizard_result is None:
            log("Wizard timed out")
            return

        if wizard_result.get("action") == "cancel":
            log("Wizard cancelled")
            return

        if wizard_result.get("action") == "apply":
            patient = wizard_result["patient"]

            # populate patient data
            patches = [
                {"path": "PID.3.1", "value": patient["mrn"]},
                {"path": "PID.3.4", "value": "MRN"},
                {"path": "PID.5.1", "value": patient["lastName"]},
                {"path": "PID.5.2", "value": patient["firstName"]},
                {"path": "PID.5.3", "value": patient.get("middleName", "")},
                {"path": "PID.7", "value": patient["dob"]},
                {"path": "PID.8", "value": patient["sex"]},
                {"path": "PID.11.1", "value": patient["address"]["street"]},
                {"path": "PID.11.3", "value": patient["address"]["city"]},
                {"path": "PID.11.4", "value": patient["address"]["state"]},
                {"path": "PID.11.5", "value": patient["address"]["zip"]},
                {"path": "PID.13.1", "value": patient["phone"]},
                {"path": "PID.18.1", "value": patient["accountNumber"]},
            ]

            patch_response = send_request("editor/patchMessage", {"patches": patches})

            if "error" in patch_response:
                log(f"Failed to update message: {patch_response['error']['message']}")
                return

            if not patch_response.get("result", {}).get("success"):
                errors = patch_response.get("result", {}).get("errors", [])
                if errors:
                    log(f"Patch failed: {errors[0]['message']}")
                    return

            log(f"Loaded patient: {patient['firstName']} {patient['lastName']}")

    finally:
        # always clean up
        close_wizard_window()
        stop_http_server()
```

The `try/finally` ensures resources are cleaned up even if errors occur.

## Step 11: Handle Shutdown

```python
def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
    close_wizard_window()
    stop_http_server()
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    }
```

## Step 12: Update the Main Loop

The main loop needs to handle responses differently because of threading:

```python
def handle_message(msg):
    """Route message to appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    if request_id is None:
        if method == "command/execute":
            handle_command(params)
        else:
            log(f"Unknown notification: {method}")
        return None

    if method == "initialize":
        return handle_initialize(request_id, params)
    elif method == "shutdown":
        response = handle_shutdown(request_id, params)
        write_message(response)
        sys.exit(0)
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }


def main():
    log("Starting")

    while True:
        try:
            msg = read_message()
            if msg is None:
                log("Connection closed")
                break

            # check if this is a response
            if "result" in msg or "error" in msg:
                handle_response(msg)
                continue

            # handle request or notification
            response = handle_message(msg)
            if response:
                write_message(response)

        except Exception as e:
            log(f"Error: {e}")
            import traceback
            traceback.print_exc(file=sys.stderr)
            break

    close_wizard_window()
    stop_http_server()
    log("Exiting")


if __name__ == "__main__":
    main()
```

## Checkpoint: Test the Wizard

1. Add the extension to Hermes settings
2. Reload extensions
3. Click the search button in the toolbar
4. A window should open with the search form
5. Type "DOE" and click Search
6. You should see two patients
7. Click one, then click Apply
8. The window closes and the message is populated

Check the extension logs to see the flow of operations.

## What You've Built

You've created a sophisticated multi-step wizard that:

```
User clicks button
       │
       ▼
Extension starts HTTP server on random port
       │
       ▼
Extension opens window pointing to localhost:PORT
       │
       ▼
User interacts with web UI
       │
       ├─ Searches for "DOE"
       ├─ Sees 2 results
       ├─ Selects one
       └─ Clicks Apply
       │
       ▼
Web UI POSTs to /api/apply
       │
       ▼
Extension receives patient data
       │
       ▼
Extension patches message with patient data
       │
       ▼
Extension closes window via ui/closeWindow
       │
       ▼
Extension stops HTTP server
       │
       ▼
Done!
```

## What You've Learned

- **HTTP servers**: Running embedded servers for custom UIs
- **Dynamic ports**: Finding available ports automatically
- **Window management**: Opening and closing windows via API
- **Threading**: Running long operations without blocking
- **Events**: Coordinating between threads with threading.Event
- **Resource cleanup**: Using try/finally for proper shutdown
- **Schema overrides**: Providing custom field definitions
- **Multi-view UIs**: Building wizard flows with state transitions

## Going Further

Try these enhancements:

1. **Real database**: Replace the simulated database with SQLite or PostgreSQL
2. **More search options**: Add date of birth and sex filters
3. **Confirmation dialog**: Ask before overwriting existing patient data
4. **Recent searches**: Remember the last 5 searches
5. **Error handling**: Show better error messages in the UI
6. **Styling**: Match your organisation's colour scheme

## Next Steps

You've completed all the tutorials! You now have the skills to build
sophisticated Hermes extensions. Explore these resources next:

- [How-To Guides](../how-to/) - Specific tasks and recipes
- [API Reference](../reference/) - Complete technical reference
- [Explanation](../explanation/) - Architecture and design rationale

Congratulations on completing the Hermes extension tutorials!
