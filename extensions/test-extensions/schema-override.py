#!/usr/bin/env python3
"""
Schema Override Test Extension

Tests schema merging by providing custom field definitions.
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
    sys.stderr.write(f"[schema-test] {message}\n")
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
            "name": "Schema Override Test",
            "version": "1.0.0",
            "description": "Tests schema merging",
            "capabilities": {
                "commands": ["schema-test/verify"],
                "schemaProvider": True
            },
            "toolbarButtons": [
                {
                    "id": "schema-verify",
                    "label": "Verify Schema",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M9 11l3 3L22 4"/>
                        <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
                    </svg>""",
                    "command": "schema-test/verify"
                }
            ],
            "schema": {
                "segments": {
                    "PID": {
                        "fields": [
                            {
                                "field": 3,
                                "component": 1,
                                "note": "8-digit MRN from Patient Master Index (test override)",
                                "required": True,
                                "minlength": 8,
                                "maxlength": 8,
                                "pattern": "^[0-9]{8}$",
                                "placeholder": "00000000"
                            },
                            {
                                "field": 3,
                                "component": 4,
                                "note": "Should always be 'MRN' (test override)",
                                "template": "MRN"
                            },
                            {
                                "field": 8,
                                "note": "Administrative sex (test override with custom values)",
                                "values": {
                                    "M": "Male (test)",
                                    "F": "Female (test)",
                                    "O": "Other (test)",
                                    "U": "Unknown (test)",
                                    "N": "Not stated (test)"
                                }
                            }
                        ]
                    },
                    "OBX": {
                        "fields": [
                            {
                                "field": 2,
                                "note": "Value type for observation (test override)",
                                "required": True,
                                "values": {
                                    "NM": "Numeric (test)",
                                    "ST": "String (test)",
                                    "TX": "Text (test)",
                                    "CE": "Coded Element (test)",
                                    "DT": "Date (test)",
                                    "TM": "Time (test)"
                                }
                            },
                            {
                                "field": 3,
                                "component": 1,
                                "note": "Observation identifier code (test override)",
                                "placeholder": "LOINC-12345"
                            },
                            {
                                "field": 11,
                                "note": "Observation result status (test override)",
                                "required": True,
                                "values": {
                                    "F": "Final (test)",
                                    "P": "Preliminary (test)",
                                    "C": "Corrected (test)",
                                    "X": "Cancelled (test)"
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

    if command == "schema-test/verify":
        log("Schema overrides have been loaded")
        log("Check field descriptions for PID.3.1, PID.3.4, PID.8, OBX.2, OBX.3.1, OBX.11")
        log("Look for '(test override)' or '(test)' in descriptions")
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
