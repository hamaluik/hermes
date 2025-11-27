#!/usr/bin/env python3
"""
Window Management Test Extension

Tests window opening and closing via the API.
"""

import sys
import json
import threading
from http.server import HTTPServer, BaseHTTPRequestHandler
import socket

# ============================================================================
# Global State
# ============================================================================

http_server = None
http_port = None
window_id = None

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
    sys.stderr.write(f"[window-test] {message}\n")
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
# HTTP Server
# ============================================================================

def get_test_html():
    """Return test HTML page."""
    return """<!DOCTYPE html>
<html>
<head>
    <title>Window Test</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            padding: 40px;
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
            margin: 0 0 16px 0;
            font-size: 20px;
            colour: #333;
        }
        p {
            colour: #666;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Window Test</h1>
        <p>This is a test window opened by the extension.</p>
        <p>You can close it manually or via the "Close Window" button.</p>
    </div>
</body>
</html>"""


class TestHandler(BaseHTTPRequestHandler):
    """HTTP request handler for test page."""

    def do_GET(self):
        if self.path == "/" or self.path == "/test":
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.end_headers()
            self.wfile.write(get_test_html().encode("utf-8"))
        else:
            self.send_error(404)

    def log_message(self, format, *args):
        # suppress HTTP logging
        pass


def find_free_port():
    """Find an available port."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def start_http_server():
    """Start HTTP server for test page."""
    global http_server, http_port

    http_port = find_free_port()
    http_server = HTTPServer(("127.0.0.1", http_port), TestHandler)

    thread = threading.Thread(target=http_server.serve_forever)
    thread.daemon = True
    thread.start()

    log(f"HTTP server started on port {http_port}")


def stop_http_server():
    """Stop HTTP server."""
    global http_server
    if http_server:
        log("Stopping HTTP server")
        http_server.shutdown()
        http_server = None


# ============================================================================
# Command Handlers
# ============================================================================

def execute_open_window():
    """Open a test window."""
    global window_id

    log("Opening window")

    if not http_server:
        start_http_server()

    response = send_request("ui/openWindow", {
        "url": f"http://127.0.0.1:{http_port}/test",
        "title": "Window Test",
        "width": 500,
        "height": 300,
        "modal": False,
        "resizable": True
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    window_id = result.get("windowId")
    log(f"Opened window: {window_id}")


def execute_close_window():
    """Close the test window."""
    global window_id

    if not window_id:
        log("No window to close")
        return

    log(f"Closing window: {window_id}")

    response = send_request("ui/closeWindow", {
        "windowId": window_id
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    success = result.get("success", False)
    log(f"Close success: {success}")

    if success:
        window_id = None


def execute_open_modal():
    """Open a modal test window."""
    log("Opening modal window")

    if not http_server:
        start_http_server()

    response = send_request("ui/openWindow", {
        "url": f"http://127.0.0.1:{http_port}/test",
        "title": "Modal Window Test",
        "width": 400,
        "height": 250,
        "modal": True
    })

    if "error" in response:
        log(f"Error: {response['error']['message']}")
        return

    result = response.get("result", {})
    modal_id = result.get("windowId")
    log(f"Opened modal window: {modal_id}")


# ============================================================================
# Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    # start server immediately
    start_http_server()

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Window Test",
            "version": "1.0.0",
            "description": "Tests window management API",
            "capabilities": {
                "commands": [
                    "window-test/open",
                    "window-test/close",
                    "window-test/open-modal"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "window-open",
                    "label": "Open Window",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
                        <line x1="3" y1="9" x2="21" y2="9"/>
                    </svg>""",
                    "command": "window-test/open"
                },
                {
                    "id": "window-close",
                    "label": "Close Window",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="18" y1="6" x2="6" y2="18"/>
                        <line x1="6" y1="6" x2="18" y2="18"/>
                    </svg>""",
                    "command": "window-test/close"
                },
                {
                    "id": "window-modal",
                    "label": "Open Modal",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="5" y="5" width="14" height="14" rx="2" ry="2"/>
                        <line x1="5" y1="11" x2="19" y2="11"/>
                    </svg>""",
                    "command": "window-test/open-modal"
                }
            ]
        }
    }


def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "window-test/open":
        execute_open_window()
    elif command == "window-test/close":
        execute_close_window()
    elif command == "window-test/open-modal":
        execute_open_modal()
    else:
        log(f"Unknown command: {command}")


def handle_window_closed(params):
    """Handle window closed notification."""
    global window_id
    closed_id = params.get("windowId")
    reason = params.get("reason", "unknown")

    log(f"Window closed: {closed_id} (reason: {reason})")

    if closed_id == window_id:
        window_id = None


def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
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

    # notifications
    if request_id is None:
        if method == "command/execute":
            handle_command(params)
        elif method == "window/closed":
            handle_window_closed(params)
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
            import traceback
            traceback.print_exc(file=sys.stderr)
            break

    stop_http_server()
    log("Exiting")


if __name__ == "__main__":
    main()
