# initialize

Startup handshake sent from Hermes to extension immediately after process
starts.

## Direction

Hermes → Extension

## Type

Request (expects response)

## Timeout

10 seconds. Extension marked as failed if exceeded.

## Parameters

| Field         | Type   | Required | Description                   |
|---------------|--------|----------|-------------------------------|
| hermesVersion | string | Yes      | Hermes application version    |
| apiVersion    | string | Yes      | Extension API version         |
| dataDirectory | string | Yes      | Path to Hermes data directory |

## Response

| Field          | Type             | Required | Description                     |
|----------------|------------------|----------|---------------------------------|
| name           | string           | Yes      | Extension display name          |
| version        | string           | Yes      | Extension version (semver)      |
| description    | string           | No       | Brief description               |
| authors        | string[]         | No       | Author names                    |
| homepage       | string           | No       | URL for more information        |
| capabilities   | Capabilities     | Yes      | Extension capabilities          |
| toolbarButtons | ToolbarButton[]  | No       | Toolbar buttons to register     |
| schema         | SchemaOverride   | No       | Field definition overrides      |

### Capabilities

| Field          | Type                | Required | Description                        |
|----------------|---------------------|----------|------------------------------------|
| commands       | string[]            | No       | Command IDs this extension handles |
| schemaProvider | boolean             | No       | Whether extension provides schema  |
| events         | EventSubscription[] | No       | Events to subscribe to             |

### EventSubscription

| Field   | Type         | Required | Description                            |
|---------|--------------|----------|----------------------------------------|
| name    | EventName    | Yes      | Event name to subscribe to             |
| options | EventOptions | No       | Event-specific options                 |

Event names: `message/changed`, `message/opened`, `message/saved`

### EventOptions

Options for `message/changed` only:

| Field          | Type          | Default | Description                |
|----------------|---------------|---------|----------------------------|
| includeContent | boolean       | false   | Include message content    |
| format         | MessageFormat | "hl7"   | Format for content         |

### ToolbarButton

| Field   | Type   | Required | Description                   |
|---------|--------|----------|-------------------------------|
| id      | string | Yes      | Unique button identifier      |
| label   | string | Yes      | Tooltip text                  |
| icon    | string | Yes      | SVG markup                    |
| command | string | Yes      | Command ID to execute         |
| group   | string | No       | Visual grouping               |

Icon requirements:
- Valid SVG
- Use `viewBox` attribute
- Use `currentColor` for theming
- Optimised for 20×20 pixels

## Error Codes

- `-32600` Invalid Request
- `-32603` Internal error

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "hermesVersion": "1.0.0",
    "apiVersion": "1.0.0",
    "dataDirectory": "/Users/user/.hermes"
  }
}
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "Patient Lookup",
    "version": "1.0.0",
    "description": "Look up patient data from hospital database",
    "capabilities": {
      "commands": ["patientLookup/search"]
    },
    "toolbarButtons": [
      {
        "id": "patient-search",
        "label": "Search Patient",
        "icon": "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"><circle cx=\"11\" cy=\"11\" r=\"8\"/><path d=\"M21 21l-4.35-4.35\"/></svg>",
        "command": "patientLookup/search"
      }
    ]
  }
}
```

## Example Response (with events)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "Message Logger",
    "version": "1.0.0",
    "description": "Log all message changes to external system",
    "capabilities": {
      "commands": ["logger/configure"],
      "events": [
        { "name": "message/opened" },
        { "name": "message/saved" },
        {
          "name": "message/changed",
          "options": { "includeContent": true, "format": "json" }
        }
      ]
    }
  }
}
```
