# Show Dialogs

This guide shows how to display system-native dialogs for user interaction.

## Show an Informational Message

```python
send_request("ui/showMessage", {
    "message": "Patient data imported successfully.",
    "title": "Import Complete",
    "kind": "info"
})
```

The dialog blocks until the user clicks OK. The response confirms
acknowledgement:

```python
response = send_request("ui/showMessage", {
    "message": "Operation complete.",
    "kind": "info"
})

# response["result"]["acknowledged"] is always True
```

## Show a Warning

```python
send_request("ui/showMessage", {
    "message": "Some fields were skipped due to validation errors.",
    "title": "Warning",
    "kind": "warning"
})
```

## Show an Error

```python
send_request("ui/showMessage", {
    "message": "Failed to connect to the database.",
    "title": "Connection Error",
    "kind": "error"
})
```

## Ask for Confirmation

```python
response = send_request("ui/showConfirm", {
    "message": "Overwrite existing patient data?",
    "title": "Confirm Overwrite",
    "buttons": "yesNo"
})

if response["result"]["confirmed"]:
    # user clicked Yes
    log("User confirmed overwrite")
    proceed_with_operation()
else:
    # user clicked No
    log("User cancelled overwrite")
    return
```

## Use OK/Cancel Buttons

```python
response = send_request("ui/showConfirm", {
    "message": "Apply changes to the message?",
    "title": "Confirm Changes",
    "buttons": "okCancel"
})

if response["result"]["confirmed"]:
    apply_changes()
```

## Select a File to Open

```python
response = send_request("ui/openFile", {
    "title": "Select Patient File",
    "filters": [
        {"name": "JSON Files", "extensions": ["json"]},
        {"name": "All Files", "extensions": ["*"]}
    ]
})

if "error" in response:
    log(f"Dialog error: {response['error']['message']}")
    return

path = response["result"]["path"]

if path is None:
    # user cancelled
    log("File selection cancelled")
    return

# user selected a file
log(f"Selected file: {path}")

try:
    with open(path, "r", encoding="utf-8") as f:
        data = f.read()
except FileNotFoundError:
    send_request("ui/showMessage", {
        "message": f"File not found:\n{path}",
        "title": "Error",
        "kind": "error"
    })
```

## Filter by File Type

```python
# only show HL7 files
response = send_request("ui/openFile", {
    "title": "Select HL7 Message",
    "filters": [
        {"name": "HL7 Files", "extensions": ["hl7", "txt"]}
    ]
})
```

```python
# multiple filter options
response = send_request("ui/openFile", {
    "title": "Select Import File",
    "filters": [
        {"name": "HL7 Files", "extensions": ["hl7"]},
        {"name": "JSON Files", "extensions": ["json"]},
        {"name": "YAML Files", "extensions": ["yaml", "yml"]},
        {"name": "All Files", "extensions": ["*"]}
    ]
})
```

## Set a Default Directory

```python
import os

response = send_request("ui/openFile", {
    "title": "Select Patient File",
    "defaultPath": os.path.expanduser("~/Documents/patients")
})
```

## Select Multiple Files

```python
response = send_request("ui/openFiles", {
    "title": "Select Messages to Import",
    "filters": [
        {"name": "HL7 Files", "extensions": ["hl7"]}
    ]
})

if "error" in response:
    log(f"Dialog error: {response['error']['message']}")
    return

paths = response["result"]["paths"]

if paths is None:
    # user cancelled
    log("File selection cancelled")
    return

# user selected one or more files
log(f"Selected {len(paths)} files")
for path in paths:
    log(f"  - {path}")
    process_file(path)
```

## Choose a Save Location

```python
response = send_request("ui/saveFile", {
    "title": "Export Message",
    "defaultName": "patient.hl7",
    "filters": [
        {"name": "HL7 Files", "extensions": ["hl7"]},
        {"name": "Text Files", "extensions": ["txt"]}
    ]
})

if "error" in response:
    log(f"Dialog error: {response['error']['message']}")
    return

path = response["result"]["path"]

if path is None:
    # user cancelled
    log("Save cancelled")
    return

# user selected a save location
log(f"Saving to: {path}")

try:
    with open(path, "w", encoding="utf-8") as f:
        f.write(message_content)

    send_request("ui/showMessage", {
        "message": f"Message saved to:\n{path}",
        "title": "Export Complete",
        "kind": "info"
    })
except IOError as e:
    send_request("ui/showMessage", {
        "message": f"Failed to save file:\n{e}",
        "title": "Export Failed",
        "kind": "error"
    })
```

## Select a Directory

```python
response = send_request("ui/selectDirectory", {
    "title": "Select Output Folder",
    "defaultPath": os.path.expanduser("~/Documents")
})

if "error" in response:
    log(f"Dialog error: {response['error']['message']}")
    return

directory = response["result"]["path"]

if directory is None:
    # user cancelled
    log("Directory selection cancelled")
    return

# user selected a directory
log(f"Selected directory: {directory}")

# create output files in the directory
output_file = os.path.join(directory, "output.hl7")
```

## Handle Dialog Errors vs Cancellation

Dialog errors occur when the system fails to show the dialog. User cancellation
is **not an error**—it returns `null` for file paths or `false` for
confirmations.

```python
response = send_request("ui/openFile", {
    "title": "Select File"
})

# check for system error first
if "error" in response:
    log(f"System error showing dialog: {response['error']['message']}")
    send_request("ui/showMessage", {
        "message": "Failed to show file selection dialog.",
        "kind": "error"
    })
    return

# check for user cancellation
if response["result"]["path"] is None:
    log("User cancelled file selection")
    return  # this is normal, not an error

# user selected a file
path = response["result"]["path"]
process_file(path)
```

## Complete Example: Load Patient from File

```python
def load_patient_from_file():
    """Load patient data from a JSON file selected by the user."""

    # show file dialog
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

    # check for cancellation
    path = response["result"]["path"]
    if path is None:
        log("File selection cancelled")
        return

    log(f"Selected file: {path}")

    # read and parse the file
    try:
        with open(path, "r", encoding="utf-8") as f:
            patient = json.load(f)
    except FileNotFoundError:
        send_request("ui/showMessage", {
            "message": f"File not found:\n{path}",
            "title": "Load Failed",
            "kind": "error"
        })
        return
    except json.JSONDecodeError as e:
        send_request("ui/showMessage", {
            "message": f"Invalid JSON format:\n{e}",
            "title": "Load Failed",
            "kind": "error"
        })
        return

    # confirm overwrite if patient data exists
    current_msg = send_request("editor/getMessage", {"format": "hl7"})
    if current_msg["result"]["message"]:
        confirm = send_request("ui/showConfirm", {
            "message": "Overwrite existing patient data?",
            "title": "Confirm Overwrite",
            "buttons": "yesNo"
        })

        if not confirm["result"]["confirmed"]:
            log("User declined overwrite")
            return

    # patch the message with patient data
    patches = [
        {"path": "PID.3.1", "value": patient.get("mrn", "")},
        {"path": "PID.5.1", "value": patient.get("lastName", "")},
        {"path": "PID.5.2", "value": patient.get("firstName", "")},
        {"path": "PID.7", "value": patient.get("dob", "")},
        {"path": "PID.8", "value": patient.get("sex", "")}
    ]

    patch_response = send_request("editor/patchMessage", {"patches": patches})

    if not patch_response["result"]["success"]:
        errors = patch_response["result"].get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        send_request("ui/showMessage", {
            "message": f"Failed to update message:\n{error_msg}",
            "title": "Load Failed",
            "kind": "error"
        })
        return

    # show success message
    name = f"{patient.get('firstName', '')} {patient.get('lastName', '')}".strip()
    send_request("ui/showMessage", {
        "message": f"Patient loaded: {name}",
        "title": "Success",
        "kind": "info"
    })

    log(f"Loaded patient: {name}")
```

## Dialog Types Summary

| Dialog Type          | Use Case                              | User Can Cancel |
|----------------------|---------------------------------------|-----------------|
| `ui/showMessage`     | Show information, warnings, errors    | No              |
| `ui/showConfirm`     | Ask yes/no or ok/cancel question      | Yes             |
| `ui/openFile`        | Select a single file to open          | Yes             |
| `ui/openFiles`       | Select multiple files to open         | Yes             |
| `ui/saveFile`        | Choose where to save a file           | Yes             |
| `ui/selectDirectory` | Choose a directory                    | Yes             |

## Related Documentation

- [Reference: UI Methods](../reference/methods.md#ui-methods)
- [How-To: Handle Errors](handle-errors.md)
- [Tutorial: Building Your First Extension](../tutorials/first-extension.md)
