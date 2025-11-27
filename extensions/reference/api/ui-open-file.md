# ui/openFile

Display file picker for selecting single file.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field       | Type         | Required | Description          |
|-------------|--------------|----------|----------------------|
| title       | string       | No       | Dialogue title       |
| defaultPath | string       | No       | Starting directory   |
| filters     | FileFilter[] | No       | File type filters    |

### FileFilter

| Field      | Type     | Required | Description                          |
|------------|----------|----------|--------------------------------------|
| name       | string   | Yes      | Display name (e.g., "HL7 Files")     |
| extensions | string[] | Yes      | Extensions without dots (e.g., `["hl7", "txt"]`) |

## Response

| Field | Type   | Required | Description                         |
|-------|--------|----------|-------------------------------------|
| path  | string | Yes      | Selected path, or null if cancelled |

## Error Codes

- `-32012` Dialogue error (failed to show dialogue)

User cancellation is not an error; returns `path: null`.

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "ui/openFile",
  "params": {
    "title": "Select HL7 Message",
    "defaultPath": "/Users/user/Documents",
    "filters": [
      {"name": "HL7 Files", "extensions": ["hl7", "txt"]},
      {"name": "All Files", "extensions": ["*"]}
    ]
  }
}
```

## Example Response (File Selected)

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "path": "/Users/user/Documents/message.hl7"
  }
}
```

## Example Response (Cancelled)

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "path": null
  }
}
```
