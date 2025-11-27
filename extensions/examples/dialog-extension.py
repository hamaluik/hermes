#!/usr/bin/env python3
"""
Dialog Extension Example

Demonstrates the dialog API by loading patient data from a JSON file.
"""

import sys
import json

# ============================================================================
# Message I/O
# ============================================================================

def read_message():
    """Read a JSON-RPC message from stdin."""
    # read headers
    headers = {}
    while True:
        line = sys.stdin.readline()
        if line == "\r\n" or line == "\n":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            headers[key.strip()] = value.strip()

    # read content
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
    """Log to stderr (visible in Hermes logs)."""
    sys.stderr.write(f"[dialog-ext] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Request Helpers
# ============================================================================

# track our outgoing request IDs
_next_id = 1
_pending = {}


def send_request(method, params):
    """Send a request to Hermes and wait for the response."""
    global _next_id

    request_id = _next_id
    _next_id += 1

    write_message({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params
    })

    # read messages until we get our response
    while True:
        msg = read_message()
        if msg is None:
            raise Exception("Connection closed")

        # is this a response to our request?
        if "result" in msg or "error" in msg:
            if msg.get("id") == request_id:
                return msg
            else:
                # response to a different request, store it
                _pending[msg.get("id")] = msg
        else:
            # this is a request from Hermes, we need to handle it
            response = handle_message(msg)
            if response:
                write_message(response)

    return _pending.pop(request_id)


def show_message(message, title=None, kind="info"):
    """Show a message dialog to the user."""
    params = {"message": message}
    if title:
        params["title"] = title
    if kind:
        params["kind"] = kind
    send_request("ui/showMessage", params)


# ============================================================================
# Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle the initialize handshake."""
    log(f"Initializing with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Dialog Extension",
            "version": "1.0.0",
            "description": "Load patient data from a JSON file",
            "capabilities": {
                "commands": ["dialog/loadPatient"]
            },
            "toolbarButtons": [
                {
                    "id": "dialog-load-patient",
                    "label": "Load Patient from File",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                        <line x1="12" y1="11" x2="12" y2="17"/>
                        <line x1="9" y1="14" x2="15" y2="14"/>
                    </svg>""",
                    "command": "dialog/loadPatient"
                }
            ]
        }
    }


def handle_command(params):
    """Handle a command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command != "dialog/loadPatient":
        log(f"Unknown command: {command}")
        return

    execute_load_patient()


def execute_load_patient():
    """Execute the loadPatient command using the file dialog."""
    # show file open dialog
    response = send_request("ui/openFile", {
        "title": "Select Patient File",
        "filters": [
            {"name": "JSON Files", "extensions": ["json"]}
        ]
    })

    # check for dialog error
    if "error" in response:
        log(f"Dialog error: {response['error']['message']}")
        return

    # check if user cancelled (path will be null)
    path = response.get("result", {}).get("path")
    if path is None:
        log("File selection cancelled")
        return

    log(f"Selected file: {path}")

    # read and parse the JSON file
    try:
        with open(path, "r", encoding="utf-8") as f:
            patient = json.load(f)
    except FileNotFoundError:
        log(f"File not found: {path}")
        show_message(f"File not found:\n{path}", "Load Failed", "error")
        return
    except json.JSONDecodeError as e:
        log(f"Invalid JSON: {e}")
        show_message(f"Invalid JSON format:\n{e}", "Load Failed", "error")
        return

    # build patches from patient data
    patches = []

    if "mrn" in patient:
        patches.append({"path": "PID.3.1", "value": patient["mrn"]})
    if "lastName" in patient:
        patches.append({"path": "PID.5.1", "value": patient["lastName"]})
    if "firstName" in patient:
        patches.append({"path": "PID.5.2", "value": patient["firstName"]})
    if "dob" in patient:
        patches.append({"path": "PID.7", "value": patient["dob"]})
    if "sex" in patient:
        patches.append({"path": "PID.8", "value": patient["sex"]})

    if not patches:
        log("No patient data found in file")
        show_message("No patient data found in file.", "Load Failed", "error")
        return

    # patch the message
    response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in response:
        log(f"Failed to patch message: {response['error']['message']}")
        show_message(
            f"Failed to update message:\n{response['error']['message']}",
            "Load Failed",
            "error"
        )
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        log(f"Patch failed: {error_msg}")
        show_message(f"Failed to update message:\n{error_msg}", "Load Failed", "error")
        return

    # show success notification
    name = f"{patient.get('firstName', '')} {patient.get('lastName', '')}".strip()
    log(f"Loaded patient: {name}")
    show_message(f"Patient loaded: {name}", "Success", "info")


def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    }


def handle_message(msg):
    """Route a message to the appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    # check if this is a notification (no id field)
    if request_id is None:
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
                "message": "Method not found",
                "data": f"Unknown method: {method}"
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

            response = handle_message(msg)
            if response:
                write_message(response)

        except Exception as e:
            log(f"Error: {e}")
            break

    log("Exiting")


if __name__ == "__main__":
    main()
