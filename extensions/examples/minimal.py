#!/usr/bin/env python3
"""
Minimal Hermes Extension Example

Adds a toolbar button that sets a sample patient name.
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
    sys.stderr.write(f"[minimal-ext] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Request Helpers
# ============================================================================

# Track our outgoing request IDs
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
            response = handle_request(msg)
            if response:
                write_message(response)

    return _pending.pop(request_id)


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
            "name":
            "Minimal Extension",
            "version":
            "1.0.0",
            "description":
            "A minimal example extension",
            "capabilities": {
                "commands": ["minimal/setPatient"]
            },
            "toolbarButtons": [{
                "id": "minimal-set-patient",
                "label": "Set Sample Patient",
                "icon":
                """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                        <circle cx="12" cy="7" r="4"/>
                    </svg>""",
                "command": "minimal/setPatient",
            }],
        },
    }


def handle_command(request_id, params):
    """Handle a command execution request."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "minimal/setPatient":
        return handle_set_patient(request_id)
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32009,
                "message": "Command not found",
                "data": f"Unknown command: {command}",
            },
        }


def handle_set_patient(request_id):
    """Set a sample patient name in the message."""

    # patch the message with sample data
    response = send_request(
        "editor/patchMessage",
        {
            "patches": [
                {
                    "path": "PID.5.1",
                    "value": "DOE"
                },
                {
                    "path": "PID.5.2",
                    "value": "JOHN"
                },
            ]
        },
    )

    if "error" in response:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "success":
                False,
                "message":
                f"Failed to patch message: {response['error']['message']}",
            },
        }

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "success": False,
                "message": f"Patch failed: {error_msg}"
            },
        }

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "success": True,
            "message": "Patient name set to DOE^JOHN"
        },
    }


def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
    return {"jsonrpc": "2.0", "id": request_id, "result": {"success": True}}


def handle_request(msg):
    """Route a request to the appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    if method == "initialize":
        return handle_initialize(request_id, params)
    elif method == "shutdown":
        response = handle_shutdown(request_id, params)
        write_message(response)
        sys.exit(0)
    elif method == "command/execute":
        return handle_command(request_id, params)
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": f"Unknown method: {method}",
            },
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

            response = handle_request(msg)
            if response:
                write_message(response)

        except Exception as e:
            log(f"Error: {e}")
            break

    log("Exiting")


if __name__ == "__main__":
    main()
