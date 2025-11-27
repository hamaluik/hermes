# Type Definitions

All types expressed in TypeScript notation. Actual protocol uses JSON.

## JSON-RPC Types

### Request

```typescript
interface Request {
  jsonrpc: "2.0";
  id: number | string;
  method: string;
  params?: object;
}
```

### SuccessResponse

```typescript
interface SuccessResponse {
  jsonrpc: "2.0";
  id: number | string;
  result: unknown;
}
```

### ErrorResponse

```typescript
interface ErrorResponse {
  jsonrpc: "2.0";
  id: number | string | null;
  error: RpcError;
}
```

### RpcError

```typescript
interface RpcError {
  code: number;
  message: string;
  data?: unknown;
}
```

## Extension Metadata

### InitializeParams

```typescript
interface InitializeParams {
  hermesVersion: string;
  apiVersion: string;
  dataDirectory: string;
}
```

### InitializeResult

```typescript
interface InitializeResult {
  name: string;
  version: string;
  description?: string;
  authors?: string[];
  homepage?: string;
  capabilities: Capabilities;
  toolbarButtons?: ToolbarButton[];
  schema?: SchemaOverride;
}
```

### Capabilities

```typescript
interface Capabilities {
  commands?: string[];
  schemaProvider?: boolean;
  events?: EventSubscription[];
}
```

### EventSubscription

```typescript
interface EventSubscription {
  name: EventName;
  options?: EventOptions;
}
```

### EventName

```typescript
type EventName = "message/changed" | "message/opened" | "message/saved";
```

### EventOptions

Options for event subscriptions. Currently only `message/changed` uses options.

```typescript
interface EventOptions {
  includeContent?: boolean;  // include message content (default: false)
  format?: MessageFormat;    // format for content (default: "hl7")
}
```

## Toolbar

### ToolbarButton

```typescript
interface ToolbarButton {
  id: string;
  label: string;
  icon: string;
  command: string;
  group?: string;
}
```

Icon requirements:
- Valid SVG markup
- Use `viewBox` attribute
- Use `currentColor` for theming
- Optimised for 20×20 pixels

## Commands

### CommandExecuteParams

```typescript
interface CommandExecuteParams {
  command: string;
}
```

## Editor Operations

### MessageFormat

```typescript
type MessageFormat = "hl7" | "json" | "yaml" | "toml";
```

### GetMessageParams

```typescript
interface GetMessageParams {
  format: MessageFormat;
}
```

### GetMessageResult

```typescript
interface GetMessageResult {
  message: string;
  hasFile: boolean;
  filePath?: string;
}
```

### PatchMessageParams

```typescript
interface PatchMessageParams {
  patches: Patch[];
}
```

### Patch

```typescript
interface Patch {
  path: string;
  value?: string;
  remove?: boolean;
  create?: boolean;
}
```

Path syntax (1-based indices):
- `SEG.F` - field F of segment
- `SEG.F.C` - component C of field F
- `SEG.F.C.S` - subcomponent S
- `SEG[N].F` - field F of Nth segment occurrence
- `SEG.F[N]` - Nth repetition of field F

### PatchMessageResult

```typescript
interface PatchMessageResult {
  success: boolean;
  patchesApplied: number;
  errors?: PatchError[];
}
```

### PatchError

```typescript
interface PatchError {
  index: number;
  path: string;
  message: string;
}
```

### SetMessageParams

```typescript
interface SetMessageParams {
  message: string;
  format: MessageFormat;
}
```

### SetMessageResult

```typescript
interface SetMessageResult {
  success: boolean;
  error?: string;
}
```

## UI Operations

### OpenWindowParams

```typescript
interface OpenWindowParams {
  url: string;
  title: string;
  width?: number;
  height?: number;
  modal?: boolean;
  resizable?: boolean;
}
```

### OpenWindowResult

```typescript
interface OpenWindowResult {
  windowId: string;
}
```

### CloseWindowParams

```typescript
interface CloseWindowParams {
  windowId: string;
}
```

### CloseWindowResult

```typescript
interface CloseWindowResult {
  success: boolean;
}
```

### WindowClosedParams

```typescript
interface WindowClosedParams {
  windowId: string;
  reason: "user" | "extension" | "shutdown";
}
```

### MessageChangedParams

```typescript
interface MessageChangedParams {
  message?: string;       // present if includeContent=true
  format?: MessageFormat; // present if includeContent=true
  hasFile: boolean;
  filePath?: string;      // present if hasFile=true
}
```

### MessageOpenedParams

```typescript
interface MessageOpenedParams {
  filePath?: string;  // present if isNew=false
  isNew: boolean;
}
```

### MessageSavedParams

```typescript
interface MessageSavedParams {
  filePath: string;
  saveAs: boolean;
}
```

### ShowMessageParams

```typescript
interface ShowMessageParams {
  message: string;
  title?: string;
  kind?: "info" | "warning" | "error";
}
```

### ShowMessageResult

```typescript
interface ShowMessageResult {
  acknowledged: boolean;
}
```

### ShowConfirmParams

```typescript
interface ShowConfirmParams {
  message: string;
  title?: string;
  buttons?: "yesNo" | "okCancel";
}
```

### ShowConfirmResult

```typescript
interface ShowConfirmResult {
  confirmed: boolean;
}
```

### FileFilter

```typescript
interface FileFilter {
  name: string;
  extensions: string[];
}
```

Extensions specified without dots (e.g., `["hl7", "txt"]`).

### OpenFileParams

```typescript
interface OpenFileParams {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
}
```

### OpenFileResult

```typescript
interface OpenFileResult {
  path: string | null;
}
```

### OpenFilesParams

```typescript
interface OpenFilesParams {
  title?: string;
  defaultPath?: string;
  filters?: FileFilter[];
}
```

### OpenFilesResult

```typescript
interface OpenFilesResult {
  paths: string[] | null;
}
```

### SaveFileParams

```typescript
interface SaveFileParams {
  title?: string;
  defaultPath?: string;
  defaultName?: string;
  filters?: FileFilter[];
}
```

### SaveFileResult

```typescript
interface SaveFileResult {
  path: string | null;
}
```

### SelectDirectoryParams

```typescript
interface SelectDirectoryParams {
  title?: string;
  defaultPath?: string;
}
```

### SelectDirectoryResult

```typescript
interface SelectDirectoryResult {
  path: string | null;
}
```

## Shutdown

### ShutdownParams

```typescript
interface ShutdownParams {
  reason?: "closing" | "disabled" | "reload" | "error";
}
```

### ShutdownResult

```typescript
interface ShutdownResult {
  success: boolean;
}
```

## Schema Types

### SchemaOverride

```typescript
interface SchemaOverride {
  segments?: {
    [segmentName: string]: SegmentOverride;
  };
}
```

### SegmentOverride

```typescript
interface SegmentOverride {
  fields?: FieldOverride[];
}
```

### FieldOverride

Field overrides use three-state semantics:

| JSON Value | Meaning                           |
|------------|-----------------------------------|
| Absent     | Inherit from base schema          |
| `null`     | Explicitly unset (remove value)   |
| Value      | Override with this value          |

```typescript
interface FieldOverride {
  field: number;
  component?: number;
  name?: string | null;
  group?: string | null;
  note?: string | null;
  minlength?: number | null;
  maxlength?: number | null;
  pattern?: string | null;
  required?: boolean | null;
  datatype?: "date" | "datetime" | null;
  placeholder?: string | null;
  values?: { [code: string]: string } | null;
  template?: string | null;
}
```

## Message Format Examples

### HL7 (Raw)

```
MSH|^~\&|APP|FAC|||20231215120000||ADT^A01|123|P|2.5.1
PID|1||12345^^^MRN||DOE^JOHN^Q||19800101|M
```

Segment separator: `\r` (carriage return, ASCII 13)

### JSON

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
    "5": { "1": "DOE", "2": "JOHN", "3": "Q" }
  }
}
```

Field indices as strings, 1-based. Empty fields omitted.

### YAML

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
    "4": MRN
  "5":
    "1": DOE
    "2": JOHN
    "3": Q
```

### TOML

```toml
[MSH]
"1" = "|"
"2" = "^~\\&"
"3" = "APP"

[MSH."9"]
"1" = "ADT"
"2" = "A01"

[PID."3"]
"1" = "12345"
"4" = "MRN"

[PID."5"]
"1" = "DOE"
"2" = "JOHN"
"3" = "Q"
```

### Repeated Segments

Repeated segments become arrays in structured formats:

```json
{
  "MSH": { ... },
  "PID": { ... },
  "OBX": [
    { "1": "1", "3": "CODE1", "5": "Value1" },
    { "1": "2", "3": "CODE2", "5": "Value2" }
  ]
}
```
