# Dialog Extension Example

This example demonstrates the dialog API with a practical use case: loading patient
data from a JSON file and patching it into the current HL7 message.

## What It Does

1. Registers a toolbar button with a folder-open icon
2. When clicked, shows a file open dialog filtered to `.json` files
3. If user cancels the dialog, does nothing (cancellation is not an error)
4. If user selects a file, reads the JSON content
5. Patches the PID segment with patient data from the file
6. Shows a success or error notification dialog to the user

## Sample Data File

Create a file called `patient.json` with this structure:

```json
{
  "mrn": "12345678",
  "lastName": "DOE",
  "firstName": "JOHN",
  "dob": "19800115",
  "sex": "M"
}
```

## Python Implementation

Save this as `dialog_extension.py` and make it executable (`chmod +x dialog_extension.py`):

```python
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
```

## Key Concepts Demonstrated

### 1. File Dialog with Filters

```python
response = send_request("ui/openFile", {
    "title": "Select Patient File",
    "filters": [
        {"name": "JSON Files", "extensions": ["json"]}
    ]
})
```

The `filters` array restricts which files are shown. Each filter has:
- `name`: Display name shown in the dialog (e.g., "JSON Files")
- `extensions`: Array of extensions without dots (e.g., `["json"]`)

### 2. Handling Cancellation vs Errors

```python
# check for dialog error (system failure)
if "error" in response:
    log(f"Dialog error: {response['error']['message']}")
    return

# check if user cancelled (not an error, just null path)
path = response.get("result", {}).get("path")
if path is None:
    log("File selection cancelled")
    return
```

Dialog cancellation returns `null` for the path, not an error. This distinction
lets you handle "user changed their mind" differently from "something went wrong".

### 3. Reading Files After Dialog

```python
try:
    with open(path, "r", encoding="utf-8") as f:
        patient = json.load(f)
except FileNotFoundError:
    log(f"File not found: {path}")
    return
except json.JSONDecodeError as e:
    log(f"Invalid JSON: {e}")
    return
```

After getting the file path from the dialog, use standard Python file I/O.
The path is an absolute path on the user's filesystem.

### 4. Message Dialogs for Feedback

```python
def show_message(message, title=None, kind="info"):
    """Show a message dialog to the user."""
    params = {"message": message}
    if title:
        params["title"] = title
    if kind:
        params["kind"] = kind
    send_request("ui/showMessage", params)
```

The `kind` parameter controls the dialog icon and styling:
- `"info"` - Information icon (default)
- `"warning"` - Warning icon
- `"error"` - Error icon

Use message dialogs to provide feedback after operations complete:

```python
# on success
show_message(f"Patient loaded: {name}", "Success", "info")

# on error
show_message(f"Invalid JSON format:\n{e}", "Load Failed", "error")
```

### 5. Building Patches Dynamically

```python
patches = []

if "mrn" in patient:
    patches.append({"path": "PID.3.1", "value": patient["mrn"]})
if "lastName" in patient:
    patches.append({"path": "PID.5.1", "value": patient["lastName"]})
# ...
```

Only include patches for fields that exist in the input file. This makes the
extension more flexible—users can provide partial data.

## Testing

### Create a Test File

Save this as `test_patient.json`:

```json
{
  "mrn": "99887766",
  "lastName": "SMITH",
  "firstName": "ALICE",
  "dob": "19900525",
  "sex": "F"
}
```

### In Hermes

1. Add to settings:
   ```json
   {
     "extensions": [
       { "path": "/path/to/dialog_extension.py", "enabled": true }
     ]
   }
   ```

2. Restart Hermes (or click "Reload Extensions")

3. Open or create an HL7 message with a PID segment

4. Click the folder icon in the toolbar

5. Select your `test_patient.json` file

6. Verify PID fields are populated:
   - PID.3.1 = 99887766
   - PID.5.1 = SMITH
   - PID.5.2 = ALICE
   - PID.7 = 19900525
   - PID.8 = F

### Testing Cancellation

1. Click the toolbar button
2. Click Cancel in the file dialog
3. Check extension logs—should show "File selection cancelled"
4. Message should be unchanged (no dialog shown)

### Testing Error Dialog

1. Create a file with invalid JSON (e.g., `{ invalid }`)
2. Click the toolbar button and select the invalid file
3. An error dialog should appear with "Invalid JSON format"

## Other Dialog Methods

This example uses `ui/openFile`. The same patterns apply to other dialogs:

| Method               | Returns                        | Use Case                     |
|----------------------|--------------------------------|------------------------------|
| `ui/openFile`        | `{path: string \| null}`       | Select one file              |
| `ui/openFiles`       | `{paths: string[] \| null}`    | Select multiple files        |
| `ui/saveFile`        | `{path: string \| null}`       | Choose save location         |
| `ui/selectDirectory` | `{path: string \| null}`       | Select a folder              |
| `ui/showMessage`     | `{acknowledged: boolean}`      | Display info/warning/error   |
| `ui/showConfirm`     | `{confirmed: boolean}`         | Yes/No or OK/Cancel prompt   |

All file/directory dialogs return `null` when cancelled.

## Next Steps

- Add `ui/showConfirm` before overwriting existing patient data
- Use `ui/saveFile` to export patient data to a file
- Support multiple file formats (JSON, CSV) with different filters
- Progress to the [Wizard Example](wizard.md) for a complete web UI extension
