#!/usr/bin/env python3
"""
Dialog Test Extension

Tests all dialog API methods.
"""

import sys
import json

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
    sys.stderr.write(f"[dialog-test] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Request Helpers
# ============================================================================

_next_id = 1
_pending = {}


def send_request(method, params):
    """Send a request to Hermes and wait for response."""
    global _next_id

    request_id = _next_id
    _next_id += 1

    write_message({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params
    })

    # read until we get our response
    while True:
        msg = read_message()
        if msg is None:
            raise Exception("Connection closed")

        # response to our request?
        if "result" in msg or "error" in msg:
            if msg.get("id") == request_id:
                return msg
            else:
                # response to different request, store it
                _pending[msg.get("id")] = msg
        else:
            # request from Hermes, handle it
            response = handle_message(msg)
            if response:
                write_message(response)

    return _pending.pop(request_id)


# ============================================================================
# Command Handlers
# ============================================================================

def execute_message(kind):
    """Show message dialog."""
    log(f"Showing {kind} message")

    messages = {
        "info": "This is an information message.",
        "warning": "This is a warning message.",
        "error": "This is an error message."
    }

    response = send_request("ui/showMessage", {
        "message": messages.get(kind, "Test message"),
        "title": f"{kind.title()} Test",
        "kind": kind
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    acknowledged = result.get("acknowledged", False)
    log(f"Message acknowledged: {acknowledged}")


def execute_confirm():
    """Show confirmation dialog."""
    log("Showing confirmation dialog")

    response = send_request("ui/showConfirm", {
        "message": "Do you want to proceed with the test?",
        "title": "Confirm Test",
        "buttons": "yesNo"
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    confirmed = result.get("confirmed", False)
    log(f"User confirmed: {confirmed}")


def execute_open_file():
    """Show file open dialog."""
    log("Showing open file dialog")

    response = send_request("ui/openFile", {
        "title": "Select Test File",
        "filters": [
            {"name": "HL7 Files", "extensions": ["hl7", "txt"]},
            {"name": "All Files", "extensions": ["*"]}
        ]
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    path = result.get("path")

    if path:
        log(f"Selected file: {path}")
    else:
        log("User cancelled")


def execute_open_files():
    """Show multiple file open dialog."""
    log("Showing open files dialog")

    response = send_request("ui/openFiles", {
        "title": "Select Multiple Test Files",
        "filters": [
            {"name": "HL7 Files", "extensions": ["hl7"]}
        ]
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    paths = result.get("paths")

    if paths:
        log(f"Selected {len(paths)} files:")
        for path in paths:
            log(f"  - {path}")
    else:
        log("User cancelled")


def execute_save_file():
    """Show save file dialog."""
    log("Showing save file dialog")

    response = send_request("ui/saveFile", {
        "title": "Save Test File",
        "defaultName": "test.hl7",
        "filters": [
            {"name": "HL7 Files", "extensions": ["hl7"]}
        ]
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    path = result.get("path")

    if path:
        log(f"Save path: {path}")
    else:
        log("User cancelled")


def execute_select_directory():
    """Show directory selection dialog."""
    log("Showing select directory dialog")

    response = send_request("ui/selectDirectory", {
        "title": "Select Test Directory"
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    path = result.get("path")

    if path:
        log(f"Selected directory: {path}")
    else:
        log("User cancelled")


# ============================================================================
# Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Dialog Test",
            "version": "1.0.0",
            "description": "Tests dialog API methods",
            "capabilities": {
                "commands": [
                    "dialog-test/message-info",
                    "dialog-test/message-warn",
                    "dialog-test/message-error",
                    "dialog-test/confirm",
                    "dialog-test/open-file",
                    "dialog-test/open-files",
                    "dialog-test/save-file",
                    "dialog-test/select-dir"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "dialog-message-info",
                    "label": "Info Message",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"/>
                        <line x1="12" y1="16" x2="12" y2="12"/>
                        <line x1="12" y1="8" x2="12.01" y2="8"/>
                    </svg>""",
                    "command": "dialog-test/message-info"
                },
                {
                    "id": "dialog-confirm",
                    "label": "Confirm",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M9 11l3 3L22 4"/>
                        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
                    </svg>""",
                    "command": "dialog-test/confirm"
                },
                {
                    "id": "dialog-open-file",
                    "label": "Open File",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                    </svg>""",
                    "command": "dialog-test/open-file"
                },
                {
                    "id": "dialog-save-file",
                    "label": "Save File",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
                        <polyline points="17 21 17 13 7 13 7 21"/>
                        <polyline points="7 3 7 8 15 8"/>
                    </svg>""",
                    "command": "dialog-test/save-file"
                }
            ]
        }
    }


def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "dialog-test/message-info":
        execute_message("info")
    elif command == "dialog-test/message-warn":
        execute_message("warning")
    elif command == "dialog-test/message-error":
        execute_message("error")
    elif command == "dialog-test/confirm":
        execute_confirm()
    elif command == "dialog-test/open-file":
        execute_open_file()
    elif command == "dialog-test/open-files":
        execute_open_files()
    elif command == "dialog-test/save-file":
        execute_save_file()
    elif command == "dialog-test/select-dir":
        execute_select_directory()
    else:
        log(f"Unknown command: {command}")


def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
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

    # notifications
    if request_id is None:
        if method == "command/execute":
            handle_command(params)
        else:
            log(f"Unknown notification: {method}")
        return None

    # requests
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

            response = handle_message(msg)
            if response:
                write_message(response)

        except Exception as e:
            log(f"Error: {e}")
            break

    log("Exiting")


if __name__ == "__main__":
    main()
