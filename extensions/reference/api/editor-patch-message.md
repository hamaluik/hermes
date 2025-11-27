# editor/patchMessage

Modify specific fields without replacing entire message.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field   | Type    | Required | Description               |
|---------|---------|----------|---------------------------|
| patches | Patch[] | Yes      | List of patches to apply  |

### Patch

| Field  | Type    | Required | Description                           |
|--------|---------|----------|---------------------------------------|
| path   | string  | Yes      | HL7 path (e.g., `PID.5.1`, `OBX[2].3`)|
| value  | string  | No       | New value (omit to clear)             |
| remove | boolean | No       | Remove entire segment                 |
| create | boolean | No       | Create segment if missing             |

### Path Syntax (1-based)

| Pattern      | Description                     | Example    |
|--------------|---------------------------------|------------|
| `SEG.F`      | Field F of segment              | `PID.5`    |
| `SEG.F.C`    | Component C of field F          | `PID.5.1`  |
| `SEG.F.C.S`  | Subcomponent S                  | `PID.5.1.1`|
| `SEG[N].F`   | Field F of Nth segment          | `OBX[2].5` |
| `SEG.F[N]`   | Nth repetition of field F       | `PID.13[2]`|

### Patch Operations

- Set field: `{"path": "PID.5.1", "value": "DOE"}`
- Clear field: `{"path": "PID.5.1", "value": ""}`
- Delete segment: `{"path": "NK1", "remove": true}`
- Create segment: `{"path": "NK1", "create": true}`

## Response

| Field          | Type         | Required | Description                 |
|----------------|--------------|----------|-----------------------------|
| success        | boolean      | Yes      | All patches applied?        |
| patchesApplied | number       | Yes      | Count of successful patches |
| errors         | PatchError[] | No       | Details for failed patches  |

### PatchError

| Field   | Type   | Required | Description         |
|---------|--------|----------|---------------------|
| index   | number | Yes      | 0-based patch index |
| path    | string | Yes      | Path that failed    |
| message | string | Yes      | Error description   |

## Error Codes

- `-32005` Invalid path (syntax error)
- `-32006` Path not found (path doesn't exist in message)

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "editor/patchMessage",
  "params": {
    "patches": [
      {"path": "PID.5.1", "value": "DOE"},
      {"path": "PID.5.2", "value": "JOHN"},
      {"path": "PID.7", "value": "19800101"}
    ]
  }
}
```

## Example Response (Success)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "success": true,
    "patchesApplied": 3
  }
}
```

## Example Response (Partial)

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
