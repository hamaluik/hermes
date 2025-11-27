# ui/saveFile

Display file save dialogue.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field       | Type         | Required | Description          |
|-------------|--------------|----------|----------------------|
| title       | string       | No       | Dialogue title       |
| defaultPath | string       | No       | Starting directory   |
| defaultName | string       | No       | Default filename     |
| filters     | FileFilter[] | No       | File type filters    |

### FileFilter

| Field      | Type     | Required | Description                          |
|------------|----------|----------|--------------------------------------|
| name       | string   | Yes      | Display name (e.g., "HL7 Files")     |
| extensions | string[] | Yes      | Extensions without dots (e.g., `["hl7"]`) |

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
  "id": 5,
  "method": "ui/saveFile",
  "params": {
    "title": "Export Message",
    "defaultName": "message.hl7",
    "filters": [
      {"name": "HL7 Files", "extensions": ["hl7"]}
    ]
  }
}
```

## Example Response (Path Selected)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "path": "/Users/user/Documents/exported.hl7"
  }
}
```

## Example Response (Cancelled)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "path": null
  }
}
```
