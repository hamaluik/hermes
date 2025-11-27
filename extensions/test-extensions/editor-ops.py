#!/usr/bin/env python3
"""
Editor Operations Test Extension

Tests all editor API methods with different formats.
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
    sys.stderr.write(f"[editor-ops] {message}\n")
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

def execute_get_message(fmt):
    """Get message in specified format."""
    log(f"Getting message in {fmt} format")

    response = send_request("editor/getMessage", {"format": fmt})

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    message = result.get("message", "")
    has_file = result.get("hasFile", False)
    file_path = result.get("filePath")

    log(f"hasFile: {has_file}")
    if file_path:
        log(f"filePath: {file_path}")

    # show first 200 chars of message
    preview = message[:200] + ("..." if len(message) > 200 else "")
    log(f"message preview: {preview}")


def execute_patch():
    """Patch PID.5.1 to TEST."""
    log("Patching PID.5.1 to 'TEST'")

    response = send_request("editor/patchMessage", {
        "patches": [
            {"path": "PID.5.1", "value": "TEST"}
        ]
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    success = result.get("success", False)
    patches_applied = result.get("patchesApplied", 0)

    log(f"success: {success}, patchesApplied: {patches_applied}")

    if not success:
        errors = result.get("errors", [])
        for err in errors:
            log(f"Patch error: {err.get('message')} at {err.get('path')}")


def execute_set():
    """Set a simple test message."""
    log("Setting test message")

    test_message = "MSH|^~\\&|TEST|FAC|||20231215120000||ADT^A01|123|P|2.5.1\rPID|1||12345||TEST^PATIENT||19800101|M"

    response = send_request("editor/setMessage", {
        "message": test_message,
        "format": "hl7"
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    success = result.get("success", False)

    log(f"success: {success}")

    if not success:
        error = result.get("error", "Unknown error")
        log(f"Set failed: {error}")


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
            "name": "Editor Operations Test",
            "version": "1.0.0",
            "description": "Tests editor API methods",
            "capabilities": {
                "commands": [
                    "editor-test/get-hl7",
                    "editor-test/get-json",
                    "editor-test/get-yaml",
                    "editor-test/get-toml",
                    "editor-test/patch",
                    "editor-test/set"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "editor-get-hl7",
                    "label": "Get HL7",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                        <polyline points="14 2 14 8 20 8"/>
                    </svg>""",
                    "command": "editor-test/get-hl7"
                },
                {
                    "id": "editor-get-json",
                    "label": "Get JSON",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                        <polyline points="14 2 14 8 20 8"/>
                    </svg>""",
                    "command": "editor-test/get-json"
                },
                {
                    "id": "editor-patch",
                    "label": "Patch Field",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                    </svg>""",
                    "command": "editor-test/patch"
                },
                {
                    "id": "editor-set",
                    "label": "Set Message",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                        <polyline points="14 2 14 8 20 8"/>
                        <line x1="12" y1="18" x2="12" y2="12"/>
                        <line x1="9" y1="15" x2="15" y2="15"/>
                    </svg>""",
                    "command": "editor-test/set"
                }
            ]
        }
    }


def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "editor-test/get-hl7":
        execute_get_message("hl7")
    elif command == "editor-test/get-json":
        execute_get_message("json")
    elif command == "editor-test/get-yaml":
        execute_get_message("yaml")
    elif command == "editor-test/get-toml":
        execute_get_message("toml")
    elif command == "editor-test/patch":
        execute_patch()
    elif command == "editor-test/set":
        execute_set()
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
