#!/usr/bin/env python3
"""
Events Test Extension

Tests event notification subscriptions for message/opened, message/saved,
and message/changed events.
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
    sys.stderr.write(f"[events-test] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Event Counters
# ============================================================================

event_counts = {
    "message/opened": 0,
    "message/saved": 0,
    "message/changed": 0
}


# ============================================================================
# Event Handlers
# ============================================================================

def handle_message_opened(params):
    """Handle message/opened event."""
    event_counts["message/opened"] += 1

    file_path = params.get("filePath", "(none)")
    is_new = params.get("isNew", False)

    if is_new:
        log(f"Event: message/opened - new message created")
    else:
        log(f"Event: message/opened - file: {file_path}")


def handle_message_saved(params):
    """Handle message/saved event."""
    event_counts["message/saved"] += 1

    file_path = params.get("filePath", "(none)")
    save_as = params.get("saveAs", False)

    operation = "Save As" if save_as else "Save"
    log(f"Event: message/saved - {operation} to: {file_path}")


def handle_message_changed(params):
    """Handle message/changed event."""
    event_counts["message/changed"] += 1

    has_file = params.get("hasFile", False)
    file_path = params.get("filePath", "(none)")
    message = params.get("message", "")
    msg_format = params.get("format", "hl7")

    # show preview of message content (first 100 chars, single line)
    preview = message[:100].replace("\r", " ").replace("\n", " ")
    if len(message) > 100:
        preview += "..."

    file_info = f"file: {file_path}" if has_file else "untitled"
    log(f"Event: message/changed - {file_info}")
    log(f"  Format: {msg_format}, Length: {len(message)} chars")
    log(f"  Preview: {preview}")


# ============================================================================
# Command Handlers
# ============================================================================

def handle_status():
    """Log current event counts."""
    log("Event counts:")
    for event, count in event_counts.items():
        log(f"  {event}: {count}")


# ============================================================================
# Protocol Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Events Test",
            "version": "1.0.0",
            "description": "Tests event notification subscriptions",
            "capabilities": {
                "commands": ["events-test/status"],
                "events": [
                    {"name": "message/opened"},
                    {"name": "message/saved"},
                    {
                        "name": "message/changed",
                        "options": {"includeContent": True, "format": "hl7"}
                    }
                ]
            },
            "toolbarButtons": [
                {
                    "id": "events-status",
                    "label": "Event Status",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
                    </svg>""",
                    "command": "events-test/status"
                }
            ]
        }
    }


def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "events-test/status":
        handle_status()
    else:
        log(f"Unknown command: {command}")


def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
    log(f"Final event counts:")
    for event, count in event_counts.items():
        log(f"  {event}: {count}")

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
        elif method == "message/opened":
            handle_message_opened(params)
        elif method == "message/saved":
            handle_message_saved(params)
        elif method == "message/changed":
            handle_message_changed(params)
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
    log("Subscribed to: message/opened, message/saved, message/changed")

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
