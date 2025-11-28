# message/changed

Notification sent when the editor message content changes.

## Direction

Hermes → Extension

## Type

Notification (no response expected)

## Timeout

None (notification)

## Debouncing

Hermes debounces rapid changes (500ms) to avoid flooding extensions during
typing. Multiple edits within the debounce window result in a single
notification.

## Subscription

Extensions must subscribe to this event via the `events` array in their
`initialize` response:

```json
{
  "capabilities": {
    "events": [
      { "name": "message/changed" }
    ]
  }
}
```

To receive message content with each notification:

```json
{
  "capabilities": {
    "events": [
      {
        "name": "message/changed",
        "options": { "includeContent": true, "format": "json" }
      }
    ]
  }
}
```

## Parameters

| Field    | Type          | Required | Description                              |
| -------- | ------------- | -------- | ---------------------------------------- |
| message  | string        | No       | Message content (if includeContent=true) |
| format   | MessageFormat | No       | Format of message (if included)          |
| hasFile  | boolean       | Yes      | Whether message has an associated file   |
| filePath | string        | No       | File path (if hasFile is true)           |

### Subscription Options

| Option         | Type          | Default | Description                        |
| -------------- | ------------- | ------- | ---------------------------------- |
| includeContent | boolean       | false   | Include message content            |
| format         | MessageFormat | "hl7"   | Format when includeContent is true |

## Response

None. This is a notification; extensions must not send a response.

## Example Notification (signal only)

```json
{
  "jsonrpc": "2.0",
  "method": "message/changed",
  "params": {
    "hasFile": true,
    "filePath": "/Users/user/messages/patient.hl7"
  }
}
```

Note: No `id` field (notification, not request).

## Example Notification (with content)

```json
{
  "jsonrpc": "2.0",
  "method": "message/changed",
  "params": {
    "message": "MSH|^~\\&|APP|FAC|||20231215120000||ADT^A01|123|P|2.5.1\rPID|1||12345",
    "format": "hl7",
    "hasFile": true,
    "filePath": "/Users/user/messages/patient.hl7"
  }
}
```

## Notes

- Only sent to extensions that subscribed to `message/changed` in capabilities
- Content included only if extension specified `includeContent: true`
- Changes from the extension itself (via `setMessage`/`patchMessage`) also
  trigger this notification
- Extensions not needing content should omit `includeContent` to reduce overhead
- If `hasFile` is false, `filePath` is omitted (untitled message)
