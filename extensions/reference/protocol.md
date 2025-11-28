# Protocol Specification

## Transport

Communication occurs over stdio using JSON-RPC 2.0.

### Channel Assignment

| Stream | Direction          | Purpose                     |
| ------ | ------------------ | --------------------------- |
| stdin  | Hermes → Extension | Incoming requests/responses |
| stdout | Extension → Hermes | Outgoing requests/responses |
| stderr | Extension only     | Debug logging (not parsed)  |

## Message Framing

Messages use HTTP-style Content-Length headers:

```
Content-Length: <length>\r\n
\r\n
<JSON-RPC message>
```

### Format

- `Content-Length` specifies byte count of JSON content
- Header terminated by `\r\n\r\n`
- JSON message follows immediately
- No trailing newline after JSON

### Example

```
Content-Length: 52\r\n
\r\n
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

## Encoding

- All messages UTF-8 encoded
- Content-Length refers to byte count, not character count
- Non-ASCII characters encoded as UTF-8 bytes

## JSON-RPC 2.0

### Request

```typescript
interface Request {
  jsonrpc: "2.0";
  id: number | string;
  method: string;
  params?: object;
}
```

### Response (Success)

```typescript
interface SuccessResponse {
  jsonrpc: "2.0";
  id: number | string;
  result: any;
}
```

### Response (Error)

```typescript
interface ErrorResponse {
  jsonrpc: "2.0";
  id: number | string | null;
  error: {
    code: number;
    message: string;
    data?: any;
  };
}
```

### Notification

```typescript
interface Notification {
  jsonrpc: "2.0";
  method: string;
  params?: object;
}
```

Notifications omit the `id` field and expect no response.

## Bidirectional Communication

Both Hermes and extensions can initiate requests.

### Hermes-Initiated

- `initialize` - startup handshake
- `shutdown` - termination request
- `command/execute` - command trigger (notification)

### Extension-Initiated

- `editor/getMessage` - read message
- `editor/patchMessage` - modify fields
- `editor/setMessage` - replace message
- `ui/openWindow` - open browser window
- `ui/closeWindow` - close window
- `ui/showMessage` - message dialogue
- `ui/showConfirm` - confirmation dialogue
- `ui/openFile` - file picker (single)
- `ui/openFiles` - file picker (multiple)
- `ui/saveFile` - save dialogue
- `ui/selectDirectory` - directory picker

## Request IDs

- Each direction maintains independent ID sequence
- IDs must be unique within direction
- Incrementing integers recommended
- String IDs allowed

Example:

```
Hermes → Extension: 1, 2, 3, 4, ...
Extension → Hermes: 1, 2, 3, 4, ...
```

## Concurrency

### Parallel Requests

- Multiple requests may be in-flight simultaneously
- Track pending requests by ID
- Match responses to requests by ID
- Responses may arrive in any order

### Notifications During Requests

Notifications may arrive while waiting for responses. Distinguish by
presence of `result`/`error` (response) vs `method` (notification/request).

## Timeouts

| Operation  | Timeout | On Timeout                 |
| ---------- | ------- | -------------------------- |
| initialize | 10s     | Extension marked as failed |
| shutdown   | 5s      | Process killed (SIGKILL)   |

Commands (fire-and-forget notifications) have no timeout.
