# ui/showConfirm

Display system-native confirmation dialogue.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field   | Type   | Required | Default    | Description      |
|---------|--------|----------|------------|------------------|
| message | string | Yes      | -          | Question text    |
| title   | string | No       | -          | Dialogue title   |
| buttons | string | No       | `"yesNo"`  | Button style     |

### Buttons Values

- `"yesNo"` - Yes/No buttons
- `"okCancel"` - OK/Cancel buttons

## Response

| Field     | Type    | Required | Description                     |
|-----------|---------|----------|---------------------------------|
| confirmed | boolean | Yes      | True if user clicked Yes/OK     |

## Error Codes

- `-32012` Dialogue error (failed to show dialogue)

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "ui/showConfirm",
  "params": {
    "message": "Overwrite existing patient data?",
    "title": "Confirm Overwrite",
    "buttons": "yesNo"
  }
}
```

## Example Response (Confirmed)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "confirmed": true
  }
}
```

## Example Response (Declined)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "confirmed": false
  }
}
```
