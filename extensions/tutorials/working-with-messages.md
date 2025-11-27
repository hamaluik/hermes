# Tutorial: Working with Messages

This tutorial builds on [Your First Extension](first-extension.md) to read,
transform, and write HL7 messages in various formats.

## What You'll Build

An extension with three toolbar buttons:
- **Uppercase Names** - Converts all patient and provider names to uppercase
- **Normalise Phone** - Formats all phone numbers consistently
- **Show Summary** - Displays a summary of the message in a dialog

## What You'll Learn

- Reading messages in HL7, JSON, YAML, and TOML formats
- Parsing JSON representations of messages
- Performing transformations on message content
- Setting the entire message content
- The read-modify-write pattern for complex operations

## Prerequisites

Start with the complete code from [Your First Extension](first-extension.md).
You'll add message handling capabilities to that extension.

## Message Formats

Hermes can export messages in multiple formats via `editor/getMessage`:

| Format | Best For                                              |
|--------|-------------------------------------------------------|
| `hl7`  | Working with raw HL7 text, line-by-line processing    |
| `json` | Parsing and navigating structure programmatically     |
| `yaml` | Human-readable export, debugging                      |
| `toml` | Configuration-style export                            |

For transformations, JSON is usually the best choice—it gives you structured
access to fields without manual parsing.

## Step 1: Add Message Reading Helpers

Add these helper functions after your `send_request()` function:

```python
# ============================================================================
# Message Helpers
# ============================================================================

def get_message_hl7():
    """Get the current message as raw HL7 text."""
    response = send_request("editor/getMessage", {"format": "hl7"})

    if "error" in response:
        log(f"Failed to read message: {response['error']['message']}")
        return None

    return response.get("result", {}).get("message")


def get_message_json():
    """Get the current message as a parsed JSON structure."""
    response = send_request("editor/getMessage", {"format": "json"})

    if "error" in response:
        log(f"Failed to read message: {response['error']['message']}")
        return None

    message_str = response.get("result", {}).get("message")
    if message_str:
        try:
            return json.loads(message_str)
        except json.JSONDecodeError as e:
            log(f"Failed to parse JSON: {e}")
            return None

    return None


def set_message_hl7(content):
    """Set the entire message content."""
    response = send_request("editor/setMessage", {"message": content})

    if "error" in response:
        log(f"Failed to set message: {response['error']['message']}")
        return False

    return response.get("result", {}).get("success", False)
```

## Step 2: Understanding the JSON Format

When you request `format: "json"`, Hermes returns the message as a hierarchical
structure:

```json
{
  "MSH": {
    "1": "|",
    "2": "^~\\&",
    "3": "SENDER",
    "9": {"1": "ADT", "2": "A01"}
  },
  "PID": {
    "3": {"1": "12345678", "4": "MRN"},
    "5": {"1": "DOE", "2": "JOHN", "3": "Q"}
  },
  "OBX": [
    {"1": "1", "3": {"1": "CODE1"}, "5": "Value1"},
    {"1": "2", "3": {"1": "CODE2"}, "5": "Value2"}
  ]
}
```

Key points:
- Field indices are **1-based strings** (e.g., `"5"` for PID.5)
- Components are nested objects (e.g., `{"1": "DOE", "2": "JOHN"}`)
- Repeated segments become arrays (e.g., multiple OBX segments)
- Empty fields are omitted

## Step 3: Add Icons

Add icons for the toolbar buttons:

```python
# ============================================================================
# Icons
# ============================================================================

ICON_UPPERCASE = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M3 7h18"/>
    <path d="M10 7v10"/>
    <path d="M14 7v10"/>
    <path d="M3 17h18"/>
</svg>"""

ICON_PHONE = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <rect x="5" y="2" width="14" height="20" rx="2" ry="2"/>
    <line x1="12" y1="18" x2="12.01" y2="18"/>
</svg>"""

ICON_INFO = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="12" cy="12" r="10"/>
    <line x1="12" y1="16" x2="12" y2="12"/>
    <line x1="12" y1="8" x2="12.01" y2="8"/>
</svg>"""
```

## Step 4: Update Initialize Handler

Register the commands and buttons:

```python
def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Message Tools",
            "version": "1.0.0",
            "description": "Transform and analyse HL7 messages",
            "capabilities": {
                "commands": [
                    "messageTools/uppercase",
                    "messageTools/normalisePhone",
                    "messageTools/summary"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "msg-uppercase",
                    "label": "Uppercase Names",
                    "icon": ICON_UPPERCASE,
                    "command": "messageTools/uppercase"
                },
                {
                    "id": "msg-phone",
                    "label": "Normalise Phone Numbers",
                    "icon": ICON_PHONE,
                    "command": "messageTools/normalisePhone"
                },
                {
                    "id": "msg-summary",
                    "label": "Show Message Summary",
                    "icon": ICON_INFO,
                    "command": "messageTools/summary"
                }
            ]
        }
    }
```

## Step 5: Implement the Uppercase Command

This command uses the read-modify-write pattern with raw HL7:

```python
def execute_uppercase():
    """Convert patient and provider names to uppercase."""
    log("Converting names to uppercase...")

    message = get_message_hl7()
    if message is None:
        return

    lines = message.split("\r")
    modified_lines = []

    for line in lines:
        if line.startswith("PID|"):
            # PID.5 is the patient name (field index 5)
            fields = line.split("|")
            if len(fields) > 5:
                fields[5] = fields[5].upper()
            modified_lines.append("|".join(fields))

        elif line.startswith("PV1|"):
            # PV1.7 is attending doctor, PV1.8 is referring doctor
            fields = line.split("|")
            if len(fields) > 7:
                fields[7] = fields[7].upper()
            if len(fields) > 8:
                fields[8] = fields[8].upper()
            modified_lines.append("|".join(fields))

        else:
            modified_lines.append(line)

    new_message = "\r".join(modified_lines)

    if set_message_hl7(new_message):
        log("Names converted to uppercase")
    else:
        log("Failed to update message")
```

## Step 6: Implement the Phone Normalisation Command

This command uses JSON for structured field access:

```python
import re

def normalise_phone(phone):
    """Convert phone to consistent format: (XXX) XXX-XXXX."""
    # extract just the digits
    digits = re.sub(r"\D", "", phone)

    # handle 10-digit North American numbers
    if len(digits) == 10:
        return f"({digits[:3]}) {digits[3:6]}-{digits[6:]}"

    # handle 11-digit numbers with country code
    if len(digits) == 11 and digits[0] == "1":
        return f"({digits[1:4]}) {digits[4:7]}-{digits[7:]}"

    # return original if we can't normalise
    return phone


def execute_normalise_phone():
    """Normalise all phone numbers in the message."""
    log("Normalising phone numbers...")

    message = get_message_json()
    if message is None:
        return

    patches = []

    # PID.13 - Home phone
    if "PID" in message:
        pid = message["PID"]
        if "13" in pid:
            phone_field = pid["13"]
            if isinstance(phone_field, dict) and "1" in phone_field:
                original = phone_field["1"]
                normalised = normalise_phone(original)
                if normalised != original:
                    patches.append({"path": "PID.13.1", "value": normalised})
            elif isinstance(phone_field, str):
                normalised = normalise_phone(phone_field)
                if normalised != phone_field:
                    patches.append({"path": "PID.13", "value": normalised})

        # PID.14 - Business phone
        if "14" in pid:
            phone_field = pid["14"]
            if isinstance(phone_field, dict) and "1" in phone_field:
                original = phone_field["1"]
                normalised = normalise_phone(original)
                if normalised != original:
                    patches.append({"path": "PID.14.1", "value": normalised})

    if not patches:
        log("No phone numbers to normalise")
        return

    response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in response:
        log(f"Failed to normalise phones: {response['error']['message']}")
        return

    log(f"Normalised {len(patches)} phone number(s)")
```

## Step 7: Implement the Summary Command

This command reads the JSON and extracts key information:

```python
def execute_summary():
    """Show a summary of the current message."""
    log("Generating message summary...")

    message = get_message_json()
    if message is None:
        send_request("ui/showMessage", {
            "title": "Error",
            "message": "Could not read message",
            "kind": "error"
        })
        return

    summary_parts = []

    # message type from MSH.9
    if "MSH" in message:
        msh = message["MSH"]
        if "9" in msh:
            msg_type = msh["9"]
            if isinstance(msg_type, dict):
                type_str = f"{msg_type.get('1', '?')}^{msg_type.get('2', '?')}"
            else:
                type_str = str(msg_type)
            summary_parts.append(f"Type: {type_str}")

    # patient info from PID
    if "PID" in message:
        pid = message["PID"]

        # patient name
        if "5" in pid:
            name = pid["5"]
            if isinstance(name, dict):
                name_str = f"{name.get('2', '')} {name.get('1', '')}".strip()
            else:
                name_str = str(name)
            if name_str:
                summary_parts.append(f"Patient: {name_str}")

        # MRN
        if "3" in pid:
            mrn_field = pid["3"]
            if isinstance(mrn_field, dict):
                mrn = mrn_field.get("1", "")
            else:
                mrn = str(mrn_field)
            if mrn:
                summary_parts.append(f"MRN: {mrn}")

        # DOB
        if "7" in pid:
            dob = pid["7"]
            if isinstance(dob, str) and len(dob) >= 8:
                formatted_dob = f"{dob[:4]}-{dob[4:6]}-{dob[6:8]}"
                summary_parts.append(f"DOB: {formatted_dob}")

    # segment count
    segment_count = sum(
        1 if not isinstance(v, list) else len(v)
        for k, v in message.items()
    )
    summary_parts.append(f"Segments: {segment_count}")

    # show the summary dialog
    summary_text = "\n".join(summary_parts) if summary_parts else "No data found"

    send_request("ui/showMessage", {
        "title": "Message Summary",
        "message": summary_text,
        "kind": "info"
    })

    log("Summary displayed")
```

## Step 8: Update Command Routing

```python
def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "messageTools/uppercase":
        execute_uppercase()
    elif command == "messageTools/normalisePhone":
        execute_normalise_phone()
    elif command == "messageTools/summary":
        execute_summary()
    else:
        log(f"Unknown command: {command}")
```

## Testing

### Test Uppercase Names

1. Create a message with lowercase names in PID.5
2. Click "Uppercase Names"
3. Verify the names are now uppercase

### Test Phone Normalisation

1. Add phone numbers in various formats to PID.13:
   - `6135551234`
   - `613-555-1234`
   - `(613) 555-1234`
2. Click "Normalise Phone Numbers"
3. Verify all phones are formatted as `(613) 555-1234`

### Test Summary

1. Open any HL7 message
2. Click "Show Message Summary"
3. Verify the dialog shows message type, patient info, and segment count

## Read-Modify-Write Patterns

Different approaches for different needs:

| Pattern                   | Best For                               |
|---------------------------|----------------------------------------|
| Raw HL7 read/write        | Line-by-line transformations           |
| JSON read + patchMessage  | Targeted field updates                 |
| JSON read + setMessage    | Complex restructuring                  |

**When to use each:**

- **Raw HL7**: When you need to preserve exact formatting or work with
  non-standard messages
- **JSON + patch**: When you know exactly which fields to change
- **JSON + set**: When transformations affect structure (adding/removing segments)

## What You've Learned

- **Reading messages** in multiple formats (HL7, JSON, YAML, TOML)
- **Parsing JSON representations** of HL7 messages
- **Transforming message content** with different approaches
- **Using setMessage** to replace entire message content
- **The read-modify-write pattern** for complex transformations

## Next Steps

Continue to [Building a Wizard with UI](wizard-with-ui.md) to learn how to
create rich web interfaces for complex workflows.

## Complete Code

```python
#!/usr/bin/env python3
"""
Message Tools Extension

Provides tools for transforming and analysing HL7 messages.
"""

import sys
import json
import re

# ============================================================================
# Icons
# ============================================================================

ICON_UPPERCASE = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M3 7h18"/>
    <path d="M10 7v10"/>
    <path d="M14 7v10"/>
    <path d="M3 17h18"/>
</svg>"""

ICON_PHONE = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <rect x="5" y="2" width="14" height="20" rx="2" ry="2"/>
    <line x1="12" y1="18" x2="12.01" y2="18"/>
</svg>"""

ICON_INFO = """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="12" cy="12" r="10"/>
    <line x1="12" y1="16" x2="12" y2="12"/>
    <line x1="12" y1="8" x2="12.01" y2="8"/>
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
    sys.stderr.write(f"[message-tools] {message}\n")
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
# Message Helpers
# ============================================================================

def get_message_hl7():
    """Get the current message as raw HL7 text."""
    response = send_request("editor/getMessage", {"format": "hl7"})

    if "error" in response:
        log(f"Failed to read message: {response['error']['message']}")
        return None

    return response.get("result", {}).get("message")


def get_message_json():
    """Get the current message as a parsed JSON structure."""
    response = send_request("editor/getMessage", {"format": "json"})

    if "error" in response:
        log(f"Failed to read message: {response['error']['message']}")
        return None

    message_str = response.get("result", {}).get("message")
    if message_str:
        try:
            return json.loads(message_str)
        except json.JSONDecodeError as e:
            log(f"Failed to parse JSON: {e}")
            return None

    return None


def set_message_hl7(content):
    """Set the entire message content."""
    response = send_request("editor/setMessage", {"message": content})

    if "error" in response:
        log(f"Failed to set message: {response['error']['message']}")
        return False

    return response.get("result", {}).get("success", False)


# ============================================================================
# Transformation Helpers
# ============================================================================

def normalise_phone(phone):
    """Convert phone to consistent format: (XXX) XXX-XXXX."""
    digits = re.sub(r"\D", "", phone)

    if len(digits) == 10:
        return f"({digits[:3]}) {digits[3:6]}-{digits[6:]}"

    if len(digits) == 11 and digits[0] == "1":
        return f"({digits[1:4]}) {digits[4:7]}-{digits[7:]}"

    return phone


# ============================================================================
# Command Implementations
# ============================================================================

def execute_uppercase():
    """Convert patient and provider names to uppercase."""
    log("Converting names to uppercase...")

    message = get_message_hl7()
    if message is None:
        return

    lines = message.split("\r")
    modified_lines = []

    for line in lines:
        if line.startswith("PID|"):
            fields = line.split("|")
            if len(fields) > 5:
                fields[5] = fields[5].upper()
            modified_lines.append("|".join(fields))

        elif line.startswith("PV1|"):
            fields = line.split("|")
            if len(fields) > 7:
                fields[7] = fields[7].upper()
            if len(fields) > 8:
                fields[8] = fields[8].upper()
            modified_lines.append("|".join(fields))

        else:
            modified_lines.append(line)

    new_message = "\r".join(modified_lines)

    if set_message_hl7(new_message):
        log("Names converted to uppercase")
    else:
        log("Failed to update message")


def execute_normalise_phone():
    """Normalise all phone numbers in the message."""
    log("Normalising phone numbers...")

    message = get_message_json()
    if message is None:
        return

    patches = []

    if "PID" in message:
        pid = message["PID"]

        # PID.13 - Home phone
        if "13" in pid:
            phone_field = pid["13"]
            if isinstance(phone_field, dict) and "1" in phone_field:
                original = phone_field["1"]
                normalised = normalise_phone(original)
                if normalised != original:
                    patches.append({"path": "PID.13.1", "value": normalised})
            elif isinstance(phone_field, str):
                normalised = normalise_phone(phone_field)
                if normalised != phone_field:
                    patches.append({"path": "PID.13", "value": normalised})

        # PID.14 - Business phone
        if "14" in pid:
            phone_field = pid["14"]
            if isinstance(phone_field, dict) and "1" in phone_field:
                original = phone_field["1"]
                normalised = normalise_phone(original)
                if normalised != original:
                    patches.append({"path": "PID.14.1", "value": normalised})

    if not patches:
        log("No phone numbers to normalise")
        return

    response = send_request("editor/patchMessage", {"patches": patches})

    if "error" in response:
        log(f"Failed to normalise phones: {response['error']['message']}")
        return

    log(f"Normalised {len(patches)} phone number(s)")


def execute_summary():
    """Show a summary of the current message."""
    log("Generating message summary...")

    message = get_message_json()
    if message is None:
        send_request("ui/showMessage", {
            "title": "Error",
            "message": "Could not read message",
            "kind": "error"
        })
        return

    summary_parts = []

    # message type from MSH.9
    if "MSH" in message:
        msh = message["MSH"]
        if "9" in msh:
            msg_type = msh["9"]
            if isinstance(msg_type, dict):
                type_str = f"{msg_type.get('1', '?')}^{msg_type.get('2', '?')}"
            else:
                type_str = str(msg_type)
            summary_parts.append(f"Type: {type_str}")

    # patient info from PID
    if "PID" in message:
        pid = message["PID"]

        if "5" in pid:
            name = pid["5"]
            if isinstance(name, dict):
                name_str = f"{name.get('2', '')} {name.get('1', '')}".strip()
            else:
                name_str = str(name)
            if name_str:
                summary_parts.append(f"Patient: {name_str}")

        if "3" in pid:
            mrn_field = pid["3"]
            if isinstance(mrn_field, dict):
                mrn = mrn_field.get("1", "")
            else:
                mrn = str(mrn_field)
            if mrn:
                summary_parts.append(f"MRN: {mrn}")

        if "7" in pid:
            dob = pid["7"]
            if isinstance(dob, str) and len(dob) >= 8:
                formatted_dob = f"{dob[:4]}-{dob[4:6]}-{dob[6:8]}"
                summary_parts.append(f"DOB: {formatted_dob}")

    segment_count = sum(
        1 if not isinstance(v, list) else len(v)
        for k, v in message.items()
    )
    summary_parts.append(f"Segments: {segment_count}")

    summary_text = "\n".join(summary_parts) if summary_parts else "No data found"

    send_request("ui/showMessage", {
        "title": "Message Summary",
        "message": summary_text,
        "kind": "info"
    })

    log("Summary displayed")


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
            "name": "Message Tools",
            "version": "1.0.0",
            "description": "Transform and analyse HL7 messages",
            "capabilities": {
                "commands": [
                    "messageTools/uppercase",
                    "messageTools/normalisePhone",
                    "messageTools/summary"
                ]
            },
            "toolbarButtons": [
                {
                    "id": "msg-uppercase",
                    "label": "Uppercase Names",
                    "icon": ICON_UPPERCASE,
                    "command": "messageTools/uppercase"
                },
                {
                    "id": "msg-phone",
                    "label": "Normalise Phone Numbers",
                    "icon": ICON_PHONE,
                    "command": "messageTools/normalisePhone"
                },
                {
                    "id": "msg-summary",
                    "label": "Show Message Summary",
                    "icon": ICON_INFO,
                    "command": "messageTools/summary"
                }
            ]
        }
    }


def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "messageTools/uppercase":
        execute_uppercase()
    elif command == "messageTools/normalisePhone":
        execute_normalise_phone()
    elif command == "messageTools/summary":
        execute_summary()
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
