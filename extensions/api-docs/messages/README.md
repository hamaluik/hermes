# Message Reference

This section documents all JSON-RPC messages in the Hermes Extension API.

## Message Categories

### Hermes → Extension

These messages are sent from Hermes to your extension:

| Method            | Document                     | Description                          |
|-------------------|------------------------------|--------------------------------------|
| `initialize`      | [initialize.md](initialize.md) | Handshake at extension startup       |
| `shutdown`        | [shutdown.md](shutdown.md)   | Request to terminate gracefully      |
| `command/execute` | [commands.md](commands.md)   | Execute a registered command         |

### Extension → Hermes

These messages are sent from your extension to Hermes:

| Method                | Document              | Description                              |
|-----------------------|-----------------------|------------------------------------------|
| `editor/getMessage`   | [editor.md](editor.md) | Get the current message                  |
| `editor/patchMessage` | [editor.md](editor.md) | Modify specific fields in the message    |
| `editor/setMessage`   | [editor.md](editor.md) | Replace the entire message               |
| `ui/openWindow`       | [ui.md](ui.md)        | Open a new window with a URL             |
| `ui/closeWindow`      | [ui.md](ui.md)        | Close a window opened by the extension   |
| `ui/showMessage`      | [ui.md](ui.md)        | Show info/warning/error message dialog   |
| `ui/showConfirm`      | [ui.md](ui.md)        | Show yes/no or ok/cancel confirmation    |
| `ui/openFile`         | [ui.md](ui.md)        | Show file open dialog (single file)      |
| `ui/openFiles`        | [ui.md](ui.md)        | Show file open dialog (multiple files)   |
| `ui/saveFile`         | [ui.md](ui.md)        | Show file save dialog                    |
| `ui/selectDirectory`  | [ui.md](ui.md)        | Show directory selection dialog          |

### Notifications (Hermes → Extension)

These notifications are sent from Hermes and do not expect a response:

| Method                | Document              | Description                              |
|-----------------------|-----------------------|------------------------------------------|
| `window/closed`       | [ui.md](ui.md)        | A window was closed                      |


## Message Flow Diagrams

### Startup Sequence

```
Hermes                                    Extension
  │                                           │
  │────── start process ─────────────────────>│
  │                                           │
  │──────── initialize ──────────────────────>│
  │                                           │
  │<─────── initialize result ────────────────│
  │         (metadata, capabilities,          │
  │          toolbar buttons, schema)         │
  │                                           │
  │         Extension is now RUNNING          │
```

### Command Execution

Commands use a fire-and-forget model:

```
Hermes                                    Extension
  │                                           │
  │──────── command/execute (notification) ──>│
  │         {command: "ext/doThing"}          │
  │                                           │
  │         Extension handles asynchronously  │
  │                                           │
  │<──────── editor/getMessage ───────────────│
  │          {format: "json"}                 │
  │                                           │
  │──────── getMessage result ───────────────>│
  │         {message: "..."}                  │
  │                                           │
  │<──────── editor/patchMessage ─────────────│
  │          {patches: [...]}                 │
  │                                           │
  │──────── patchMessage result ─────────────>│
  │         {success: true}                   │
  │                                           │
```

### Shutdown Sequence

```
Hermes                                    Extension
  │                                           │
  │──────── shutdown ────────────────────────>│
  │                                           │
  │         (extension cleans up)             │
  │                                           │
  │<─────── shutdown result ──────────────────│
  │         {success: true}                   │
  │                                           │
  │         (Hermes terminates process)       │
```

## Common Patterns

### Request Structure

All requests follow this structure:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "method/name",
  "params": { }
}
```

### Success Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { }
}
```

### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Error description",
    "data": { }
  }
}
```

## Type References

Many messages share common types. See [types.md](../types.md) for definitions of:

- `ToolbarButton` - Toolbar button configuration
- `Capabilities` - Extension capabilities
- `Patch` - Message patch operation
- `MessageFormat` - Message format enumeration
- `Schema` - Schema override structure

## Error Handling

See [errors.md](../errors.md) for:

- Standard JSON-RPC error codes
- Extension-specific error codes
- Error handling best practices

## Index

- [initialize.md](initialize.md) - Startup handshake
- [shutdown.md](shutdown.md) - Graceful termination
- [commands.md](commands.md) - Command execution (fire-and-forget notifications)
- [editor.md](editor.md) - Message reading and modification
- [ui.md](ui.md) - User interface operations
