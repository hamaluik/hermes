# Error Code Catalogue

## Standard JSON-RPC Errors

| Code   | Name             | When It Occurs                  |
| ------ | ---------------- | ------------------------------- |
| -32700 | Parse error      | Invalid JSON received           |
| -32600 | Invalid Request  | JSON not a valid request object |
| -32601 | Method not found | Method does not exist           |
| -32602 | Invalid params   | Invalid method parameters       |
| -32603 | Internal error   | Internal JSON-RPC error         |

## Hermes Extension Errors

| Code   | Name                | When It Occurs                    |
| ------ | ------------------- | --------------------------------- |
| -32000 | General error       | Generic extension error           |
| -32001 | Not initialised     | Extension not yet initialised     |
| -32002 | Already initialised | Initialise called twice           |
| -32003 | No message open     | Editor has no message             |
| -32004 | Invalid message     | Message parsing failed            |
| -32005 | Invalid path        | HL7 path syntax error             |
| -32006 | Path not found      | HL7 path doesn't exist in message |
| -32007 | Invalid URL         | URL format or scheme error        |
| -32008 | Window error        | Failed to open/manage window      |
| -32009 | Command not found   | Unknown command ID                |
| -32010 | Command timeout     | Reserved, unused                  |
| -32011 | Validation error    | Schema validation failed          |
| -32012 | Dialogue error      | Failed to show system dialogue    |

## Error Code Ranges

| Range            | Reserved For                    |
| ---------------- | ------------------------------- |
| -32000 to -32049 | Hermes (current and future use) |
| -32050 to -32099 | Extensions (custom error codes) |

## Error Response Format

```typescript
interface ErrorResponse {
  jsonrpc: "2.0";
  id: number | string | null;
  error: {
    code: number;
    message: string;
    data?: unknown;
  };
}
```

Example:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": "Missing required field: method"
  }
}
```

## Method-Specific Errors

### editor/getMessage

- `-32003` No message open

### editor/patchMessage

- `-32005` Invalid path
- `-32006` Path not found

### editor/setMessage

- `-32004` Invalid message

### ui/openWindow

- `-32007` Invalid URL
- `-32008` Window error

### ui/closeWindow

- `-32008` Window error (window ID not recognised)

### ui/showMessage, ui/showConfirm, ui/openFile, ui/openFiles, ui/saveFile, ui/selectDirectory

- `-32012` Dialogue error

User cancellation is not an error. File dialogues return `null` for path
when cancelled; confirmation dialogues return `confirmed: false`.
