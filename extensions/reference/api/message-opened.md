# message/opened

Notification sent when a message file is opened or a new message is created.

## Direction

Hermes → Extension

## Type

Notification (no response expected)

## Timeout

None (notification)

## Subscription

Extensions must subscribe to this event via the `events` array in their
`initialize` response:

```json
{
  "capabilities": {
    "events": [
      { "name": "message/opened" }
    ]
  }
}
```

## Parameters

| Field    | Type    | Required | Description                              |
|----------|---------|----------|------------------------------------------|
| filePath | string  | No       | Path to opened file (omitted if new)     |
| isNew    | boolean | Yes      | True if new/untitled message             |

## Response

None. This is a notification; extensions must not send a response.

## Example Notification (file opened)

```json
{
  "jsonrpc": "2.0",
  "method": "message/opened",
  "params": {
    "filePath": "/Users/user/messages/patient.hl7",
    "isNew": false
  }
}
```

Note: No `id` field (notification, not request).

## Example Notification (new message)

```json
{
  "jsonrpc": "2.0",
  "method": "message/opened",
  "params": {
    "isNew": true
  }
}
```

## Notes

- Sent when user opens a file via File > Open or drag-and-drop
- Sent when user creates a new message via File > New
- Sent when user creates from template via File > New from Template
- When `isNew` is true, `filePath` is omitted (untitled message)
- Extensions can call `editor/getMessage` to retrieve the content if needed
