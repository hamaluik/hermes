# Error Codes

This document defines all error codes used in the Hermes Extension API.

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

## Standard JSON-RPC Errors

These are defined by the JSON-RPC 2.0 specification:

| Code   | Name              | Description                                   |
|--------|-------------------|-----------------------------------------------|
| -32700 | Parse error       | Invalid JSON received                         |
| -32600 | Invalid Request   | JSON is not a valid request object            |
| -32601 | Method not found  | Method does not exist or is not available     |
| -32602 | Invalid params    | Invalid method parameters                     |
| -32603 | Internal error    | Internal JSON-RPC error                       |

### -32700: Parse error

The message is not valid JSON:

```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32700,
    "message": "Parse error",
    "data": "Unexpected token at position 42"
  }
}
```

### -32600: Invalid Request

The JSON is valid but not a valid JSON-RPC request:

```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": "Missing required field: jsonrpc"
  }
}
```

### -32601: Method not found

The requested method doesn't exist:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -32601,
    "message": "Method not found",
    "data": "Unknown method: editor/unknownMethod"
  }
}
```

### -32602: Invalid params

Method parameters are invalid:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": "format must be one of: hl7, json, yaml, toml"
  }
}
```

### -32603: Internal error

Unexpected internal error:

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "error": {
    "code": -32603,
    "message": "Internal error",
    "data": "Failed to parse HL7 message"
  }
}
```

## Extension-Specific Errors

These codes are specific to the Hermes Extension API. They use the range -32000 to -32099 (reserved for implementation-defined server errors).

| Code   | Name                    | Description                              |
|--------|-------------------------|------------------------------------------|
| -32000 | General error           | Generic extension error                  |
| -32001 | Not initialized         | Extension not yet initialized            |
| -32002 | Already initialized     | Initialize called twice                  |
| -32003 | No message open         | Editor has no message                    |
| -32004 | Invalid message         | Message parsing failed                   |
| -32005 | Invalid path            | HL7 path syntax error                    |
| -32006 | Path not found          | HL7 path doesn't exist in message        |
| -32007 | Invalid URL             | URL format or scheme error               |
| -32008 | Window error            | Failed to open/manage window             |
| -32009 | Command not found       | Unknown command ID                       |
| -32010 | Command timeout         | Command took too long                    |
| -32011 | Validation error        | Schema validation failed                 |

### Code Ranges

| Range             | Reserved For                                        |
|-------------------|-----------------------------------------------------|
| -32000 to -32049  | Hermes (current and future use)                     |
| -32050 to -32099  | Extensions (for custom error codes)                 |

Extensions may define their own error codes in the -32050 to -32099 range. Document your custom codes for users of your extension.

### -32000: General error

Use for errors that don't fit other categories:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "General error",
    "data": "Database connection failed"
  }
}
```

### -32001: Not initialized

Request made before `initialize` completed:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32001,
    "message": "Not initialized",
    "data": "Extension must complete initialization before handling commands"
  }
}
```

### -32002: Already initialized

`initialize` called more than once:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32002,
    "message": "Already initialized",
    "data": "Extension has already been initialized"
  }
}
```

### -32003: No message open

Editor operation when no message is loaded:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -32003,
    "message": "No message open",
    "data": "Cannot patch message: editor is empty"
  }
}
```

### -32004: Invalid message

Message content is malformed:

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "error": {
    "code": -32004,
    "message": "Invalid message",
    "data": "Failed to parse HL7: MSH segment missing"
  }
}
```

### -32005: Invalid path

HL7 path syntax is wrong:

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "error": {
    "code": -32005,
    "message": "Invalid path",
    "data": "Cannot parse path 'PID.5.': unexpected end of path"
  }
}
```

### -32006: Path not found

Path is valid but doesn't exist in the message:

```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "error": {
    "code": -32006,
    "message": "Path not found",
    "data": "Segment ZZZ does not exist in message"
  }
}
```

### -32007: Invalid URL

URL is malformed or uses prohibited scheme:

```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "error": {
    "code": -32007,
    "message": "Invalid URL",
    "data": "URL scheme must be http or https"
  }
}
```

### -32008: Window error

Problem opening or managing a window:

```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "error": {
    "code": -32008,
    "message": "Window error",
    "data": "Failed to create window: system resource exhausted"
  }
}
```

### -32009: Command not found

Unknown command ID:

```json
{
  "jsonrpc": "2.0",
  "id": 11,
  "error": {
    "code": -32009,
    "message": "Command not found",
    "data": "Unknown command: myExtension/nonexistent"
  }
}
```

### -32010: Command timeout

This error code is **reserved but unused**. Commands use a fire-and-forget model (as
notifications) and have no timeout constraints. This code is reserved for potential
future use and should not be used by extensions.

### -32011: Validation error

Schema validation failed:

```json
{
  "jsonrpc": "2.0",
  "id": 13,
  "error": {
    "code": -32011,
    "message": "Validation error",
    "data": {
      "field": "PID.8",
      "value": "X",
      "allowed": ["M", "F", "O", "U"]
    }
  }
}
```

## Error Handling Best Practices

### For Extensions

#### Return appropriate codes

```python
def handle_command(command):
    if command not in known_commands:
        return error_response(-32009, "Command not found", f"Unknown: {command}")

    try:
        result = execute_command(command)
        return success_response(result)
    except DatabaseError as e:
        return error_response(-32000, "Database error", str(e))
    except ValidationError as e:
        return error_response(-32011, "Validation error", e.details)
```

#### Include helpful data

```python
# Good: helpful error data
return {
    "code": -32005,
    "message": "Invalid path",
    "data": {
        "path": "PID.5.",
        "position": 5,
        "expected": "field number or end of path"
    }
}

# Bad: unhelpful error data
return {
    "code": -32005,
    "message": "Invalid path"
}
```

#### Use specific codes

```python
# Good: specific code
if not message:
    return error_response(-32003, "No message open")

# Less good: generic code
if not message:
    return error_response(-32000, "Cannot proceed", "No message")
```

### For Error Consumers

#### Handle by code, not message

```python
# Good: switch on code
if error["code"] == -32003:
    show_dialog("Please open a message first")
elif error["code"] == -32005:
    show_dialog(f"Invalid path: {error.get('data', '')}")

# Bad: match on message text (fragile)
if "No message" in error["message"]:
    # ...
```

#### Log errors for debugging

```python
def handle_error(error):
    # always log
    log.error(f"Error {error['code']}: {error['message']}")
    if "data" in error:
        log.debug(f"Error details: {error['data']}")

    # then handle appropriately
    if error["code"] in USER_ERRORS:
        show_user_message(error)
    else:
        show_generic_error()
```

## Error Categories

### User Errors (recoverable)

User can fix these:

| Code   | Typical Resolution                    |
|--------|---------------------------------------|
| -32003 | Open a message first                  |
| -32005 | Fix the path syntax                   |
| -32006 | Use a path that exists                |
| -32011 | Use a valid value                     |

### System Errors (may need retry)

Transient issues:

| Code   | Typical Resolution                    |
|--------|---------------------------------------|
| -32000 | Retry or check configuration          |
| -32008 | Close other windows, retry            |

### Programming Errors (fix code)

Bugs in the extension:

| Code   | Typical Resolution                    |
|--------|---------------------------------------|
| -32001 | Wait for initialize before commands   |
| -32002 | Don't call initialize twice           |
| -32009 | Check command registration            |
| -32601 | Check method name spelling            |
| -32602 | Check parameter types                 |

## Related Documentation

- [Protocol](protocol.md) - JSON-RPC specification
- [Types](types.md) - Error response type definitions
- [Messages](messages/README.md) - When errors occur
