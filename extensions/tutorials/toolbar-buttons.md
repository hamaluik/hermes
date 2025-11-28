# Tutorial: Adding Toolbar Buttons

This tutorial builds on [Your First Extension](first-extension.md) to add
multiple toolbar buttons with custom SVG icons.

## What You'll Build

An extension with two toolbar buttons:
- **Clear Patient** - Removes all PID.5 (name) data
- **Fill Test Data** - Populates PID with realistic test patient information

## What You'll Learn

- Creating SVG icons that match Hermes' visual style
- Registering multiple toolbar buttons
- Routing multiple commands to different handlers
- Reading the current message before modifying it

## Prerequisites

Start with the complete code from [Your First Extension](first-extension.md).
You'll modify that extension to add the new features.

## Step 1: Design Your Icons

Toolbar buttons need SVG icons. Hermes icons use a consistent style:
- **24×24 viewBox** for consistent sizing
- **stroke="currentColor"** so icons match the theme
- **Simple shapes** that work at small sizes
- **2px stroke width** for consistency

Add these icon constants at the top of your file, after the imports:

```python
# ============================================================================
# Icons
# ============================================================================

ICON_CLEAR = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <polyline points="3 6 5 6 21 6"/>
    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
    <line x1="10" y1="11" x2="10" y2="17"/>
    <line x1="14" y1="11" x2="14" y2="17"/>
</svg>"""

ICON_FILL = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
    <circle cx="12" cy="7" r="4"/>
    <path d="M16 11h6"/>
    <path d="M19 8v6"/>
</svg>"""
```

You can find free icons at [Lucide](https://lucide.dev/) or
[Feather Icons](https://feathericons.com/).

## Step 2: Register Both Buttons

Update your `handle_initialize()` function to register both commands and
buttons:

```python
def handle_initialize(request_id, params):
    """Handle the initialize handshake."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Patient Tools",
            "version": "1.0.0",
            "description": "Clear and fill patient data",
            "capabilities": {
                "commands": [
                    "patientTools/clear",
                    "patientTools/fill"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "patient-clear",
                    "label": "Clear Patient Data",
                    "icon": ICON_CLEAR,
                    "command": "patientTools/clear"
                },
                {
                    "id": "patient-fill",
                    "label": "Fill Test Data",
                    "icon": ICON_FILL,
                    "command": "patientTools/fill"
                }
            ]
        }
    }
```

Key points:
- List both commands in `capabilities.commands`
- Each button needs a unique `id` and `command`
- The `label` appears on hover

## Step 3: Route Commands to Handlers

Update `handle_command()` to route to different functions based on the command:

```python
def handle_command(params):
    """Handle a command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "patientTools/clear":
        execute_clear()
    elif command == "patientTools/fill":
        execute_fill()
    else:
        log(f"Unknown command: {command}")
```

This simple if/elif chain works for a few commands. For many commands, consider
using a dictionary mapping command names to functions.

## Step 4: Implement the Clear Command

Add this function to clear patient name data:

```python
def execute_clear():
    """Clear patient name data."""
    log("Clearing patient data...")

    response = send_request("editor/patchMessage", {
        "patches": [
            {"path": "PID.5.1", "value": ""},  # family name
            {"path": "PID.5.2", "value": ""},  # given name
            {"path": "PID.5.3", "value": ""},  # middle name
        ]
    })

    if "error" in response:
        log(f"Failed to clear data: {response['error']['message']}")
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        log(f"Clear failed: {error_msg}")
        return

    log("Patient data cleared")
```

Setting fields to empty strings effectively removes them from the message.

## Step 5: Implement the Fill Command

Add this function to populate multiple fields with test data:

```python
def execute_fill():
    """Fill in test patient data."""
    log("Filling test patient data...")

    patches = [
        {"path": "PID.3.1", "value": "TEST123456"},        # MRN
        {"path": "PID.5.1", "value": "TESTPATIENT"},       # family name
        {"path": "PID.5.2", "value": "JANE"},              # given name
        {"path": "PID.5.3", "value": "Q"},                 # middle initial
        {"path": "PID.7", "value": "19850615"},            # DOB
        {"path": "PID.8", "value": "F"},                   # sex
        {"path": "PID.11.1", "value": "123 TEST ST"},      # street
        {"path": "PID.11.3", "value": "TESTCITY"},         # city
        {"path": "PID.11.4", "value": "ON"},               # province
        {"path": "PID.11.5", "value": "K1A 0A1"},          # postal code
        {"path": "PID.13.1", "value": "6135551234"},       # phone
    ]

    response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in response:
        log(f"Failed to fill data: {response['error']['message']}")
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        log(f"Fill failed: {error_msg}")
        return

    log("Test patient data filled")
```

The patches are applied atomically—either all succeed or all fail.

## Testing

1. Add your extension to Hermes settings
2. Reload extensions
3. Look for two new buttons in the toolbar
4. Create or open an HL7 message with a PID segment
5. Click "Fill Test Data" and verify fields are populated
6. Click "Clear Patient Data" and verify name fields are emptied

## Going Further: Conditional Logic

Make the fill button smarter by only filling empty fields. Update
`execute_fill()` to read the message first:

```python
def execute_fill():
    """Fill in test patient data (only empty fields)."""
    log("Filling test patient data...")

    # read the current message first
    get_response = send_request("editor/getMessage", {"format": "hl7"})

    if "error" in get_response:
        log(f"Failed to read message: {get_response['error']['message']}")
        return

    current_message = get_response.get("result", {}).get("message", "")

    # check if PID.5 already has content
    if "PID|" in current_message:
        lines = current_message.split("\r")
        for line in lines:
            if line.startswith("PID|"):
                fields = line.split("|")
                # PID.5 is the 6th field (index 5)
                if len(fields) > 5 and fields[5].strip():
                    log("Patient name already exists, skipping fill")
                    return

    # if we get here, it's safe to fill
    patches = [
        {"path": "PID.3.1", "value": "TEST123456"},
        {"path": "PID.5.1", "value": "TESTPATIENT"},
        {"path": "PID.5.2", "value": "JANE"},
        # ... rest of patches
    ]

    response = send_request("editor/patchMessage", {"patches": patches})
    # ... error handling
```

This pattern—read first, then decide whether to modify—is common in extensions.

## What You've Learned

- **Creating SVG icons** that match Hermes' visual style
- **Registering multiple buttons** in a single extension
- **Routing commands** to different handler functions
- **Reading the current message** before modifying it
- **Applying multiple patches** in a single request

## Next Steps

Continue to [Using Dialogs](dialogs.md) to learn how to show messages, ask for
confirmation, and open file dialogs.

## Complete Code

```python
#!/usr/bin/env python3
"""
Patient Tools Extension

Provides buttons for clearing and filling patient data.
"""

import sys
import json

# ============================================================================
# Icons
# ============================================================================

ICON_CLEAR = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <polyline points="3 6 5 6 21 6"/>
    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
    <line x1="10" y1="11" x2="10" y2="17"/>
    <line x1="14" y1="11" x2="14" y2="17"/>
</svg>"""

ICON_FILL = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
    <circle cx="12" cy="7" r="4"/>
    <path d="M16 11h6"/>
    <path d="M19 8v6"/>
</svg>"""

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
    """Log to stderr (visible in Hermes logs)."""
    sys.stderr.write(f"[patient-tools] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Request Helpers
# ============================================================================

_next_id = 1

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

    while True:
        msg = read_message()
        if msg is None:
            raise Exception("Connection closed")

        if "result" in msg or "error" in msg:
            if msg.get("id") == request_id:
                return msg


# ============================================================================
# Command Implementations
# ============================================================================

def execute_clear():
    """Clear patient name data."""
    log("Clearing patient data...")

    response = send_request("editor/patchMessage", {
        "patches": [
            {"path": "PID.5.1", "value": ""},
            {"path": "PID.5.2", "value": ""},
            {"path": "PID.5.3", "value": ""},
        ]
    })

    if "error" in response:
        log(f"Failed to clear data: {response['error']['message']}")
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        log(f"Clear failed: {error_msg}")
        return

    log("Patient data cleared")


def execute_fill():
    """Fill in test patient data."""
    log("Filling test patient data...")

    patches = [
        {"path": "PID.3.1", "value": "TEST123456"},
        {"path": "PID.5.1", "value": "TESTPATIENT"},
        {"path": "PID.5.2", "value": "JANE"},
        {"path": "PID.5.3", "value": "Q"},
        {"path": "PID.7", "value": "19850615"},
        {"path": "PID.8", "value": "F"},
        {"path": "PID.11.1", "value": "123 TEST ST"},
        {"path": "PID.11.3", "value": "TESTCITY"},
        {"path": "PID.11.4", "value": "ON"},
        {"path": "PID.11.5", "value": "K1A 0A1"},
        {"path": "PID.13.1", "value": "6135551234"},
    ]

    response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in response:
        log(f"Failed to fill data: {response['error']['message']}")
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        log(f"Fill failed: {error_msg}")
        return

    log("Test patient data filled")


# ============================================================================
# Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle the initialize handshake."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Patient Tools",
            "version": "1.0.0",
            "description": "Clear and fill patient data",
            "capabilities": {
                "commands": [
                    "patientTools/clear",
                    "patientTools/fill"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "patient-clear",
                    "label": "Clear Patient Data",
                    "icon": ICON_CLEAR,
                    "command": "patientTools/clear"
                },
                {
                    "id": "patient-fill",
                    "label": "Fill Test Data",
                    "icon": ICON_FILL,
                    "command": "patientTools/fill"
                }
            ]
        }
    }


def handle_command(params):
    """Handle a command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "patientTools/clear":
        execute_clear()
    elif command == "patientTools/fill":
        execute_fill()
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
    """Route a message to the appropriate handler."""
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
```
