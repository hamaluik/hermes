# Protocol Specification

The Hermes Extension API uses **JSON-RPC 2.0** over **stdio** for communication between Hermes and extensions.

## Transport Layer

### Channel Assignment

| Stream   | Direction          | Purpose                    |
|----------|--------------------|-----------------------------|
| `stdin`  | Hermes → Extension | Incoming requests/responses |
| `stdout` | Extension → Hermes | Outgoing requests/responses |
| `stderr` | Extension only     | Debug logging (not parsed)  |

**Important:** Never write anything to stdout except valid JSON-RPC messages. Use stderr for all logging and debug output.

### Message Framing

Messages are framed using HTTP-style headers, identical to the Language Server Protocol:

```
Content-Length: <length>\r\n
\r\n
<JSON-RPC message>
```

Where:
- `Content-Length` is the byte length of the JSON content (not character count)
- The header is followed by `\r\n\r\n`
- The JSON message follows immediately with no trailing newline

#### Example

```
Content-Length: 52\r\n
\r\n
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

### Encoding

- All messages must be UTF-8 encoded
- The `Content-Length` header refers to **byte count**, not character count
- Non-ASCII characters in strings must be properly encoded in UTF-8

## JSON-RPC 2.0

The protocol follows the [JSON-RPC 2.0 specification](https://www.jsonrpc.org/specification) with no extensions.

### Request Object

```typescript
interface Request {
  jsonrpc: "2.0";
  id: number | string;    // unique identifier for this request
  method: string;         // method name to invoke
  params?: object;        // method parameters (optional)
}
```

Example:
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

### Response Object

#### Success Response

```typescript
interface SuccessResponse {
  jsonrpc: "2.0";
  id: number | string;    // must match the request id
  result: any;            // method-specific result
}
```

Example:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "message": "{\"MSH\":{\"1\":\"|\",\"2\":\"^~\\\\&\"}}"
  }
}
```

#### Error Response

```typescript
interface ErrorResponse {
  jsonrpc: "2.0";
  id: number | string | null;  // null if request id couldn't be determined
  error: {
    code: number;              // error code (see errors.md)
    message: string;           // short description
    data?: any;                // additional error details
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

### Notification Object

Notifications are requests without an `id` field. They do not expect a response.

```typescript
interface Notification {
  jsonrpc: "2.0";
  method: string;
  params?: object;
}
```

Example:
```json
{
  "jsonrpc": "2.0",
  "method": "initialized"
}
```

**Note:** The Hermes Extension API uses notifications for events that don't require a response, such as `window/closed`. Extensions should handle these without sending a reply.

## Bidirectional Communication

Unlike traditional client-server models, **both sides can initiate requests**:

### Hermes-Initiated Requests

Hermes sends these requests to extensions:

| Method            | When                                        |
|-------------------|---------------------------------------------|
| `initialize`      | Immediately after starting the extension    |
| `shutdown`        | When Hermes is closing or disabling ext     |
| `command/execute` | When user triggers an extension command     |

### Extension-Initiated Requests

Extensions send these requests to Hermes:

| Method                | Purpose                                |
|-----------------------|----------------------------------------|
| `editor/getMessage`   | Read the current message               |
| `editor/patchMessage` | Modify specific fields                 |
| `editor/setMessage`   | Replace the entire message             |
| `ui/openWindow`       | Open a new browser window              |


## Request IDs

- Both Hermes and extensions maintain their own ID sequences
- IDs should be unique within each direction
- Use incrementing integers for simplicity
- String IDs are allowed but integers are recommended

Example ID sequences:
```
Hermes → Extension: 1, 2, 3, 4, ...
Extension → Hermes: 1, 2, 3, 4, ...
```

These sequences are independent; both sides can use ID `1` without conflict.

## Concurrency

### Parallel Requests

Both sides may send multiple requests before receiving responses. Implementations must:

1. Track pending requests by ID
2. Match responses to requests by ID
3. Handle responses in any order

### Request Ordering

- Requests may be processed in any order
- Responses may arrive in any order
- Do not assume sequential processing

### During Command Execution

Command execution uses a **fire-and-forget** model. When Hermes sends `command/execute`,
the extension handles it asynchronously without sending a response:

```
Hermes                              Extension
  │                                     │
  │──── command/execute (notification)─>│
  │                                     │
  │         Extension handles work      │
  │                                     │
  │<─── editor/getMessage (id:1) ───────│
  │                                     │
  │──── getMessage response ───────────>│
  │                                     │
```

This fire-and-forget model allows extensions to perform complex operations—including
making multiple requests to Hermes—without the overhead of acknowledgement tracking.

### Notifications During Requests

Notifications (messages without an `id`) can arrive at any time, including while the extension is waiting for a response. For example, a `window/closed` notification may arrive while the extension is waiting for an `editor/patchMessage` response:

```
Hermes                              Extension
  │                                     │
  │<─── editor/patchMessage (id:2) ─────│
  │                                     │
  │──── window/closed ─────────────────>│  notification, no id
  │                                     │
  │──── patchMessage response (id:2) ──>│
  │                                     │
```

Extensions should handle this by checking whether each incoming message is a response (has `result` or `error`) or a notification/request (has `method`). A simple approach:

```python
while waiting_for_response:
    msg = read_message()
    if "result" in msg or "error" in msg:
        # this is a response - check if it matches our pending request
        handle_response(msg)
    else:
        # this is a notification or new request - handle it
        handle_incoming(msg)
```

## Timeouts

### Hermes Timeouts

| Operation              | Timeout | Behaviour on timeout               |
|------------------------|---------|-------------------------------------|
| `initialize`           | 10s     | Extension marked as failed          |
| `shutdown`             | 5s      | Extension process killed            |

**Note:** Commands are fire-and-forget notifications and do not have timeout constraints.

### Extension Timeouts

Extensions should implement reasonable timeouts for requests to Hermes. Recommended: 5 seconds.

## Implementation Examples

### Reading Messages (Python)

```python
import sys
import json

def read_message():
    # read header
    header = ""
    while True:
        line = sys.stdin.readline()
        if line == "\r\n":
            break
        header += line

    # parse content length
    content_length = 0
    for line in header.split("\r\n"):
        if line.startswith("Content-Length:"):
            content_length = int(line.split(":")[1].strip())

    # read content
    content = sys.stdin.read(content_length)
    return json.loads(content)

def write_message(msg):
    content = json.dumps(msg)
    content_bytes = content.encode("utf-8")
    header = f"Content-Length: {len(content_bytes)}\r\n\r\n"
    sys.stdout.write(header)
    sys.stdout.write(content)
    sys.stdout.flush()
```

### Reading Messages (Node.js)

```javascript
const readline = require('readline');

class MessageReader {
  constructor() {
    this.buffer = '';
    this.contentLength = null;
  }

  feed(chunk) {
    this.buffer += chunk;
    return this.tryParse();
  }

  tryParse() {
    const messages = [];

    while (true) {
      if (this.contentLength === null) {
        const headerEnd = this.buffer.indexOf('\r\n\r\n');
        if (headerEnd === -1) break;

        const header = this.buffer.slice(0, headerEnd);
        const match = header.match(/Content-Length: (\d+)/);
        if (!match) throw new Error('Invalid header');

        this.contentLength = parseInt(match[1], 10);
        this.buffer = this.buffer.slice(headerEnd + 4);
      }

      if (Buffer.byteLength(this.buffer, 'utf8') < this.contentLength) break;

      // extract exactly contentLength bytes
      const content = Buffer.from(this.buffer, 'utf8')
        .slice(0, this.contentLength)
        .toString('utf8');

      messages.push(JSON.parse(content));
      this.buffer = this.buffer.slice(content.length);
      this.contentLength = null;
    }

    return messages;
  }
}

function writeMessage(msg) {
  const content = JSON.stringify(msg);
  const contentBytes = Buffer.byteLength(content, 'utf8');
  process.stdout.write(`Content-Length: ${contentBytes}\r\n\r\n`);
  process.stdout.write(content);
}
```

### Reading Messages (Rust)

```rust
use std::io::{self, BufRead, Write};
use serde_json::Value;

fn read_message() -> io::Result<Value> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    // read headers
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" {
            break;
        }
        if let Some(len) = line.strip_prefix("Content-Length: ") {
            content_length = len.trim().parse().unwrap();
        }
    }

    // read content
    let mut content = vec![0u8; content_length];
    reader.read_exact(&mut content)?;

    Ok(serde_json::from_slice(&content)?)
}

fn write_message(msg: &Value) -> io::Result<()> {
    let content = serde_json::to_string(msg)?;
    let content_bytes = content.as_bytes();

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    write!(writer, "Content-Length: {}\r\n\r\n", content_bytes.len())?;
    writer.write_all(content_bytes)?;
    writer.flush()
}
```

## Related Documentation

- [Lifecycle](lifecycle.md) - Extension startup and shutdown flow
- [Messages](messages/README.md) - Complete message reference
- [Errors](errors.md) - Error codes and handling
