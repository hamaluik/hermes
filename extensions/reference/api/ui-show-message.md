# ui/showMessage

Display system-native message dialogue.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field   | Type   | Required | Default  | Description      |
|---------|--------|----------|----------|------------------|
| message | string | Yes      | -        | Message text     |
| title   | string | No       | -        | Dialogue title   |
| kind    | string | No       | `"info"` | Dialogue styling |

### Kind Values

- `"info"` - Informational message
- `"warning"` - Warning message
- `"error"` - Error message

## Response

| Field        | Type    | Required | Description                      |
|--------------|---------|----------|----------------------------------|
| acknowledged | boolean | Yes      | Always true (user clicked OK)    |

## Error Codes

- `-32012` Dialogue error (failed to show dialogue)

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ui/showMessage",
  "params": {
    "message": "Patient data imported successfully.",
    "title": "Import Complete",
    "kind": "info"
  }
}
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "acknowledged": true
  }
}
```
