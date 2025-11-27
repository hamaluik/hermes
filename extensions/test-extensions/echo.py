#!/usr/bin/env python3
"""
Echo Test Extension

Minimal test for verifying basic extension communication.
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
    """Log to stderr (visible in Hermes extension logs)."""
    sys.stderr.write(f"[echo-test] {message}\n")
    sys.stderr.flush()


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
            "name": "Echo Test",
            "version": "1.0.0",
            "description": "Minimal test extension",
            "capabilities": {
                "commands": ["echo/ping"]
            },
            "toolbarButtons": [
                {
                    "id": "echo-ping",
                    "label": "Echo Ping",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"/>
                        <path d="M12 6v6l4 2"/>
                    </svg>""",
                    "command": "echo/ping"
                }
            ]
        }
    }


def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "echo/ping":
        log("pong")
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

    # notifications (no id field)
    if request_id is None:
        if method == "command/execute":
            handle_command(params)
        else:
            log(f"Unknown notification: {method}")
        return None

    # requests (with id field)
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
