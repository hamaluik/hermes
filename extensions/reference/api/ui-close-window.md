# ui/closeWindow

Close window opened by extension.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field    | Type   | Required | Description                        |
|----------|--------|----------|------------------------------------|
| windowId | string | Yes      | Window ID from `ui/openWindow`     |

## Response

| Field   | Type    | Required | Description              |
|---------|---------|----------|--------------------------|
| success | boolean | Yes      | Whether window was closed|

## Error Codes

- `-32008` Window error (window ID not recognised)

## Behaviour

| Scenario                 | Result                        |
|--------------------------|-------------------------------|
| Window open              | Closes window, returns true   |
| Window already closed    | Returns true (no-op)          |
| Window ID not recognised | Returns error -32008          |

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "ui/closeWindow",
  "params": {
    "windowId": "ext-window-abc123"
  }
}
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "success": true
  }
}
```
