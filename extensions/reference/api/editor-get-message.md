# editor/getMessage

Retrieve current message from editor.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field  | Type   | Required | Description   |
| ------ | ------ | -------- | ------------- |
| format | string | Yes      | Output format |

### Format Values

- `"hl7"` - Raw pipe-delimited HL7
- `"json"` - Hierarchical JSON
- `"yaml"` - YAML format
- `"toml"` - TOML format

## Response

| Field    | Type    | Required | Description                         |
| -------- | ------- | -------- | ----------------------------------- |
| message  | string  | Yes      | Message content in requested format |
| hasFile  | boolean | Yes      | Whether a file is currently open    |
| filePath | string  | No       | File path if open                   |

### Message Format

#### HL7

Pipe-delimited with `\r` segment separators:

```
MSH|^~\&|APP|FAC|||20231215120000||ADT^A01|123|P|2.5.1\rPID|1||12345||DOE^JOHN
```

Note: In JSON response, appears as `"MSH|...\rPID|..."` with escaped `\r`.

#### JSON

Hierarchical with 1-based string indices:

```json
{
  "MSH": { "1": "|", "3": "APP" },
  "PID": { "5": { "1": "DOE", "2": "JOHN" } }
}
```

#### YAML/TOML

Same structure as JSON, different serialisation format.

## Error Codes

- `-32003` No message open (message will be empty string)

## Example Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "editor/getMessage",
  "params": {
    "format": "json"
  }
}
```

## Example Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "message": "{\"MSH\":{\"1\":\"|\",\"2\":\"^~\\\\&\"},\"PID\":{\"5\":{\"1\":\"DOE\"}}}",
    "hasFile": true,
    "filePath": "/Users/user/messages/patient.hl7"
  }
}
```
