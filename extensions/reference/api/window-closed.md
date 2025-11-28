# window/closed

Notification sent when a window opened by the extension is closed.

## Direction

Hermes → Extension

## Type

Notification (no response expected)

## Timeout

None (notification)

## Parameters

| Field    | Type   | Required | Description                      |
| -------- | ------ | -------- | -------------------------------- |
| windowId | string | Yes      | ID of the window that was closed |

## Response

None. This is a notification; extensions must not send a response.

## Example Notification

```json
{
  "jsonrpc": "2.0",
  "method": "window/closed",
  "params": {
    "windowId": "ext-patient-wizard-1"
  }
}
```

Note: No `id` field (notification, not request).

## Notes

- Sent when user closes the window via the title bar close button
- Sent when extension closes the window via `ui/closeWindow`
- Extensions should clean up any state associated with the window
- The `windowId` matches the ID returned from `ui/openWindow`
