# Read and Modify Messages

This guide shows how to read and modify the HL7 message currently open in the
Hermes editor.

## Read the Current Message in HL7 Format

```python
response = send_request("editor/getMessage", {
    "format": "hl7"
})

if "error" in response:
    # handle error
    log(f"Failed to get message: {response['error']['message']}")
    return

hl7_message = response["result"]["message"]
# hl7_message is a string like:
# "MSH|^~\\&|APP|FAC||...\\rPID|1||12345||DOE^JOHN"

# split into segments
segments = hl7_message.split("\r")
for segment in segments:
    log(f"Segment: {segment}")
```

The HL7 format uses `\r` (carriage return) as the segment separator. In the
JSON response, this appears as the escape sequence `\r`.

## Read the Message as JSON

```python
import json

response = send_request("editor/getMessage", {
    "format": "json"
})

if "error" in response:
    log(f"Failed to get message: {response['error']['message']}")
    return

# the message field is a JSON string, parse it
message_json = json.loads(response["result"]["message"])

# access fields using the hierarchical structure
patient_id = message_json.get("PID", {}).get("3", {}).get("1", "")
last_name = message_json.get("PID", {}).get("5", {}).get("1", "")

log(f"Patient: {patient_id} - {last_name}")
```

JSON format uses 1-based string indices. For example, `PID.5.1` becomes
`message["PID"]["5"]["1"]`.

## Read as YAML or TOML

```python
import yaml  # requires PyYAML

response = send_request("editor/getMessage", {
    "format": "yaml"
})

message_yaml = yaml.safe_load(response["result"]["message"])
```

```python
import tomllib  # Python 3.11+, or use `tomli` for earlier versions

response = send_request("editor/getMessage", {
    "format": "toml"
})

message_toml = tomllib.loads(response["result"]["message"])
```

## Patch Specific Fields

Use `patchMessage` for targeted field updates:

```python
response = send_request("editor/patchMessage", {
    "patches": [
        {"path": "PID.5.1", "value": "DOE"},
        {"path": "PID.5.2", "value": "JOHN"},
        {"path": "PID.7", "value": "19800101"},
        {"path": "PID.8", "value": "M"}
    ]
})

if "error" in response:
    log(f"Failed to patch: {response['error']['message']}")
    return

result = response["result"]
if not result["success"]:
    # some patches failed
    for error in result.get("errors", []):
        log(f"Patch {error['index']} failed: {error['message']}")
else:
    log(f"Applied {result['patchesApplied']} patches")
```

### Path Syntax Examples

| Path        | Description                             |
| ----------- | --------------------------------------- |
| `PID.5`     | Entire patient name field               |
| `PID.5.1`   | Family name component                   |
| `PID.5.1.1` | Family name subcomponent                |
| `OBX[2].5`  | Observation value in second OBX segment |
| `PID.13[2]` | Second repetition of phone number       |

## Clear Field Values

Set fields to empty strings:

```python
response = send_request("editor/patchMessage", {
    "patches": [
        {"path": "PID.6", "value": ""},    # clear mother's maiden name
        {"path": "PID.13", "value": ""}    # clear all phone numbers
    ]
})
```

## Replace the Entire Message

Use `setMessage` to replace all content:

```python
# set from HL7 format
response = send_request("editor/setMessage", {
    "message": "MSH|^~\\&|APP|FAC|||20231215120000||ADT^A01|123|P|2.5.1\rPID|1||12345||DOE^JOHN",
    "format": "hl7"
})

if not response["result"]["success"]:
    log(f"Failed to set message: {response['result']['error']}")
```

```python
# set from JSON format
import json

message = {
    "MSH": {"1": "|", "2": "^~\\&", "3": "APP"},
    "PID": {"5": {"1": "DOE", "2": "JOHN"}}
}

response = send_request("editor/setMessage", {
    "message": json.dumps(message),
    "format": "json"
})
```

## Read-Modify-Write Pattern

The safest pattern for complex transformations:

```python
import json

# 1. read current message
response = send_request("editor/getMessage", {"format": "json"})
if "error" in response:
    return

message = json.loads(response["result"]["message"])

# 2. modify the data structure
if "PID" in message:
    # convert last name to uppercase
    if "5" in message["PID"] and "1" in message["PID"]["5"]:
        message["PID"]["5"]["1"] = message["PID"]["5"]["1"].upper()

# 3. write it back
response = send_request("editor/setMessage", {
    "message": json.dumps(message),
    "format": "json"
})

if not response["result"]["success"]:
    log(f"Failed to update: {response['result']['error']}")
```

## Check if a Message is Open

```python
response = send_request("editor/getMessage", {"format": "hl7"})

if "error" in response:
    log("Error getting message")
    return

if not response["result"]["message"]:
    log("No message is currently open")
    # optionally show a dialog
    send_request("ui/showMessage", {
        "message": "Please open a message first.",
        "kind": "warning"
    })
    return

# proceed with message operations
```

## Check if a File is Saved

```python
response = send_request("editor/getMessage", {"format": "hl7"})

if response["result"]["hasFile"]:
    file_path = response["result"]["filePath"]
    log(f"Message is saved at: {file_path}")
else:
    log("Message has not been saved to a file")
```

## Create New Segments

```python
# create a new NK1 segment
response = send_request("editor/patchMessage", {
    "patches": [
        {"path": "NK1", "create": True},
        {"path": "NK1.2.1", "value": "DOE"},
        {"path": "NK1.2.2", "value": "JANE"},
        {"path": "NK1.3", "value": "SPOUSE"}
    ]
})
```

If the segment type already exists, the new segment is appended after the last
occurrence. If it doesn't exist, it's added at the end of the message.

## Delete Segments

```python
# remove all NK1 segments
response = send_request("editor/patchMessage", {
    "patches": [
        {"path": "NK1", "remove": True}
    ]
})

# remove the second OBX segment
response = send_request("editor/patchMessage", {
    "patches": [
        {"path": "OBX[2]", "remove": True}
    ]
})
```

## Mix Multiple Operations

```python
# update patient info, clear old data, and add next of kin
response = send_request("editor/patchMessage", {
    "patches": [
        # update patient name
        {"path": "PID.5.1", "value": "SMITH"},
        {"path": "PID.5.2", "value": "JANE"},

        # clear mother's maiden name
        {"path": "PID.6", "value": ""},

        # update patient class
        {"path": "PV1.2", "value": "I"},

        # add next of kin
        {"path": "NK1", "create": True},
        {"path": "NK1.2.1", "value": "SMITH"},
        {"path": "NK1.2.2", "value": "JOHN"},
        {"path": "NK1.3", "value": "SPOUSE"}
    ]
})

# check for partial success
if not response["result"]["success"]:
    log(f"Applied {response['result']['patchesApplied']} out of {len(patches)} patches")
    for error in response["result"].get("errors", []):
        log(f"  Patch {error['index']} ({error['path']}): {error['message']}")
```

## Format Comparison

| Format | Best For                                          |
| ------ | ------------------------------------------------- |
| `hl7`  | Direct manipulation, sending via MLLP, inspection |
| `json` | Programmatic access, complex transformations      |
| `yaml` | Human-readable inspection, configuration          |
| `toml` | Configuration files, simple structures            |

## Related Documentation

- [Reference: editor/getMessage](../reference/methods.md#editorgetmessage)
- [Reference: editor/patchMessage](../reference/methods.md#editorpatchmessage)
- [Reference: editor/setMessage](../reference/methods.md#editorsetmessage)
- [Tutorial: Building Your First Extension](../tutorials/first-extension.md)
