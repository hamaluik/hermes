# Tutorial: Providing Schema Overrides

In this advanced tutorial, you'll learn how to customise Hermes' field
definitions by providing schema overrides. You'll build an extension that adds
organisation- specific validation rules, allowed values, and help text for HL7
fields.

## What You'll Build

An extension that customises field handling for a hospital environment:
- Enforces 8-digit MRN format for patient IDs
- Adds a dropdown for administrative sex with custom values
- Provides organisation-specific notes and placeholders
- Sets template values for new messages

This is a "passive" extension—it modifies Hermes' behaviour without adding
toolbar buttons. The customisations take effect automatically.

## What You'll Learn

- Providing schema overrides in the initialize response
- Adding custom field notes and placeholders
- Defining allowed values (dropdown lists)
- Setting validation rules (required, length, patterns)
- Configuring template defaults for new messages
- Understanding schema merging behaviour

## Prerequisites

Start with the complete code from [Your First Extension](first-extension.md).
This tutorial is simpler than that one because schema extensions don't need
`send_request()` or command handlers—they're purely passive.

## How Schema Overrides Work

When extensions provide schema overrides, Hermes merges them with the built-in
schema. Your customisations overlay the defaults:

```
┌─────────────────────────────────────┐
│ Built-in Schema                     │
│                                     │
│  PID.3.1: "Patient ID"              │
│  PID.8:   (no values defined)       │
│                                     │
└─────────────────────────────────────┘
              │
              │ merge
              ▼
┌─────────────────────────────────────┐
│ Extension Schema Override           │
│                                     │
│  PID.3.1: note="8-digit MRN..."     │
│           required=True             │
│           pattern="^[0-9]{8}$"      │
│                                     │
│  PID.8:   values={M, F, O, U}       │
│                                     │
└─────────────────────────────────────┘
              │
              │ result
              ▼
┌─────────────────────────────────────┐
│ Effective Schema                    │
│                                     │
│  PID.3.1: "Patient ID"              │
│           note="8-digit MRN..."     │  ← added
│           required=True             │  ← added
│           pattern="^[0-9]{8}$"      │  ← added
│                                     │
│  PID.8:   values={M, F, O, U}       │  ← added
│                                     │
└─────────────────────────────────────┘
```

Properties you specify replace or add to the built-in values. Properties you
don't specify are inherited unchanged.

## Step 1: Create the Extension File

Create `hospital_schema.py`:

```bash
touch hospital_schema.py
chmod +x hospital_schema.py
```

Add the imports:

```python
#!/usr/bin/env python3
"""
Hospital Schema Extension

Provides organisation-specific schema overrides for HL7 fields.
"""

import sys
import json
```

## Step 2: Add Message I/O Functions

Use the `read_message()`, `write_message()`, and `log()` functions from the
first tutorial. This extension doesn't need `send_request()` since it's purely
passive—it just responds to initialize and shutdown.

```python
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
    sys.stderr.write(f"[hospital-schema] {message}\n")
    sys.stderr.flush()
```

## Step 3: Define the Schema Structure

Schema overrides are organised by segment, then by field:

```python
# ============================================================================
# Schema Definition
# ============================================================================

HOSPITAL_SCHEMA = {
    "segments": {
        "PID": {
            "fields": [
                # ... field definitions go here
            ]
        },
        "PV1": {
            "fields": [
                # ... field definitions go here
            ]
        }
    }
}
```

Each field definition specifies which field (and optionally which component)
it applies to, plus the properties to override.

## Step 4: Add Patient ID Validation

Let's enforce that patient IDs (MRN) must be exactly 8 digits:

```python
HOSPITAL_SCHEMA = {
    "segments": {
        "PID": {
            "fields": [
                # PID.3.1 - Patient ID (MRN)
                {
                    "field": 3,
                    "component": 1,
                    "note": "8-digit MRN from the hospital information system. "
                            "Contact HIM if MRN is unknown.",
                    "required": True,
                    "minlength": 8,
                    "maxlength": 8,
                    "pattern": "^[0-9]{8}$",
                    "placeholder": "00000000"
                },

                # PID.3.4 - Assigning Authority
                {
                    "field": 3,
                    "component": 4,
                    "note": "Always use 'MRN' for hospital-assigned IDs.",
                    "template": "MRN"
                },
            ]
        }
    }
}
```

**Field properties explained:**

| Property      | Effect                                             |
| ------------- | -------------------------------------------------- |
| `note`        | Help text shown when hovering over the field       |
| `required`    | Field must have a value; validation error if empty |
| `minlength`   | Minimum character count                            |
| `maxlength`   | Maximum character count                            |
| `pattern`     | Regular expression the value must match            |
| `placeholder` | Grey text shown in empty input fields              |
| `template`    | Default value when creating new messages           |

## Step 5: Add Allowed Values (Dropdowns)

When you define `values`, the field becomes a dropdown instead of free text:

```python
                # PID.8 - Administrative Sex
                {
                    "field": 8,
                    "note": "Patient's administrative sex as recorded at registration.",
                    "required": True,
                    "values": {
                        "M": "Male",
                        "F": "Female",
                        "O": "Other",
                        "U": "Unknown",
                        "A": "Ambiguous",
                        "N": "Not applicable"
                    }
                },
```

The keys (`M`, `F`, etc.) are stored in the HL7 message. The values (`Male`,
`Female`, etc.) are displayed in the dropdown for user-friendliness.

## Step 6: Add Patient Name Fields

```python
                # PID.5.1 - Family Name
                {
                    "field": 5,
                    "component": 1,
                    "name": "Family Name",
                    "note": "Patient's legal surname. Use uppercase.",
                    "required": True,
                    "maxlength": 50,
                    "placeholder": "SURNAME"
                },

                # PID.5.2 - Given Name
                {
                    "field": 5,
                    "component": 2,
                    "name": "Given Name",
                    "note": "Patient's first name. Use uppercase.",
                    "required": True,
                    "maxlength": 50,
                    "placeholder": "FIRSTNAME"
                },

                # PID.5.3 - Second/Middle Name
                {
                    "field": 5,
                    "component": 3,
                    "name": "Middle Name",
                    "note": "Optional middle name or initial.",
                    "maxlength": 50,
                    "placeholder": "MIDDLE"
                },
```

## Step 7: Add Date Fields

Date fields get special handling when you specify the `datatype`:

```python
                # PID.7 - Date of Birth
                {
                    "field": 7,
                    "name": "Date of Birth",
                    "note": "Patient's date of birth in YYYYMMDD format.",
                    "required": True,
                    "datatype": "date",
                    "placeholder": "YYYYMMDD"
                },

                # PID.29 - Date of Death
                {
                    "field": 29,
                    "name": "Date/Time of Death",
                    "note": "Leave empty if patient is alive.",
                    "datatype": "datetime"
                },
```

The `datatype` property enables date-specific validation and potentially a
date picker in the UI:
- `"date"` - Date only (YYYYMMDD)
- `"datetime"` - Date and time (YYYYMMDDHHmmss)

## Step 8: Add Visit Information (PV1)

Extend the schema to cover visit information:

```python
        "PV1": {
            "fields": [
                # PV1.2 - Patient Class
                {
                    "field": 2,
                    "name": "Patient Class",
                    "note": "Type of patient encounter.",
                    "required": True,
                    "values": {
                        "I": "Inpatient",
                        "O": "Outpatient",
                        "E": "Emergency",
                        "P": "Preadmit",
                        "R": "Recurring patient",
                        "B": "Obstetrics"
                    }
                },

                # PV1.3.1 - Assigned Patient Location (Point of Care)
                {
                    "field": 3,
                    "component": 1,
                    "name": "Nursing Station",
                    "note": "Ward or unit code (e.g., 4N, ICU, ER).",
                    "placeholder": "UNIT"
                },

                # PV1.3.2 - Room
                {
                    "field": 3,
                    "component": 2,
                    "name": "Room",
                    "note": "Room number within the unit.",
                    "placeholder": "ROOM"
                },

                # PV1.3.3 - Bed
                {
                    "field": 3,
                    "component": 3,
                    "name": "Bed",
                    "note": "Bed designation (A, B, 1, 2, etc.).",
                    "placeholder": "BED"
                },

                # PV1.44 - Admit Date/Time
                {
                    "field": 44,
                    "name": "Admit Date/Time",
                    "note": "When the patient was admitted.",
                    "datatype": "datetime"
                },

                # PV1.45 - Discharge Date/Time
                {
                    "field": 45,
                    "name": "Discharge Date/Time",
                    "note": "When the patient was discharged. Empty if still admitted.",
                    "datatype": "datetime"
                },
            ]
        }
```

## Step 9: Add Observation Segments (OBX)

For lab results and other observations:

```python
        "OBX": {
            "fields": [
                # OBX.2 - Value Type
                {
                    "field": 2,
                    "name": "Value Type",
                    "note": "Data type of the observation value.",
                    "required": True,
                    "values": {
                        "NM": "Numeric",
                        "ST": "String",
                        "TX": "Text",
                        "CE": "Coded Element",
                        "DT": "Date",
                        "TM": "Time",
                        "TS": "Timestamp",
                        "FT": "Formatted Text",
                        "ED": "Encapsulated Data"
                    }
                },

                # OBX.3.1 - Observation Identifier
                {
                    "field": 3,
                    "component": 1,
                    "name": "Observation Code",
                    "note": "LOINC code preferred. See lab catalogue for local codes.",
                    "placeholder": "12345-6"
                },

                # OBX.3.2 - Observation Name
                {
                    "field": 3,
                    "component": 2,
                    "name": "Observation Name",
                    "note": "Human-readable name for the observation.",
                    "placeholder": "Test Name"
                },

                # OBX.11 - Observation Result Status
                {
                    "field": 11,
                    "name": "Result Status",
                    "note": "Current status of this observation.",
                    "required": True,
                    "values": {
                        "F": "Final",
                        "P": "Preliminary",
                        "C": "Corrected",
                        "X": "Cancelled",
                        "I": "In Progress",
                        "R": "Not Reviewed",
                        "U": "Status Changed to Final"
                    }
                },
            ]
        }
```

## Step 10: Complete the Schema

Here's the complete schema definition:

```python
# ============================================================================
# Schema Definition
# ============================================================================

HOSPITAL_SCHEMA = {
    "segments": {
        "PID": {
            "fields": [
                # patient ID
                {
                    "field": 3,
                    "component": 1,
                    "note": "8-digit MRN from the hospital information system. "
                            "Contact HIM if MRN is unknown.",
                    "required": True,
                    "minlength": 8,
                    "maxlength": 8,
                    "pattern": "^[0-9]{8}$",
                    "placeholder": "00000000"
                },
                {
                    "field": 3,
                    "component": 4,
                    "note": "Always use 'MRN' for hospital-assigned IDs.",
                    "template": "MRN"
                },

                # patient name
                {
                    "field": 5,
                    "component": 1,
                    "name": "Family Name",
                    "note": "Patient's legal surname. Use uppercase.",
                    "required": True,
                    "maxlength": 50,
                    "placeholder": "SURNAME"
                },
                {
                    "field": 5,
                    "component": 2,
                    "name": "Given Name",
                    "note": "Patient's first name. Use uppercase.",
                    "required": True,
                    "maxlength": 50,
                    "placeholder": "FIRSTNAME"
                },
                {
                    "field": 5,
                    "component": 3,
                    "name": "Middle Name",
                    "note": "Optional middle name or initial.",
                    "maxlength": 50,
                    "placeholder": "MIDDLE"
                },

                # date of birth
                {
                    "field": 7,
                    "name": "Date of Birth",
                    "note": "Patient's date of birth in YYYYMMDD format.",
                    "required": True,
                    "datatype": "date",
                    "placeholder": "YYYYMMDD"
                },

                # sex
                {
                    "field": 8,
                    "note": "Patient's administrative sex as recorded at registration.",
                    "required": True,
                    "values": {
                        "M": "Male",
                        "F": "Female",
                        "O": "Other",
                        "U": "Unknown",
                        "A": "Ambiguous",
                        "N": "Not applicable"
                    }
                },

                # death date
                {
                    "field": 29,
                    "name": "Date/Time of Death",
                    "note": "Leave empty if patient is alive.",
                    "datatype": "datetime"
                },
            ]
        },

        "PV1": {
            "fields": [
                {
                    "field": 2,
                    "name": "Patient Class",
                    "note": "Type of patient encounter.",
                    "required": True,
                    "values": {
                        "I": "Inpatient",
                        "O": "Outpatient",
                        "E": "Emergency",
                        "P": "Preadmit",
                        "R": "Recurring patient",
                        "B": "Obstetrics"
                    }
                },
                {
                    "field": 3,
                    "component": 1,
                    "name": "Nursing Station",
                    "note": "Ward or unit code (e.g., 4N, ICU, ER).",
                    "placeholder": "UNIT"
                },
                {
                    "field": 3,
                    "component": 2,
                    "name": "Room",
                    "note": "Room number within the unit.",
                    "placeholder": "ROOM"
                },
                {
                    "field": 3,
                    "component": 3,
                    "name": "Bed",
                    "note": "Bed designation (A, B, 1, 2, etc.).",
                    "placeholder": "BED"
                },
                {
                    "field": 44,
                    "name": "Admit Date/Time",
                    "note": "When the patient was admitted.",
                    "datatype": "datetime"
                },
                {
                    "field": 45,
                    "name": "Discharge Date/Time",
                    "note": "When the patient was discharged. Empty if still admitted.",
                    "datatype": "datetime"
                },
            ]
        },

        "OBX": {
            "fields": [
                {
                    "field": 2,
                    "name": "Value Type",
                    "note": "Data type of the observation value.",
                    "required": True,
                    "values": {
                        "NM": "Numeric",
                        "ST": "String",
                        "TX": "Text",
                        "CE": "Coded Element",
                        "DT": "Date",
                        "TM": "Time",
                        "TS": "Timestamp",
                        "FT": "Formatted Text",
                        "ED": "Encapsulated Data"
                    }
                },
                {
                    "field": 3,
                    "component": 1,
                    "name": "Observation Code",
                    "note": "LOINC code preferred. See lab catalogue for local codes.",
                    "placeholder": "12345-6"
                },
                {
                    "field": 3,
                    "component": 2,
                    "name": "Observation Name",
                    "note": "Human-readable name for the observation.",
                    "placeholder": "Test Name"
                },
                {
                    "field": 11,
                    "name": "Result Status",
                    "note": "Current status of this observation.",
                    "required": True,
                    "values": {
                        "F": "Final",
                        "P": "Preliminary",
                        "C": "Corrected",
                        "X": "Cancelled",
                        "I": "In Progress",
                        "R": "Not Reviewed",
                        "U": "Status Changed to Final"
                    }
                },
            ]
        }
    }
}
```

## Step 11: Handle Initialize

The schema is provided in the initialize response:

```python
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
            "name": "Hospital Schema",
            "version": "1.0.0",
            "description": "Organisation-specific field definitions and validation",
            "capabilities": {
                "schemaProvider": True
            },
            # no toolbar buttons - this is a passive extension
            "schema": HOSPITAL_SCHEMA
        }
    }
```

**Key points:**
- Set `schemaProvider: True` in capabilities to indicate you provide schema
- Include the schema object directly in the result
- No toolbar buttons are required—schema extensions can be purely passive

## Step 12: Add Remaining Handlers

```python
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

    # this extension handles no commands, but still needs to handle
    # notifications gracefully
    if request_id is None:
        if method == "command/execute":
            log(f"Ignoring command: {params.get('command')}")
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
```

## Step 13: Add the Main Loop

```python
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

## Checkpoint: Test Your Extension

### Install the Extension

1. Add the extension in Settings > Extensions
2. Enter the path: `/full/path/to/hospital_schema.py`
3. Click "Reload Extensions"
4. Verify "Running" status

### Test Field Notes

1. Create or open an HL7 message with a PID segment
2. Place your cursor on PID.3.1 (patient ID field)
3. You should see the custom note about 8-digit MRN

### Test Allowed Values

1. Look at PID.8 (administrative sex)
2. The field should show a dropdown with Male, Female, Other, etc.
3. Try typing an invalid value—validation should flag it

### Test Validation

1. Enter a 5-digit value in PID.3.1
2. Run validation (Cmd+Shift+V)
3. You should see an error about the pattern not matching

### Test Templates

1. Create a new message from template (File > New from Template)
2. Check that PID.3.4 is pre-filled with "MRN"

## Schema Properties Reference

| Property      | Type    | Description                            |
| ------------- | ------- | -------------------------------------- |
| `field`       | number  | 1-based field number within segment    |
| `component`   | number  | 1-based component number (optional)    |
| `name`        | string  | Display name override                  |
| `note`        | string  | Help text shown on hover               |
| `required`    | boolean | Whether field must have a value        |
| `minlength`   | number  | Minimum character count                |
| `maxlength`   | number  | Maximum character count                |
| `pattern`     | string  | Regular expression for validation      |
| `placeholder` | string  | Grey text shown in empty fields        |
| `template`    | string  | Default value for new messages         |
| `datatype`    | string  | "date" or "datetime" for date handling |
| `values`      | object  | Code→description mapping for dropdowns |

## Merging Behaviour

When multiple extensions provide schema overrides:

1. Built-in schema provides the base
2. Extensions are merged in the order they're loaded
3. Later extensions override earlier ones
4. Properties you specify replace built-in values
5. Properties you omit are inherited unchanged
6. Set a property to `null` to explicitly remove an inherited value

### Removing an Inherited Value

If the built-in schema defines allowed values but you want free text:

```python
{
    "field": 8,
    "values": None  # removes the dropdown, allows free text
}
```

## Complete Code

Here's the full extension:

```python
#!/usr/bin/env python3
"""
Hospital Schema Extension

Provides organisation-specific schema overrides for HL7 fields.
"""

import sys
import json

# ============================================================================
# Schema Definition
# ============================================================================

HOSPITAL_SCHEMA = {
    "segments": {
        "PID": {
            "fields": [
                {
                    "field": 3,
                    "component": 1,
                    "note": "8-digit MRN from the hospital information system. "
                            "Contact HIM if MRN is unknown.",
                    "required": True,
                    "minlength": 8,
                    "maxlength": 8,
                    "pattern": "^[0-9]{8}$",
                    "placeholder": "00000000"
                },
                {
                    "field": 3,
                    "component": 4,
                    "note": "Always use 'MRN' for hospital-assigned IDs.",
                    "template": "MRN"
                },
                {
                    "field": 5,
                    "component": 1,
                    "name": "Family Name",
                    "note": "Patient's legal surname. Use uppercase.",
                    "required": True,
                    "maxlength": 50,
                    "placeholder": "SURNAME"
                },
                {
                    "field": 5,
                    "component": 2,
                    "name": "Given Name",
                    "note": "Patient's first name. Use uppercase.",
                    "required": True,
                    "maxlength": 50,
                    "placeholder": "FIRSTNAME"
                },
                {
                    "field": 5,
                    "component": 3,
                    "name": "Middle Name",
                    "note": "Optional middle name or initial.",
                    "maxlength": 50,
                    "placeholder": "MIDDLE"
                },
                {
                    "field": 7,
                    "name": "Date of Birth",
                    "note": "Patient's date of birth in YYYYMMDD format.",
                    "required": True,
                    "datatype": "date",
                    "placeholder": "YYYYMMDD"
                },
                {
                    "field": 8,
                    "note": "Patient's administrative sex as recorded at registration.",
                    "required": True,
                    "values": {
                        "M": "Male",
                        "F": "Female",
                        "O": "Other",
                        "U": "Unknown",
                        "A": "Ambiguous",
                        "N": "Not applicable"
                    }
                },
                {
                    "field": 29,
                    "name": "Date/Time of Death",
                    "note": "Leave empty if patient is alive.",
                    "datatype": "datetime"
                },
            ]
        },
        "PV1": {
            "fields": [
                {
                    "field": 2,
                    "name": "Patient Class",
                    "note": "Type of patient encounter.",
                    "required": True,
                    "values": {
                        "I": "Inpatient",
                        "O": "Outpatient",
                        "E": "Emergency",
                        "P": "Preadmit",
                        "R": "Recurring patient",
                        "B": "Obstetrics"
                    }
                },
                {
                    "field": 3,
                    "component": 1,
                    "name": "Nursing Station",
                    "note": "Ward or unit code (e.g., 4N, ICU, ER).",
                    "placeholder": "UNIT"
                },
                {
                    "field": 3,
                    "component": 2,
                    "name": "Room",
                    "note": "Room number within the unit.",
                    "placeholder": "ROOM"
                },
                {
                    "field": 3,
                    "component": 3,
                    "name": "Bed",
                    "note": "Bed designation (A, B, 1, 2, etc.).",
                    "placeholder": "BED"
                },
                {
                    "field": 44,
                    "name": "Admit Date/Time",
                    "note": "When the patient was admitted.",
                    "datatype": "datetime"
                },
                {
                    "field": 45,
                    "name": "Discharge Date/Time",
                    "note": "When the patient was discharged. Empty if still admitted.",
                    "datatype": "datetime"
                },
            ]
        },
        "OBX": {
            "fields": [
                {
                    "field": 2,
                    "name": "Value Type",
                    "note": "Data type of the observation value.",
                    "required": True,
                    "values": {
                        "NM": "Numeric",
                        "ST": "String",
                        "TX": "Text",
                        "CE": "Coded Element",
                        "DT": "Date",
                        "TM": "Time",
                        "TS": "Timestamp",
                        "FT": "Formatted Text",
                        "ED": "Encapsulated Data"
                    }
                },
                {
                    "field": 3,
                    "component": 1,
                    "name": "Observation Code",
                    "note": "LOINC code preferred. See lab catalogue for local codes.",
                    "placeholder": "12345-6"
                },
                {
                    "field": 3,
                    "component": 2,
                    "name": "Observation Name",
                    "note": "Human-readable name for the observation.",
                    "placeholder": "Test Name"
                },
                {
                    "field": 11,
                    "name": "Result Status",
                    "note": "Current status of this observation.",
                    "required": True,
                    "values": {
                        "F": "Final",
                        "P": "Preliminary",
                        "C": "Corrected",
                        "X": "Cancelled",
                        "I": "In Progress",
                        "R": "Not Reviewed",
                        "U": "Status Changed to Final"
                    }
                },
            ]
        }
    }
}

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
    sys.stderr.write(f"[hospital-schema] {message}\n")
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
            "name": "Hospital Schema",
            "version": "1.0.0",
            "description": "Organisation-specific field definitions and validation",
            "capabilities": {
                "schemaProvider": True
            },
            "schema": HOSPITAL_SCHEMA
        }
    }


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
            log(f"Ignoring command: {params.get('command')}")
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

## What You've Learned

You now know how to:

- **Provide schema overrides** in the initialize response
- **Add field notes** for organisation-specific guidance
- **Define allowed values** to create dropdown fields
- **Set validation rules** with required, length, and pattern constraints
- **Configure templates** for new messages
- **Handle date fields** with the datatype property
- **Understand merging** of multiple schema sources

## Common Patterns

| Use Case                      | Properties to Use                   |
| ----------------------------- | ----------------------------------- |
| Add help text                 | `note`                              |
| Restrict to a list of values  | `values`                            |
| Set defaults for new messages | `template`                          |
| Make a field mandatory        | `required`                          |
| Validate format               | `pattern`, `minlength`, `maxlength` |
| Enable date handling          | `datatype`                          |
| Show hints in empty fields    | `placeholder`                       |

## Next Steps

Congratulations! You've completed all the Hermes extension tutorials. You now
have the skills to build sophisticated extensions.

Explore these resources next:

- [How-To Guides](../how-to/) - Task-focused recipes
- [Reference](../reference/) - Complete API documentation
- [Explanation](../explanation/) - Architecture and design rationale
