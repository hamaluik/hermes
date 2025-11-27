# ui/openWindow

Open browser window to display extension UI.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field     | Type    | Required | Default | Description                      |
|-----------|---------|----------|---------|----------------------------------|
| url       | string  | Yes      | -       | URL to load (http:// or https://)|
| title     | string  | Yes      | -       | Window title                     |
| width     | number  | No       | 800     | Width in pixels                  |
| height    | number  | No       | 600     | Height in pixels                 |
| modal     | boolean | No       | false   | Block main window interaction    |
| resizable | boolean | No       | true    | Allow window resizing            |

### URL Requirements

- Scheme must be `http://` or `https://`
- Typically `localhost` with extension-provided HTTP server

## Response

| Field    | Type   | Required | Description              |
|----------|--------|----------|--------------------------|
| windowId | string | Yes      | Unique window identifier |

## Error Codes

- `-32007` Invalid URL (malformed or invalid scheme)
- `-32008` Window error (failed to create window)

## Notes

- JavaScript `window.close()` does not work
- Use `ui/closeWindow` with returned `windowId` to close programmatically
- Extension must run HTTP server to serve content

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ui/openWindow",
  "params": {
    "url": "http://localhost:9876/wizard",
    "title": "Patient Lookup",
    "width": 600,
    "height": 400,
    "modal": true
  }
}
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "windowId": "ext-window-abc123"
  }
}
```
