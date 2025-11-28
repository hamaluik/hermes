# shutdown

Request to terminate extension gracefully.

## Direction

Hermes → Extension

## Type

Request (expects response)

## Timeout

5 seconds. Process killed with SIGKILL if exceeded.

## Parameters

| Field  | Type   | Required | Description            |
| ------ | ------ | -------- | ---------------------- |
| reason | string | No       | Why shutdown requested |

### Reason Values

- `"closing"` - Hermes is closing
- `"disabled"` - Extension disabled in settings
- `"reload"` - Extension configuration changed
- `"error"` - Error occurred

## Response

| Field   | Type    | Required | Description                |
| ------- | ------- | -------- | -------------------------- |
| success | boolean | Yes      | Shutdown completed cleanly |

## Error Codes

- `-32000` General error

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "method": "shutdown",
  "params": {
    "reason": "closing"
  }
}
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "result": {
    "success": true
  }
}
```
