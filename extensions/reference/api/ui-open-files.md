# ui/openFiles

Display file picker for selecting multiple files.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field       | Type         | Required | Description        |
| ----------- | ------------ | -------- | ------------------ |
| title       | string       | No       | Dialogue title     |
| defaultPath | string       | No       | Starting directory |
| filters     | FileFilter[] | No       | File type filters  |

### FileFilter

| Field      | Type     | Required | Description                                      |
| ---------- | -------- | -------- | ------------------------------------------------ |
| name       | string   | Yes      | Display name (e.g., "HL7 Files")                 |
| extensions | string[] | Yes      | Extensions without dots (e.g., `["hl7", "txt"]`) |

## Response

| Field | Type     | Required | Description                          |
| ----- | -------- | -------- | ------------------------------------ |
| paths | string[] | Yes      | Selected paths, or null if cancelled |

## Error Codes

- `-32012` Dialogue error (failed to show dialogue)

User cancellation is not an error; returns `paths: null`.

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "ui/openFiles",
  "params": {
    "title": "Select Messages to Import",
    "filters": [
      {"name": "HL7 Files", "extensions": ["hl7"]}
    ]
  }
}
```

## Example Response (Files Selected)

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "paths": [
      "/Users/user/Documents/message1.hl7",
      "/Users/user/Documents/message2.hl7"
    ]
  }
}
```

## Example Response (Cancelled)

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "paths": null
  }
}
```
