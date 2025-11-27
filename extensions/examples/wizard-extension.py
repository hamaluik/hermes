#!/usr/bin/env python3
"""
Patient Lookup Wizard Extension

Demonstrates:
- Web UI via HTTP server
- User interaction handling
- Message patching
- Schema overrides
- Resource cleanup
- Window management via ui/closeWindow
"""

import sys
import json
import threading
from http.server import HTTPServer, BaseHTTPRequestHandler
import socket

# ============================================================================
# Configuration
# ============================================================================

EXTENSION_NAME = "Patient Lookup Wizard"
EXTENSION_VERSION = "1.0.0"

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
    "87654321": {
        "mrn": "87654321",
        "lastName": "SMITH",
        "firstName": "JANE",
        "middleName": "A",
        "dob": "19751220",
        "sex": "F",
        "address": {
            "street": "456 OAK AVE",
            "city": "SOMEWHERE",
            "state": "BC",
            "zip": "V1V 1V1"
        },
        "phone": "5559876543",
        "accountNumber": "ACC005678"
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

# ============================================================================
# Global State
# ============================================================================

http_server = None
http_port = None
wizard_window_id = None
wizard_result = None
wizard_event = threading.Event()

# ============================================================================
# Message I/O
# ============================================================================

def read_message():
    """Read a JSON-RPC message from stdin."""
    headers = {}
    while True:
        line = sys.stdin.readline()
        if line == "\r\n" or line == "\n":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            headers[key.strip()] = value.strip()

    content_length = int(headers.get("Content-Length", 0))
    if content_length == 0:
        return None

    content = sys.stdin.read(content_length)
    return json.loads(content)


def write_message(msg):
    """Write a JSON-RPC message to stdout."""
    content = json.dumps(msg)
    content_bytes = content.encode("utf-8")
    sys.stdout.write(f"Content-Length: {len(content_bytes)}\r\n\r\n")
    sys.stdout.write(content)
    sys.stdout.flush()


def log(message):
    """Log to stderr."""
    sys.stderr.write(f"[wizard-ext] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Request/Response Handling
# ============================================================================

_next_id = 1
_pending = {}
_message_lock = threading.Lock()


def send_request(method, params):
    """Send a request to Hermes and wait for response."""
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

    # wait for response
    _pending[request_id].wait(timeout=30)

    with _message_lock:
        if request_id in _pending:
            result = _pending.pop(request_id)
            if isinstance(result, threading.Event):
                raise TimeoutError(f"Request {method} timed out")
            return result

    raise Exception("Request lost")


def handle_response(msg):
    """Handle a response from Hermes."""
    request_id = msg.get("id")
    with _message_lock:
        if request_id in _pending:
            event = _pending[request_id]
            _pending[request_id] = msg
            event.set()


# ============================================================================
# HTTP Server for Web UI
# ============================================================================

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
        .loading {
            text-align: center;
            padding: 20px;
            display: none;
        }
        .help-text {
            font-size: 12px;
            color: #888;
            margin-top: 4px;
        }
        .view { display: none; }
        .view.active { display: block; }

        /* Results list */
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
        .no-results {
            padding: 20px;
            text-align: center;
            color: #888;
        }
        .results-header {
            font-size: 14px;
            color: #666;
            margin-bottom: 8px;
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
                <div class="help-text">Try: DOE, SMITH, 123, or 87654321</div>
            </div>

            <div class="buttons">
                <button class="btn-primary" onclick="search()">Search</button>
                <button class="btn-secondary" onclick="cancel()">Cancel</button>
            </div>

            <div id="searchError" class="error"></div>
        </div>

        <!-- Results View -->
        <div id="resultsView" class="view">
            <div class="results-header" id="resultsHeader">Select a patient:</div>
            <div class="results-list" id="resultsList"></div>

            <div class="buttons">
                <button class="btn-primary" id="applyBtn" onclick="applyPatient()" disabled>Apply</button>
                <button class="btn-secondary" onclick="backToSearch()">Back</button>
                <button class="btn-secondary" onclick="cancel()">Cancel</button>
            </div>
        </div>

        <!-- Loading View -->
        <div id="loadingView" class="view">
            <div class="loading" style="display: block;">
                Searching...
            </div>
        </div>
    </div>

    <script>
        const queryInput = document.getElementById('query');
        const searchView = document.getElementById('searchView');
        const resultsView = document.getElementById('resultsView');
        const loadingView = document.getElementById('loadingView');
        const searchError = document.getElementById('searchError');
        const resultsList = document.getElementById('resultsList');
        const resultsHeader = document.getElementById('resultsHeader');
        const applyBtn = document.getElementById('applyBtn');

        let selectedMrn = null;
        let searchResults = [];

        // focus input on load
        queryInput.focus();

        // allow Enter to submit
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
            resultsHeader.textContent = `Found ${searchResults.length} patient${searchResults.length === 1 ? '' : 's'}:`;
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

            // update selection UI
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
                // on success, the extension will close the window
            } catch (err) {
                showError('Failed to connect to extension');
                showView(resultsView);
            }
        }

        function backToSearch() {
            selectedMrn = null;
            searchResults = [];
            showView(searchView);
            queryInput.focus();
        }

        async function cancel() {
            try {
                await fetch('/api/cancel', { method: 'POST' });
            } catch (err) {
                // ignore errors on cancel
            }
            // extension will close the window
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
        # log to extension stderr
        log(f"HTTP: {args[0]}")


def find_free_port():
    """Find an available port."""
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


# ============================================================================
# Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initializing with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": EXTENSION_NAME,
            "version": EXTENSION_VERSION,
            "description": "Look up patients and populate HL7 messages",
            "authors": ["Example Author <author@example.com>"],
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
                    "command": "wizard/patientLookup",
                    "group": "wizards"
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
                            },
                            {
                                "field": 8,
                                "note": "Administrative sex",
                                "values": {
                                    "M": "Male",
                                    "F": "Female",
                                    "O": "Other",
                                    "U": "Unknown"
                                }
                            }
                        ]
                    }
                }
            }
        }
    }


def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    # check if we recognise this command
    if command != "wizard/patientLookup":
        log(f"Unknown command: {command}")
        return

    # start async work in background thread
    thread = threading.Thread(target=execute_patient_lookup)
    thread.start()


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

        # wait for user interaction
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
        close_wizard_window()
        stop_http_server()


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


# ============================================================================
# Main Loop
# ============================================================================

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
