# ui/selectDirectory

Display directory selection dialogue.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field       | Type   | Required | Description        |
| ----------- | ------ | -------- | ------------------ |
| title       | string | No       | Dialogue title     |
| defaultPath | string | No       | Starting directory |

## Response

| Field | Type   | Required | Description                              |
| ----- | ------ | -------- | ---------------------------------------- |
| path  | string | Yes      | Selected directory, or null if cancelled |

## Error Codes

- `-32012` Dialogue error (failed to show dialogue)

User cancellation is not an error; returns `path: null`.

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "ui/selectDirectory",
  "params": {
    "title": "Select Output Folder",
    "defaultPath": "/Users/user/Documents"
  }
}
```

## Example Response (Directory Selected)

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "path": "/Users/user/Documents/exports"
  }
}
```

## Example Response (Cancelled)

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "path": null
  }
}
```
