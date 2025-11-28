# command/execute

Trigger command execution (fire-and-forget notification).

## Direction

Hermes → Extension

## Type

Notification (no response expected)

## Timeout

None (fire-and-forget)

## Parameters

| Field   | Type   | Required | Description           |
| ------- | ------ | -------- | --------------------- |
| command | string | Yes      | Command ID to execute |

## Response

None. This is a notification; extensions must not send a response.

## Example Notification

```json
{
  "jsonrpc": "2.0",
  "method": "command/execute",
  "params": {
    "command": "myExtension/search"
  }
}
```

Note: No `id` field (notification, not request).

## Notes

- Extension handles command asynchronously
- Extension may make subsequent requests to Hermes (e.g.,
  `editor/getMessage`, `editor/patchMessage`)
- Progress and errors logged to stderr
