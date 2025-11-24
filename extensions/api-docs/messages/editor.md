# Editor Messages

These messages allow extensions to read and modify the HL7 message currently open in the Hermes editor.

## Direction

**Extension → Hermes**

---

## editor/getMessage

Retrieves the current message from the editor.

### Request

#### Method

```
editor/getMessage
```

#### Parameters

```typescript
interface GetMessageParams {
  /** Desired output format */
  format: "hl7" | "json" | "yaml" | "toml";
}
```

#### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "editor/getMessage",
  "params": {
    "format": "json"
  }
}
```

### Response

#### Result

```typescript
interface GetMessageResult {
  /** The message content in the requested format */
  message: string;

  /** Whether a file is currently open (has a path) */
  hasFile: boolean;

  /** The file path, if one is open */
  filePath?: string;
}
```

#### Success Response (HL7 format)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "message": "MSH|^~\\&|APP|FAC|APP|FAC|20231215120000||ADT^A01|123|P|2.5.1\rPID|1||12345^^^MRN||DOE^JOHN||19800101|M",
    "hasFile": true,
    "filePath": "/Users/user/messages/patient.hl7"
  }
}
```

#### Success Response (JSON format)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "message": "{\"MSH\":{\"1\":\"|\",\"2\":\"^~\\\\&\",\"3\":\"APP\",\"4\":\"FAC\",\"9\":{\"1\":\"ADT\",\"2\":\"A01\"}},\"PID\":{\"3\":{\"1\":\"12345\"},\"5\":{\"1\":\"DOE\",\"2\":\"JOHN\"}}}",
    "hasFile": false
  }
}
```

### Output Formats

#### HL7 (Raw)

The native HL7 pipe-delimited format with `\r` (carriage return, ASCII 13) segment separators:

```
MSH|^~\&|APP|FAC|APP|FAC|20231215120000||ADT^A01|123|P|2.5.1
PID|1||12345^^^MRN||DOE^JOHN||19800101|M
```

**Encoding note:** In the JSON response string, segment separators appear as the escape sequence `\r`. For example, a two-segment message returns as `"MSH|...\rPID|..."`. Parse with a JSON library first, then split on the literal carriage return character.

#### JSON

Hierarchical structure with 1-based string indices:

```json
{
  "MSH": {
    "1": "|",
    "2": "^~\\&",
    "3": "APP",
    "9": { "1": "ADT", "2": "A01" }
  },
  "PID": {
    "3": { "1": "12345", "4": "MRN" },
    "5": { "1": "DOE", "2": "JOHN" }
  }
}
```

#### YAML

Same structure as JSON in YAML format:

```yaml
MSH:
  "1": "|"
  "2": "^~\\&"
  "3": APP
  "9":
    "1": ADT
    "2": A01
PID:
  "3":
    "1": "12345"
  "5":
    "1": DOE
    "2": JOHN
```

#### TOML

Same structure as JSON in TOML format:

```toml
[MSH]
"1" = "|"
"2" = "^~\\&"
"3" = "APP"

[MSH."9"]
"1" = "ADT"
"2" = "A01"

[PID."5"]
"1" = "DOE"
"2" = "JOHN"
```

### Format Details

| Format | Best For                                        |
|--------|-------------------------------------------------|
| `hl7`  | Direct manipulation, sending via MLLP           |
| `json` | Programmatic access, complex transformations    |
| `yaml` | Human-readable inspection, configuration        |
| `toml` | Configuration files, simple structures          |

### Repeated Segments

When a message contains repeated segments (e.g., multiple OBX), they become arrays in structured formats:

```json
{
  "MSH": { ... },
  "OBX": [
    { "1": "1", "3": "CODE1", "5": "Value1" },
    { "1": "2", "3": "CODE2", "5": "Value2" }
  ]
}
```

### Empty Editor

If no message is open:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "message": "",
    "hasFile": false
  }
}
```

---

## editor/patchMessage

Modifies specific fields in the current message without replacing the entire message.

### Request

#### Method

```
editor/patchMessage
```

#### Parameters

```typescript
interface PatchMessageParams {
  /** List of patch operations to apply */
  patches: Patch[];
}

interface Patch {
  /** HL7 path to the field or segment (e.g., "PID.5.1", "OBX[2].5", "NK1") */
  path: string;

  /** New value for the field, or omit/null to remove */
  value?: string;

  /** Explicitly remove this path */
  remove?: boolean;

  /** Create a new segment (path must be segment name only, e.g., "NK1") */
  create?: boolean;
}
```

#### Example Request: Set Fields

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "editor/patchMessage",
  "params": {
    "patches": [
      { "path": "PID.5.1", "value": "DOE" },
      { "path": "PID.5.2", "value": "JOHN" },
      { "path": "PID.7", "value": "19800101" },
      { "path": "PID.8", "value": "M" }
    ]
  }
}
```

#### Example Request: Clear Fields

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "editor/patchMessage",
  "params": {
    "patches": [
      { "path": "PID.13", "value": "" },
      { "path": "PID.6", "value": "" }
    ]
  }
}
```

#### Example Request: Delete Segments

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "editor/patchMessage",
  "params": {
    "patches": [
      { "path": "NK1", "remove": true },
      { "path": "OBX[2]", "remove": true }
    ]
  }
}
```

**Note:** Use `remove: true` only for entire segments. To clear a field's value, set it to an empty string.

#### Example Request: Mixed Operations

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "editor/patchMessage",
  "params": {
    "patches": [
      { "path": "PID.5.1", "value": "SMITH" },
      { "path": "PID.6", "value": "" },
      { "path": "PV1.2", "value": "I" }
    ]
  }
}
```

#### Example Request: Create Segments

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "editor/patchMessage",
  "params": {
    "patches": [
      { "path": "NK1", "create": true },
      { "path": "NK1.2.1", "value": "DOE" },
      { "path": "NK1.2.2", "value": "JANE" },
      { "path": "NK1.3", "value": "SPOUSE" }
    ]
  }
}
```

When creating a segment:
- Use `create: true` with the segment name only (e.g., `"NK1"`, not `"NK1.1"`)
- If that segment type already exists, the new segment is appended after the last occurrence
- If the segment type doesn't exist, it's appended at the end of the message
- You can immediately set fields on the new segment in subsequent patches

**Indexed segment creation:** Use `{ "path": "OBX[2]", "create": true }` to create a specific instance. If earlier instances don't exist (e.g., creating `OBX[3]` when only `OBX[1]` exists), empty segments are created to fill the gap.

### Path Syntax

The path follows HL7 query syntax:

| Pattern           | Description                           | Example        |
|-------------------|---------------------------------------|----------------|
| `SEG.F`           | Field F of segment SEG                | `PID.5`        |
| `SEG.F.C`         | Component C of field F                | `PID.5.1`      |
| `SEG.F.C.S`       | Subcomponent S of component C         | `PID.5.1.1`    |
| `SEG[N].F`        | Field F of Nth occurrence of SEG      | `OBX[2].5`     |
| `SEG.F[N]`        | Nth repetition of field F             | `PID.13[2]`    |

**Note:** All indices are 1-based.

### Response

#### Result

```typescript
interface PatchMessageResult {
  /** Whether all patches were applied successfully */
  success: boolean;

  /** Number of patches applied */
  patchesApplied: number;

  /** Errors for patches that failed (if any) */
  errors?: PatchError[];
}

interface PatchError {
  /** Index of the failed patch (0-based) */
  index: number;

  /** The path that failed */
  path: string;

  /** Error description */
  message: string;
}
```

#### Success Response

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "success": true,
    "patchesApplied": 4
  }
}
```

#### Partial Success

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "success": false,
    "patchesApplied": 2,
    "errors": [
      {
        "index": 2,
        "path": "XYZ.1",
        "message": "Segment XYZ does not exist"
      }
    ]
  }
}
```

### Patch Behaviour

| Scenario                          | Result                                         |
|-----------------------------------|------------------------------------------------|
| Set field that exists             | Value is replaced                              |
| Set field that doesn't exist      | Field/component is created                     |
| Set field on missing segment      | Error (use `create: true` first)               |
| Clear field (`value: ""`)         | Field set to empty                             |
| Create segment (new type)         | Segment appended at end of message             |
| Create segment (existing type)    | New instance appended after last of that type  |
| Remove segment that exists        | Segment deleted from message                   |
| Remove segment that doesn't exist | Silently succeeds                              |
| Invalid path syntax               | Error                                          |

### Creating Fields

When setting a value for a field that doesn't exist:

- The field is created with empty values for any gaps
- Components are padded as needed

Example: Setting `PID.10` when only `PID.1-5` exist creates `PID.6-9` as empty.

### Best-Effort Application

Patches are applied using **best-effort** semantics:

- All patches are attempted in order
- Valid patches are applied even if others fail
- Invalid patches are skipped and reported in the `errors` array
- `patchesApplied` indicates how many succeeded
- `success` is `true` only if all patches were applied

This allows partial updates when some fields can't be modified (e.g., missing segments).

### Undo Behaviour

The entire `patchMessage` operation is recorded as a **single undo entry**. When the user presses Cmd+Z after an extension patches the message, all applied patches are reverted together.

---

## editor/setMessage

Replaces the entire message in the editor.

### Request

#### Method

```
editor/setMessage
```

#### Parameters

```typescript
interface SetMessageParams {
  /** The message content */
  message: string;

  /** Format of the message being set */
  format: "hl7" | "json" | "yaml" | "toml";
}
```

#### Example Request (HL7 format)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "editor/setMessage",
  "params": {
    "message": "MSH|^~\\&|APP|FAC|||20231215120000||ADT^A01|123|P|2.5.1\rPID|1||12345||DOE^JOHN||19800101|M",
    "format": "hl7"
  }
}
```

#### Example Request (JSON format)

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "editor/setMessage",
  "params": {
    "message": "{\"MSH\":{\"1\":\"|\",\"2\":\"^~\\\\&\",\"3\":\"APP\"},\"PID\":{\"5\":{\"1\":\"DOE\",\"2\":\"JOHN\"}}}",
    "format": "json"
  }
}
```

### Response

#### Result

```typescript
interface SetMessageResult {
  /** Whether the message was set successfully */
  success: boolean;

  /** Error message if failed */
  error?: string;
}
```

#### Success Response

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "success": true
  }
}
```

#### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "success": false,
    "error": "Invalid HL7 message: MSH segment missing"
  }
}
```

### Validation

When setting a message, Hermes validates:

| Check               | Behaviour on Failure           |
|---------------------|--------------------------------|
| Valid format        | Returns error                  |
| MSH segment present | Returns error                  |
| Basic HL7 structure | Returns error                  |

### Side Effects

Setting a message:

1. Replaces the editor content
2. Clears undo history (the operation itself becomes a single undoable action)
3. Marks the document as modified (unsaved)
4. Does **not** change the file path (if one was open)

---

## Common Patterns

### Read-Modify-Write

The safest pattern for modifying messages:

```python
# 1. Get current message
response = send_request("editor/getMessage", {"format": "json"})
message = json.loads(response["result"]["message"])

# 2. Modify as needed
message["PID"]["5"]["1"] = "NEWNAME"

# 3. Set the modified message
send_request("editor/setMessage", {
    "message": json.dumps(message),
    "format": "json"
})
```

### Targeted Patches

For simple field updates, patching is more efficient:

```python
send_request("editor/patchMessage", {
    "patches": [
        {"path": "PID.5.1", "value": "NEWNAME"}
    ]
})
```

### Checking Before Modifying

```python
# check if message exists
response = send_request("editor/getMessage", {"format": "hl7"})
if not response["result"]["message"]:
    return {"success": False, "message": "No message open"}

# proceed with modification
```

## Related Documentation

- [Commands](commands.md) - Triggering editor operations from commands
- [Types](../types.md) - Type definitions
- [Errors](../errors.md) - Error handling
