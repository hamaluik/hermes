# API Reference

Complete reference for all Hermes extension API messages. For transport details, see
[protocol.md](protocol.md). For detailed explanations, see the [messages/](messages/)
directory.

## Message Index

| Method                | Direction             | Type         | Description                        |
|-----------------------|-----------------------|--------------|------------------------------------|
| [initialize]          | Hermes → Extension    | Request      | Startup handshake                  |
| [shutdown]            | Hermes → Extension    | Request      | Graceful termination               |
| [command/execute]     | Hermes → Extension    | Notification | Trigger a command                  |
| [window/closed]       | Hermes → Extension    | Notification | Window was closed                  |
| [editor/getMessage]   | Extension → Hermes    | Request      | Get current message                |
| [editor/patchMessage] | Extension → Hermes    | Request      | Modify specific fields             |
| [editor/setMessage]   | Extension → Hermes    | Request      | Replace entire message             |
| [ui/openWindow]       | Extension → Hermes    | Request      | Open browser window                |
| [ui/closeWindow]      | Extension → Hermes    | Request      | Close a window                     |
| [ui/showMessage]      | Extension → Hermes    | Request      | Show message dialog                |
| [ui/showConfirm]      | Extension → Hermes    | Request      | Show confirmation dialog           |
| [ui/openFile]         | Extension → Hermes    | Request      | File open dialog (single)          |
| [ui/openFiles]        | Extension → Hermes    | Request      | File open dialog (multiple)        |
| [ui/saveFile]         | Extension → Hermes    | Request      | File save dialog                   |
| [ui/selectDirectory]  | Extension → Hermes    | Request      | Directory selection dialog         |

[initialize]: #initialize
[shutdown]: #shutdown
[command/execute]: #commandexecute
[window/closed]: #windowclosed
[editor/getMessage]: #editorgetmessage
[editor/patchMessage]: #editorpatchmessage
[editor/setMessage]: #editorsetmessage
[ui/openWindow]: #uiopenwindow
[ui/closeWindow]: #uiclosewindow
[ui/showMessage]: #uishowmessage
[ui/showConfirm]: #uishowconfirm
[ui/openFile]: #uiopenfile
[ui/openFiles]: #uiopenfiles
[ui/saveFile]: #uisavefile
[ui/selectDirectory]: #uiselectdirectory

---

## Hermes → Extension

### initialize

Startup handshake sent immediately after the extension process starts.

**Type:** Request
**Timeout:** 10 seconds (extension marked as failed if exceeded)

**Parameters:**

| Field         | Type   | Description                     |
|---------------|--------|---------------------------------|
| hermesVersion | string | Hermes application version      |
| apiVersion    | string | Extension API version           |
| dataDirectory | string | Path to Hermes data directory   |

**Response:**

| Field          | Type             | Required | Description                        |
|----------------|------------------|----------|------------------------------------|
| name           | string           | Yes      | Extension display name             |
| version        | string           | Yes      | Extension version (semver)         |
| description    | string           | No       | Brief description                  |
| authors        | string[]         | No       | Author names                       |
| homepage       | string           | No       | URL for more information           |
| capabilities   | Capabilities     | Yes      | What the extension can do          |
| toolbarButtons | ToolbarButton[]  | No       | Buttons to add to toolbar          |
| schema         | SchemaOverride   | No       | Field definition overrides         |

**Capabilities:**

| Field          | Type     | Description                              |
|----------------|----------|------------------------------------------|
| commands       | string[] | Command IDs this extension handles       |
| schemaProvider | boolean  | Whether extension provides schema        |

**ToolbarButton:**

| Field   | Type   | Description                                       |
|---------|--------|---------------------------------------------------|
| id      | string | Unique button identifier                          |
| label   | string | Tooltip text                                      |
| icon    | string | SVG markup as string (use `currentColor` for theme support) |
| command | string | Command ID to execute when clicked                |
| group   | string | Optional visual grouping                          |

See [types.md](types.md) for `SchemaOverride` definition.

---

### shutdown

Request to terminate gracefully.

**Type:** Request
**Timeout:** 5 seconds (process killed with SIGKILL if exceeded)

**Parameters:**

| Field  | Type                                         | Description          |
|--------|----------------------------------------------|----------------------|
| reason | `"closing"` \| `"disabled"` \| `"reload"` \| `"error"` | Why shutdown was requested |

**Response:**

| Field   | Type    | Description               |
|---------|---------|---------------------------|
| success | boolean | Shutdown completed cleanly |

---

### command/execute

Trigger a command (e.g., when toolbar button clicked).

**Type:** Notification (no response expected)

**Parameters:**

| Field   | Type   | Description                  |
|---------|--------|------------------------------|
| command | string | Command ID to execute        |

**Notes:**
- Fire-and-forget: extension handles asynchronously
- Extension may call `editor/*` or `ui/*` requests during handling
- Log progress to stderr

---

### window/closed

Notification that a window opened by the extension was closed.

**Type:** Notification (no response expected)

**Parameters:**

| Field    | Type                                     | Description           |
|----------|------------------------------------------|-----------------------|
| windowId | string                                   | ID of the closed window |
| reason   | `"user"` \| `"extension"` \| `"shutdown"` | How the window closed |

---

## Extension → Hermes

### Editor Operations

#### editor/getMessage

Get the current message from the editor.

**Type:** Request

**Parameters:**

| Field  | Type                                  | Description     |
|--------|---------------------------------------|-----------------|
| format | `"hl7"` \| `"json"` \| `"yaml"` \| `"toml"` | Output format |

**Response:**

| Field    | Type    | Description                      |
|----------|---------|----------------------------------|
| message  | string  | Message content in requested format |
| hasFile  | boolean | Whether a file is currently open |
| filePath | string? | File path if open                |

**Errors:** `-32003` (No message open)

---

#### editor/patchMessage

Modify specific fields without replacing the entire message.

**Type:** Request

**Parameters:**

| Field   | Type    | Description           |
|---------|---------|-----------------------|
| patches | Patch[] | List of patches to apply |

**Patch:**

| Field  | Type    | Required | Description                           |
|--------|---------|----------|---------------------------------------|
| path   | string  | Yes      | HL7 path (e.g., `PID.5.1`, `OBX[2].3`) |
| value  | string? | No       | New value (omit to clear)             |
| remove | boolean | No       | Remove the segment                    |
| create | boolean | No       | Create segment if missing             |

**Path syntax (1-based indices):**
- `SEG.F` — Field F of segment
- `SEG.F.C` — Component C of field F
- `SEG.F.C.S` — Subcomponent S
- `SEG[N].F` — Field F of Nth occurrence of segment
- `SEG.F[N]` — Nth repetition of field F

**Response:**

| Field          | Type         | Description                   |
|----------------|--------------|-------------------------------|
| success        | boolean      | All patches applied?          |
| patchesApplied | number       | Count of successful patches   |
| errors         | PatchError[] | Details for failed patches    |

**PatchError:**

| Field   | Type   | Description           |
|---------|--------|-----------------------|
| index   | number | 0-based patch index   |
| path    | string | Path that failed      |
| message | string | Error description     |

**Errors:** `-32005` (Invalid path), `-32006` (Path not found)

**Notes:**
- Patches are applied individually in order; if one fails, subsequent patches are still
  attempted and successful patches remain in effect
- Entire operation is a single undo entry

---

#### editor/setMessage

Replace the entire message in the editor.

**Type:** Request

**Parameters:**

| Field   | Type                                  | Description     |
|---------|---------------------------------------|-----------------|
| message | string                                | Message content |
| format  | `"hl7"` \| `"json"` \| `"yaml"` \| `"toml"` | Input format  |

**Response:**

| Field   | Type    | Description                |
|---------|---------|----------------------------|
| success | boolean | Whether message was set    |
| error   | string? | Error message if failed    |

**Errors:** `-32004` (Invalid message)

**Notes:**
- MSH segment must be present
- Replaces undo history with a single entry for this operation
- Marks document as modified
- Does not change file path

---

### UI Operations

#### ui/openWindow

Open a browser window to display extension web UI.

**Type:** Request

**Parameters:**

| Field     | Type    | Default | Description                 |
|-----------|---------|---------|-----------------------------|
| url       | string  | —       | URL (http:// or https://)   |
| title     | string  | —       | Window title                |
| width     | number  | 800     | Width in pixels             |
| height    | number  | 600     | Height in pixels            |
| modal     | boolean | false   | Block interaction with main window |
| resizable | boolean | true    | Allow window resizing       |

**Response:**

| Field    | Type   | Description            |
|----------|--------|------------------------|
| windowId | string | Unique window identifier |

**Errors:** `-32007` (Invalid URL), `-32008` (Window error)

**Notes:**
- URL must use `http://` or `https://` scheme
- JavaScript `window.close()` does not work; use `ui/closeWindow`

---

#### ui/closeWindow

Close a window opened by the extension.

**Type:** Request

**Parameters:**

| Field    | Type   | Description                    |
|----------|--------|--------------------------------|
| windowId | string | Window ID from `ui/openWindow` |

**Response:**

| Field   | Type    | Description            |
|---------|---------|------------------------|
| success | boolean | Whether window was closed |

**Errors:** `-32008` (Window ID not recognised)

**Notes:**
- Closing an already-closed window returns `success: true`

---

#### ui/showMessage

Show a system-native message dialog.

**Type:** Request

**Parameters:**

| Field | Type                              | Default | Description      |
|-------|-----------------------------------|---------|------------------|
| message | string                          | —       | Message text     |
| title | string                            | —       | Dialog title     |
| kind  | `"info"` \| `"warning"` \| `"error"` | `"info"` | Dialog styling |

**Response:**

| Field        | Type    | Description                  |
|--------------|---------|------------------------------|
| acknowledged | boolean | Always `true` (user clicked OK) |

---

#### ui/showConfirm

Show a yes/no or ok/cancel confirmation dialog.

**Type:** Request

**Parameters:**

| Field   | Type                        | Default    | Description      |
|---------|-----------------------------|------------|------------------|
| message | string                      | —          | Question text    |
| title   | string                      | —          | Dialog title     |
| buttons | `"yesNo"` \| `"okCancel"`    | `"yesNo"`  | Button style     |

**Response:**

| Field     | Type    | Description                     |
|-----------|---------|---------------------------------|
| confirmed | boolean | `true` if user clicked Yes/OK   |

---

#### ui/openFile

Show file open dialog for selecting a single file.

**Type:** Request

**Parameters:**

| Field       | Type         | Description              |
|-------------|--------------|--------------------------|
| title       | string?      | Dialog title             |
| defaultPath | string?      | Starting directory       |
| filters     | FileFilter[] | File type filters        |

**FileFilter:**

| Field      | Type     | Description                          |
|------------|----------|--------------------------------------|
| name       | string   | Display name (e.g., "HL7 Files")     |
| extensions | string[] | Extensions without dots (e.g., `["hl7", "txt"]`) |

**Response:**

| Field | Type    | Description                     |
|-------|---------|---------------------------------|
| path  | string? | Selected path, `null` if cancelled |

---

#### ui/openFiles

Show file open dialog for selecting multiple files.

**Type:** Request

**Parameters:** Same as [ui/openFile](#uiopenfile)

**Response:**

| Field | Type      | Description                       |
|-------|-----------|-----------------------------------|
| paths | string[]? | Selected paths, `null` if cancelled |

---

#### ui/saveFile

Show file save dialog.

**Type:** Request

**Parameters:**

| Field       | Type         | Description          |
|-------------|--------------|----------------------|
| title       | string?      | Dialog title         |
| defaultPath | string?      | Starting directory   |
| defaultName | string?      | Default filename     |
| filters     | FileFilter[] | File type filters    |

**Response:**

| Field | Type    | Description                     |
|-------|---------|---------------------------------|
| path  | string? | Selected path, `null` if cancelled |

---

#### ui/selectDirectory

Show directory selection dialog.

**Type:** Request

**Parameters:**

| Field       | Type    | Description        |
|-------------|---------|--------------------|
| title       | string? | Dialog title       |
| defaultPath | string? | Starting directory |

**Response:**

| Field | Type    | Description                         |
|-------|---------|-------------------------------------|
| path  | string? | Selected directory, `null` if cancelled |

---

## Error Codes

### Standard JSON-RPC Errors

| Code   | Name            | Description                    |
|--------|-----------------|--------------------------------|
| -32700 | Parse error     | Invalid JSON                   |
| -32600 | Invalid Request | Not a valid request object     |
| -32601 | Method not found| Method doesn't exist           |
| -32602 | Invalid params  | Invalid method parameters      |
| -32603 | Internal error  | Internal JSON-RPC error        |

### Hermes Extension Errors

| Code   | Name               | Description                    |
|--------|--------------------|--------------------------------|
| -32000 | General error      | Generic extension error        |
| -32001 | Not initialised    | Extension not yet initialised  |
| -32002 | Already initialised| Initialise called twice        |
| -32003 | No message open    | Editor has no message          |
| -32004 | Invalid message    | Message parsing failed         |
| -32005 | Invalid path       | HL7 path syntax error          |
| -32006 | Path not found     | HL7 path doesn't exist         |
| -32007 | Invalid URL        | URL format or scheme error     |
| -32008 | Window error       | Failed to open/manage window   |
| -32009 | Command not found  | Unknown command ID             |
| -32011 | Validation error   | Schema validation failed       |
| -32012 | Dialog error       | Failed to show system dialog   |

See [errors.md](errors.md) for detailed error handling guidance.

---

## See Also

- [protocol.md](protocol.md) — Message framing and transport
- [lifecycle.md](lifecycle.md) — Startup, runtime, and shutdown flow
- [types.md](types.md) — Complete type definitions
- [schema.md](schema.md) — Schema override format and merging
- [errors.md](errors.md) — Error handling patterns
- [examples/](examples/) — Working extension examples
