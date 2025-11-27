# editor/setMessage

Replace entire message in editor.

## Direction

Extension → Hermes

## Type

Request (expects response)

## Parameters

| Field   | Type   | Required | Description       |
|---------|--------|----------|-------------------|
| message | string | Yes      | Message content   |
| format  | string | Yes      | Input format      |

### Format Values

- `"hl7"` - Raw pipe-delimited HL7
- `"json"` - Hierarchical JSON
- `"yaml"` - YAML format
- `"toml"` - TOML format

## Response

| Field   | Type    | Required | Description              |
|---------|---------|----------|--------------------------|
| success | boolean | Yes      | Whether message was set  |
| error   | string  | No       | Error message if failed  |

## Error Codes

- `-32004` Invalid message (parsing failed)

## Validation

Message must:
- Be valid for specified format
- Contain MSH segment
- Parse as valid HL7 structure

## Side Effects

- Replaces editor content
- Clears undo history (operation becomes single undoable action)
- Marks document as modified
- Does not change file path

## Example Request (HL7)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "editor/setMessage",
  "params": {
    "message": "MSH|^~\\&|APP|FAC|||20231215120000||ADT^A01|123|P|2.5.1\rPID|1||12345||DOE^JOHN",
    "format": "hl7"
  }
}
```

## Example Request (JSON)

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "editor/setMessage",
  "params": {
    "message": "{\"MSH\":{\"1\":\"|\",\"2\":\"^~\\\\&\",\"3\":\"APP\"},\"PID\":{\"5\":{\"1\":\"DOE\"}}}",
    "format": "json"
  }
}
```

## Example Response (Success)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "success": true
  }
}
```

## Example Response (Error)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "success": false,
    "error": "Invalid HL7 message: MSH segment missing"
  }
}
```
