# message/saved

Notification sent when the current message is saved to disk.

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
      { "name": "message/saved" }
    ]
  }
}
```

## Parameters

| Field    | Type    | Required | Description               |
| -------- | ------- | -------- | ------------------------- |
| filePath | string  | Yes      | Path where file was saved |
| saveAs   | boolean | Yes      | True if Save As operation |

## Response

None. This is a notification; extensions must not send a response.

## Example Notification

```json
{
  "jsonrpc": "2.0",
  "method": "message/saved",
  "params": {
    "filePath": "/Users/user/messages/patient.hl7",
    "saveAs": false
  }
}
```

Note: No `id` field (notification, not request).

## Example Notification (Save As)

```json
{
  "jsonrpc": "2.0",
  "method": "message/saved",
  "params": {
    "filePath": "/Users/user/messages/patient-copy.hl7",
    "saveAs": true
  }
}
```

## Notes

- Sent after successful save completes
- `saveAs` distinguishes Save (false) from Save As (true)
- Auto-save triggers this notification with `saveAs: false`
- `filePath` is always present (save requires a destination)
