# Tutorial: Using Dialogs

This tutorial builds on [Your First Extension](first-extension.md) to add
system-native dialogs for user interaction.

## What You'll Build

An extension with two toolbar buttons:
- **Load Patient** - Opens a file picker, loads patient data from JSON, and
  populates the HL7 message
- **Clear Patient** - Asks for confirmation, then clears patient fields

## What You'll Learn

- Showing informational messages, warnings, and errors
- Asking yes/no and ok/cancel questions
- Opening file selection dialogs with filters
- Handling user cancellation vs system errors
- Chaining dialogs in a workflow

## Prerequisites

Start with the complete code from [Your First Extension](first-extension.md).
You'll add dialog capabilities to that extension.

## Step 1: Add Dialog Helper Functions

Add these helper functions after your `send_request()` function:

```python
# ============================================================================
# Dialog Helpers
# ============================================================================

def show_info(message, title="Information"):
    """Show an informational message dialog."""
    send_request("ui/showMessage", {
        "message": message,
        "title": title,
        "kind": "info"
    })


def show_error(message, title="Error"):
    """Show an error message dialog."""
    send_request("ui/showMessage", {
        "message": message,
        "title": title,
        "kind": "error"
    })


def show_warning(message, title="Warning"):
    """Show a warning message dialog."""
    send_request("ui/showMessage", {
        "message": message,
        "title": title,
        "kind": "warning"
    })
```

The `kind` parameter controls the icon displayed:
- `"info"` - Blue circle with "i"
- `"warning"` - Yellow triangle
- `"error"` - Red circle with "x"

## Step 2: Add Confirmation Dialog Helper

The `ui/showConfirm` method asks yes/no or ok/cancel questions:

```python
def ask_confirm(message, title="Confirm"):
    """Ask the user for confirmation."""
    response = send_request("ui/showConfirm", {
        "message": message,
        "title": title,
        "buttons": "yesNo"  # or "okCancel"
    })

    if "error" in response:
        log(f"Confirm dialog error: {response['error']['message']}")
        return False

    return response.get("result", {}).get("confirmed", False)
```

The `buttons` parameter controls which buttons appear:
- `"yesNo"` - Yes and No buttons
- `"okCancel"` - OK and Cancel buttons

## Step 3: Add File Selection Dialog Helper

The `ui/openFile` method shows a native file picker:

```python
def select_file():
    """Show a file selection dialog."""
    response = send_request("ui/openFile", {
        "title": "Select Patient File",
        "filters": [
            {"name": "JSON Files", "extensions": ["json"]},
            {"name": "All Files", "extensions": ["*"]}
        ]
    })

    # system error (dialog failed to open)
    if "error" in response:
        log(f"File dialog error: {response['error']['message']}")
        return None

    # user cancellation returns None path, not an error
    return response.get("result", {}).get("path")
```

**Important**: User cancellation is **not an error**. When the user clicks Cancel,
the response is successful but `path` is `None`.

## Step 4: Add Icons

Add icons for the toolbar buttons after the imports:

```python
# ============================================================================
# Icons
# ============================================================================

ICON_LOAD = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
    <polyline points="12 13 12 17"/>
    <polyline points="9 14 12 11 15 14"/>
</svg>"""

ICON_CLEAR = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
    <circle cx="12" cy="7" r="4"/>
    <line x1="8" y1="11" x2="16" y2="3"/>
</svg>"""
```

## Step 5: Update Initialize Handler

Update `handle_initialize()` to register the new commands and buttons:

```python
def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Patient Loader",
            "version": "1.0.0",
            "description": "Load patient data from JSON files",
            "capabilities": {
                "commands": [
                    "patientLoader/load",
                    "patientLoader/clear"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "patient-load",
                    "label": "Load Patient from File",
                    "icon": ICON_LOAD,
                    "command": "patientLoader/load"
                },
                {
                    "id": "patient-clear",
                    "label": "Clear Patient Data",
                    "icon": ICON_CLEAR,
                    "command": "patientLoader/clear"
                }
            ]
        }
    }
```

## Step 6: Implement the Load Patient Command

This command demonstrates a multi-step dialog workflow:

```python
def execute_load_patient():
    """Load patient data from a JSON file."""
    log("Starting patient load workflow")

    # step 1: show file selection dialog
    path = select_file()

    if path is None:
        log("User cancelled file selection")
        return

    log(f"Selected file: {path}")

    # step 2: read and parse the file
    try:
        with open(path, "r", encoding="utf-8") as f:
            patient = json.load(f)
    except FileNotFoundError:
        show_error(f"File not found:\n{path}", "Load Failed")
        return
    except json.JSONDecodeError as e:
        show_error(f"Invalid JSON format:\n{e}", "Load Failed")
        return
    except Exception as e:
        show_error(f"Could not read file:\n{e}", "Load Failed")
        return

    # step 3: check if message already has patient data
    get_response = send_request("editor/getMessage", {"format": "hl7"})

    if "error" not in get_response:
        current_message = get_response.get("result", {}).get("message", "")

        if "PID|" in current_message:
            lines = current_message.split("\r")
            for line in lines:
                if line.startswith("PID|"):
                    fields = line.split("|")
                    if len(fields) > 5 and fields[5].strip():
                        # patient name exists, confirm overwrite
                        if not ask_confirm(
                            "The message already contains patient data.\n\n"
                            "Do you want to overwrite it?",
                            "Confirm Overwrite"
                        ):
                            log("User declined overwrite")
                            return

    # step 4: patch the message with patient data
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
        show_warning("The file did not contain any patient data.", "No Data")
        return

    patch_response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in patch_response:
        show_error(
            f"Failed to update message:\n{patch_response['error']['message']}",
            "Load Failed"
        )
        return

    if not patch_response.get("result", {}).get("success"):
        errors = patch_response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        show_error(f"Failed to update message:\n{error_msg}", "Load Failed")
        return

    # step 5: show success message
    name = f"{patient.get('firstName', '')} {patient.get('lastName', '')}".strip()
    show_info(f"Patient loaded successfully:\n{name}", "Load Complete")
    log(f"Loaded patient: {name}")
```

This workflow demonstrates:
1. File picker → Read file → Check existing data → Confirm overwrite → Patch →
   Show success

## Step 7: Implement the Clear Patient Command

```python
def execute_clear_patient():
    """Clear patient data after confirmation."""
    log("Starting patient clear workflow")

    # confirm the action first
    if not ask_confirm(
        "This will clear all patient identification data.\n\n"
        "Are you sure you want to continue?",
        "Clear Patient Data"
    ):
        log("User cancelled clear operation")
        return

    # clear the fields
    patches = [
        {"path": "PID.3.1", "value": ""},   # MRN
        {"path": "PID.5.1", "value": ""},   # last name
        {"path": "PID.5.2", "value": ""},   # first name
        {"path": "PID.5.3", "value": ""},   # middle name
        {"path": "PID.7", "value": ""},     # DOB
        {"path": "PID.8", "value": ""},     # sex
    ]

    response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in response:
        show_error(
            f"Failed to clear data:\n{response['error']['message']}",
            "Clear Failed"
        )
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        show_error(f"Failed to clear data:\n{error_msg}", "Clear Failed")
        return

    show_info("Patient data cleared.", "Clear Complete")
    log("Patient data cleared")
```

## Step 8: Update Command Routing

Update `handle_command()` to route to the new functions:

```python
def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "patientLoader/load":
        execute_load_patient()
    elif command == "patientLoader/clear":
        execute_clear_patient()
    else:
        log(f"Unknown command: {command}")
```

## Testing

### Create a Test Patient File

Create `test_patient.json`:

```json
{
    "mrn": "12345678",
    "firstName": "JOHN",
    "lastName": "DOE",
    "dob": "19800115",
    "sex": "M"
}
```

### Test the Load Button

1. Add your extension to Hermes settings
2. Click "Load Patient from File"
3. Select `test_patient.json`
4. Verify the patient data appears in the message

### Test Confirmation

1. With patient data loaded, click "Load Patient from File" again
2. Select the same file
3. You should see the confirmation dialog
4. Click "No" to cancel, or "Yes" to proceed

### Test the Clear Button

1. Click "Clear Patient Data"
2. Click "Yes" in the confirmation dialog
3. Verify the patient fields are now empty

### Test Error Handling

1. Click "Load Patient from File"
2. Select a non-JSON file
3. You should see an error dialog

## Dialog Types Summary

| Method               | Purpose                    | User Can Cancel |
|----------------------|----------------------------|-----------------|
| `ui/showMessage`     | Show info, warning, error  | No              |
| `ui/showConfirm`     | Ask yes/no or ok/cancel    | Yes             |
| `ui/openFile`        | Select a file to open      | Yes             |
| `ui/openFiles`       | Select multiple files      | Yes             |
| `ui/saveFile`        | Choose where to save       | Yes             |
| `ui/selectDirectory` | Choose a folder            | Yes             |

## What You've Learned

- **Showing messages** with different kinds (info, warning, error)
- **Asking for confirmation** with yes/no and ok/cancel buttons
- **Opening file dialogs** with type filters
- **Handling cancellation** separately from errors
- **Chaining dialogs** in a realistic workflow

## Next Steps

Continue to [Working with Messages](working-with-messages.md) to learn how to
read messages in different formats and perform complex transformations.

## Complete Code

```python
#!/usr/bin/env python3
"""
Patient Loader Extension

Loads patient data from JSON files using dialog interactions.
"""

import sys
import json

# ============================================================================
# Icons
# ============================================================================

ICON_LOAD = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
    <polyline points="12 13 12 17"/>
    <polyline points="9 14 12 11 15 14"/>
</svg>"""

ICON_CLEAR = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
    <circle cx="12" cy="7" r="4"/>
    <line x1="8" y1="11" x2="16" y2="3"/>
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
    sys.stderr.write(f"[patient-loader] {message}\n")
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
# Dialog Helpers
# ============================================================================

def show_info(message, title="Information"):
    """Show an informational message dialog."""
    send_request("ui/showMessage", {
        "message": message,
        "title": title,
        "kind": "info"
    })


def show_error(message, title="Error"):
    """Show an error message dialog."""
    send_request("ui/showMessage", {
        "message": message,
        "title": title,
        "kind": "error"
    })


def show_warning(message, title="Warning"):
    """Show a warning message dialog."""
    send_request("ui/showMessage", {
        "message": message,
        "title": title,
        "kind": "warning"
    })


def ask_confirm(message, title="Confirm"):
    """Ask the user for confirmation."""
    response = send_request("ui/showConfirm", {
        "message": message,
        "title": title,
        "buttons": "yesNo"
    })

    if "error" in response:
        log(f"Confirm dialog error: {response['error']['message']}")
        return False

    return response.get("result", {}).get("confirmed", False)


def select_file():
    """Show a file selection dialog."""
    response = send_request("ui/openFile", {
        "title": "Select Patient File",
        "filters": [
            {"name": "JSON Files", "extensions": ["json"]},
            {"name": "All Files", "extensions": ["*"]}
        ]
    })

    if "error" in response:
        log(f"File dialog error: {response['error']['message']}")
        return None

    return response.get("result", {}).get("path")


# ============================================================================
# Command Implementations
# ============================================================================

def execute_load_patient():
    """Load patient data from a JSON file."""
    log("Starting patient load workflow")

    path = select_file()

    if path is None:
        log("User cancelled file selection")
        return

    log(f"Selected file: {path}")

    try:
        with open(path, "r", encoding="utf-8") as f:
            patient = json.load(f)
    except FileNotFoundError:
        show_error(f"File not found:\n{path}", "Load Failed")
        return
    except json.JSONDecodeError as e:
        show_error(f"Invalid JSON format:\n{e}", "Load Failed")
        return
    except Exception as e:
        show_error(f"Could not read file:\n{e}", "Load Failed")
        return

    get_response = send_request("editor/getMessage", {"format": "hl7"})

    if "error" not in get_response:
        current_message = get_response.get("result", {}).get("message", "")

        if "PID|" in current_message:
            lines = current_message.split("\r")
            for line in lines:
                if line.startswith("PID|"):
                    fields = line.split("|")
                    if len(fields) > 5 and fields[5].strip():
                        if not ask_confirm(
                            "The message already contains patient data.\n\n"
                            "Do you want to overwrite it?",
                            "Confirm Overwrite"
                        ):
                            log("User declined overwrite")
                            return

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
        show_warning("The file did not contain any patient data.", "No Data")
        return

    patch_response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in patch_response:
        show_error(
            f"Failed to update message:\n{patch_response['error']['message']}",
            "Load Failed"
        )
        return

    if not patch_response.get("result", {}).get("success"):
        errors = patch_response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        show_error(f"Failed to update message:\n{error_msg}", "Load Failed")
        return

    name = f"{patient.get('firstName', '')} {patient.get('lastName', '')}".strip()
    show_info(f"Patient loaded successfully:\n{name}", "Load Complete")
    log(f"Loaded patient: {name}")


def execute_clear_patient():
    """Clear patient data after confirmation."""
    log("Starting patient clear workflow")

    if not ask_confirm(
        "This will clear all patient identification data.\n\n"
        "Are you sure you want to continue?",
        "Clear Patient Data"
    ):
        log("User cancelled clear operation")
        return

    patches = [
        {"path": "PID.3.1", "value": ""},
        {"path": "PID.5.1", "value": ""},
        {"path": "PID.5.2", "value": ""},
        {"path": "PID.5.3", "value": ""},
        {"path": "PID.7", "value": ""},
        {"path": "PID.8", "value": ""},
    ]

    response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in response:
        show_error(
            f"Failed to clear data:\n{response['error']['message']}",
            "Clear Failed"
        )
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        show_error(f"Failed to clear data:\n{error_msg}", "Clear Failed")
        return

    show_info("Patient data cleared.", "Clear Complete")
    log("Patient data cleared")


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
            "name": "Patient Loader",
            "version": "1.0.0",
            "description": "Load patient data from JSON files",
            "capabilities": {
                "commands": [
                    "patientLoader/load",
                    "patientLoader/clear"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "patient-load",
                    "label": "Load Patient from File",
                    "icon": ICON_LOAD,
                    "command": "patientLoader/load"
                },
                {
                    "id": "patient-clear",
                    "label": "Clear Patient Data",
                    "icon": ICON_CLEAR,
                    "command": "patientLoader/clear"
                }
            ]
        }
    }


def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "patientLoader/load":
        execute_load_patient()
    elif command == "patientLoader/clear":
        execute_clear_patient()
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
