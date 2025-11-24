# Type Definitions

This document defines all types used in the Hermes Extension API. Types are expressed in TypeScript notation for clarity, but the actual protocol uses JSON.

## JSON-RPC Types

### Request

```typescript
interface Request {
  /** Must be "2.0" */
  jsonrpc: "2.0";

  /** Unique identifier for this request */
  id: number | string;

  /** Method name to invoke */
  method: string;

  /** Method parameters */
  params?: object;
}
```

### SuccessResponse

```typescript
interface SuccessResponse {
  /** Must be "2.0" */
  jsonrpc: "2.0";

  /** Must match the request id */
  id: number | string;

  /** Method-specific result */
  result: unknown;
}
```

### ErrorResponse

```typescript
interface ErrorResponse {
  /** Must be "2.0" */
  jsonrpc: "2.0";

  /** Must match the request id, or null if id couldn't be determined */
  id: number | string | null;

  /** Error details */
  error: RpcError;
}
```

### RpcError

```typescript
interface RpcError {
  /** Error code (see errors.md) */
  code: number;

  /** Short error description */
  message: string;

  /** Additional error details */
  data?: unknown;
}
```

---

## Extension Metadata

### InitializeParams

```typescript
interface InitializeParams {
  /** Version of Hermes (e.g., "1.0.0") */
  hermesVersion: string;

  /** Version of the Extension API (e.g., "1.0.0") */
  apiVersion: string;

  /** Path to Hermes data directory */
  dataDirectory: string;
}
```

### InitializeResult

```typescript
interface InitializeResult {
  /** Display name of the extension */
  name: string;

  /** Semantic version (e.g., "1.0.0") */
  version: string;

  /** Brief description */
  description?: string;

  /** List of authors */
  authors?: string[];

  /** Homepage or documentation URL */
  homepage?: string;

  /** Extension capabilities */
  capabilities: Capabilities;

  /** Toolbar buttons to register */
  toolbarButtons?: ToolbarButton[];

  /** Schema overrides */
  schema?: SchemaOverride;
}
```

### Capabilities

```typescript
interface Capabilities {
  /** Command IDs this extension can handle */
  commands?: string[];

  /** Extension provides schema overrides */
  schemaProvider?: boolean;
}
```

**Note:** The `commands` array lists command IDs that the extension can handle via
`command/execute`. This allows extensions to register commands independently of toolbar
buttons. Commands from `toolbarButtons[].command` are automatically included.

---

## Toolbar

### ToolbarButton

```typescript
interface ToolbarButton {
  /** Unique identifier for this button */
  id: string;

  /** Tooltip text */
  label: string;

  /** SVG icon markup */
  icon: string;

  /** Command ID to execute when clicked */
  command: string;

  /** Visual grouping (optional) */
  group?: string;
}
```

**Icon Requirements:**
- Must be valid SVG
- Should use `viewBox` attribute (not fixed width/height)
- Use `currentColor` for stroke and fill to inherit theme colours
- Keep simple for visibility at 20×20 pixels

---

## Commands

### CommandExecuteParams

```typescript
interface CommandExecuteParams {
  /** Command identifier */
  command: string;
}
```

### CommandExecuteResult

```typescript
interface CommandExecuteResult {
  /** Whether the command succeeded */
  success: boolean;

  /** Message to display to user */
  message?: string;
}
```

---

## Editor Operations

### MessageFormat

```typescript
type MessageFormat = "hl7" | "json" | "yaml" | "toml";
```

### GetMessageParams

```typescript
interface GetMessageParams {
  /** Desired output format */
  format: MessageFormat;
}
```

### GetMessageResult

```typescript
interface GetMessageResult {
  /** Message content in requested format */
  message: string;

  /** Whether a file is currently open */
  hasFile: boolean;

  /** Path to the open file (if any) */
  filePath?: string;
}
```

### PatchMessageParams

```typescript
interface PatchMessageParams {
  /** List of patches to apply */
  patches: Patch[];
}
```

### Patch

```typescript
interface Patch {
  /** HL7 path or segment name (e.g., "PID.5.1", "OBX[2].3", "NK1") */
  path: string;

  /** New value for the field */
  value?: string;

  /** Delete an entire segment (path must be segment reference, e.g., "NK1" or "OBX[2]") */
  remove?: boolean;

  /** Create a new segment (path must be segment name only) */
  create?: boolean;
}
```

**Usage:**
- Set a field: `{ "path": "PID.5.1", "value": "DOE" }`
- Clear a field: `{ "path": "PID.5.1", "value": "" }`
- Delete a segment: `{ "path": "NK1", "remove": true }` or `{ "path": "OBX[2]", "remove": true }`
- Create a segment: `{ "path": "NK1", "create": true }`

Note: `remove` is for deleting entire segments, not individual fields. To clear a field's value, set it to an empty string.

### PatchMessageResult

```typescript
interface PatchMessageResult {
  /** Whether all patches were applied */
  success: boolean;

  /** Number of patches applied */
  patchesApplied: number;

  /** Errors for failed patches */
  errors?: PatchError[];
}
```

### PatchError

```typescript
interface PatchError {
  /** Index of the failed patch (0-based) */
  index: number;

  /** The path that failed */
  path: string;

  /** Error description */
  message: string;
}
```

### SetMessageParams

```typescript
interface SetMessageParams {
  /** Message content */
  message: string;

  /** Format of the message */
  format: MessageFormat;
}
```

### SetMessageResult

```typescript
interface SetMessageResult {
  /** Whether the message was set */
  success: boolean;

  /** Error message if failed */
  error?: string;
}
```

---

## UI Operations

### OpenWindowParams

```typescript
interface OpenWindowParams {
  /** URL to load */
  url: string;

  /** Window title */
  title: string;

  /** Width in pixels */
  width?: number;

  /** Height in pixels */
  height?: number;

  /** Whether window is modal */
  modal?: boolean;

  /** Whether window is resizable */
  resizable?: boolean;
}
```

### OpenWindowResult

```typescript
interface OpenWindowResult {
  /** Unique window identifier */
  windowId: string;
}
```

### CloseWindowParams

```typescript
interface CloseWindowParams {
  /** The window ID returned from ui/openWindow */
  windowId: string;
}
```

### CloseWindowResult

```typescript
interface CloseWindowResult {
  /** Whether the window was closed */
  success: boolean;
}
```

### WindowClosedParams

Notification sent when a window is closed.

```typescript
interface WindowClosedParams {
  /** The window ID that was closed */
  windowId: string;

  /** How the window was closed */
  reason: "user" | "extension" | "shutdown";
}
```

---

## Shutdown

### ShutdownParams

```typescript
interface ShutdownParams {
  /** Reason for shutdown */
  reason?: "closing" | "disabled" | "reload" | "error";
}
```

### ShutdownResult

```typescript
interface ShutdownResult {
  /** Whether shutdown completed successfully */
  success: boolean;
}
```

---

## Schema Types

### SchemaOverride

```typescript
interface SchemaOverride {
  /** Segment overrides */
  segments?: {
    [segmentName: string]: SegmentOverride;
  };
}
```

### SegmentOverride

```typescript
interface SegmentOverride {
  /** Field overrides */
  fields?: FieldOverride[];
}
```

### FieldOverride

```typescript
interface FieldOverride {
  /** 1-based field number */
  field: number;

  /** 1-based component number */
  component?: number;

  /** Human-readable field name */
  name?: string;

  /** UI grouping */
  group?: string;

  /** Help text / notes */
  note?: string;

  /** Minimum length */
  minlength?: number;

  /** Maximum length */
  maxlength?: number;

  /** Regex pattern for validation */
  pattern?: string;

  /** Whether field is required */
  required?: boolean;

  /** Data type for special handling */
  datatype?: "date" | "datetime";

  /** Placeholder text for UI */
  placeholder?: string;

  /** Allowed values (code → description) */
  values?: { [code: string]: string };

  /** Default value for templates */
  template?: string;
}
```

---

## HL7 Path Syntax

The `path` field in patches uses HL7 query syntax:

| Pattern           | Description                           | Example        |
|-------------------|---------------------------------------|----------------|
| `SEG.F`           | Field F of segment SEG                | `PID.5`        |
| `SEG.F.C`         | Component C of field F                | `PID.5.1`      |
| `SEG.F.C.S`       | Subcomponent S                        | `PID.5.1.1`    |
| `SEG[N].F`        | Field F of Nth segment occurrence     | `OBX[2].5`     |
| `SEG.F[N]`        | Nth repetition of field F             | `PID.13[2]`    |
| `SEG[N].F.C`      | Component C of Nth segment            | `OBX[3].5.1`   |

All indices are **1-based**.

---

## Message Format Examples

### HL7 (Raw)

```
MSH|^~\&|APP|FAC|||20231215120000||ADT^A01|123|P|2.5.1
PID|1||12345^^^MRN||DOE^JOHN^Q||19800101|M
```

### JSON

```json
{
  "MSH": {
    "1": "|",
    "2": "^~\\&",
    "3": "APP",
    "4": "FAC",
    "7": "20231215120000",
    "9": {
      "1": "ADT",
      "2": "A01"
    },
    "10": "123",
    "11": "P",
    "12": "2.5.1"
  },
  "PID": {
    "1": "1",
    "3": {
      "1": "12345",
      "4": "MRN"
    },
    "5": {
      "1": "DOE",
      "2": "JOHN",
      "3": "Q"
    },
    "7": "19800101",
    "8": "M"
  }
}
```

### YAML

```yaml
MSH:
  "1": "|"
  "2": "^~\\&"
  "3": "APP"
  "4": "FAC"
  "7": "20231215120000"
  "9":
    "1": "ADT"
    "2": "A01"
  "10": "123"
  "11": "P"
  "12": "2.5.1"
PID:
  "1": "1"
  "3":
    "1": "12345"
    "4": "MRN"
  "5":
    "1": "DOE"
    "2": "JOHN"
    "3": "Q"
  "7": "19800101"
  "8": "M"
```

### Repeated Segments

When a message contains repeated segments, they become arrays in structured formats.

#### JSON

```json
{
  "MSH": { ... },
  "PID": { ... },
  "OBX": [
    {
      "1": "1",
      "3": { "1": "GLUCOSE" },
      "5": "95"
    },
    {
      "1": "2",
      "3": { "1": "CREATININE" },
      "5": "1.2"
    }
  ]
}
```

#### YAML

```yaml
MSH:
  # ...
PID:
  # ...
OBX:
  - "1": "1"
    "3":
      "1": "GLUCOSE"
    "5": "95"
  - "1": "2"
    "3":
      "1": "CREATININE"
    "5": "1.2"
```

#### TOML

TOML uses `[[section]]` syntax for arrays of tables:

```toml
[MSH]
# ...

[PID]
# ...

[[OBX]]
"1" = "1"
"5" = "95"

[OBX."3"]
"1" = "GLUCOSE"

[[OBX]]
"1" = "2"
"5" = "1.2"

[OBX."3"]
"1" = "CREATININE"
```

**Note:** TOML's nested table syntax for repeated segments can be verbose. JSON or YAML are often more practical for complex messages with many repeated segments.

---

## JSON Encoding Notes

### Escaping

In JSON strings:
- Backslash: `\\`
- Quote: `\"`
- Newline: `\n`
- Carriage return: `\r`
- Tab: `\t`

The HL7 escape character `\` becomes `\\` in JSON:
- MSH.2 `^~\&` → `"^~\\&"`

### Field Indices as Strings

Field indices are **strings**, not numbers:

```json
{
  "PID": {
    "5": { "1": "DOE", "2": "JOHN" }
  }
}
```

Not:
```json
{
  "PID": {
    5: { 1: "DOE", 2: "JOHN" }
  }
}
```

### Empty Values

Empty fields are omitted from the structured formats:

HL7:
```
PID|1||12345||DOE^JOHN
```

JSON (PID.2 and PID.4 are missing, not null):
```json
{
  "PID": {
    "1": "1",
    "3": "12345",
    "5": { "1": "DOE", "2": "JOHN" }
  }
}
```

---

## Related Documentation

- [Schema](schema.md) - Schema format details
- [Messages](messages/README.md) - Message reference
- [Errors](errors.md) - Error codes
