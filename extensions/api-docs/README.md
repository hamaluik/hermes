# Hermes Extension API

This documentation describes the Extension API for Hermes, a desktop application for composing, sending, and receiving HL7 messages. Extensions allow you to add custom functionality to Hermes without modifying the core application.

## Overview

Extensions are standalone programs that communicate with Hermes over **stdio** using **JSON-RPC 2.0**. When Hermes starts, it launches configured extension processes and establishes bidirectional communication:

- **Hermes → Extension:** Sends commands when toolbar buttons are clicked or menu items are triggered
- **Extension → Hermes:** Requests to read/modify the current message, open windows, etc.

This architecture is similar to the Language Server Protocol (LSP), allowing extensions to be written in any programming language.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                           Hermes                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐  ┌───────────┐  │
│  │ Toolbar │  │ Editor  │  │ Schema Cache│  │  Windows  │  │
│  └────┬────┘  └────┬────┘  └──────┬──────┘  └─────┬─────┘  │
│       │            │              │               │         │
│       └────────────┴──────────────┴───────────────┘         │
│                            │                                │
│                    ┌───────┴───────┐                        │
│                    │ Extension Host│                        │
│                    └───────┬───────┘                        │
└────────────────────────────┼────────────────────────────────┘
                             │ stdio (JSON-RPC 2.0)
              ┌──────────────┼──────────────┐
              │              │              │
         ┌────┴────┐    ┌────┴────┐    ┌────┴────┐
         │Extension│    │Extension│    │Extension│
         │    A    │    │    B    │    │    C    │
         └─────────┘    └─────────┘    └─────────┘
```

## Capabilities

Extensions can:

| Capability          | Description                                                              |
|---------------------|--------------------------------------------------------------------------|
| **Toolbar Buttons** | Add buttons to the Hermes toolbar with custom SVG icons                  |
| **Commands**        | Define commands that Hermes invokes when buttons are clicked             |
| **Read Messages**   | Get the currently open HL7 message in various formats                    |
| **Modify Messages** | Patch specific fields or replace the entire message                      |
| **Schema Overrides**| Provide custom field definitions, notes, and template values             |
| **Web UI**          | Open browser windows pointing to an HTTP server the extension hosts      |

## Quick Start

### 1. Create the Extension Executable

Your extension must be an executable that:
- Reads JSON-RPC messages from **stdin**
- Writes JSON-RPC responses to **stdout**
- Logs debug output to **stderr** (not stdout)

### 2. Handle the Initialize Handshake

When Hermes starts your extension, it sends an `initialize` request. Your extension must respond with its metadata and capabilities:

```json
// Request from Hermes
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "hermesVersion": "1.0.0",
    "apiVersion": "1.0.0"
  }
}

// Response from Extension
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "My Extension",
    "version": "1.0.0",
    "capabilities": {
      "commands": ["myExtension/doSomething"],
      "schemaProvider": false
    },
    "toolbarButtons": [
      {
        "id": "my-action",
        "label": "Do Something",
        "icon": "<svg>...</svg>",
        "command": "myExtension/doSomething"
      }
    ]
  }
}
```

### 3. Handle Commands

When a user clicks your toolbar button, Hermes sends a command notification. Commands use
a fire-and-forget model: the extension receives the notification and handles it without
sending a response.

```json
// Notification from Hermes (no id field)
{
  "jsonrpc": "2.0",
  "method": "command/execute",
  "params": {
    "command": "myExtension/doSomething"
  }
}

// Extension handles it internally, no response needed
// Extension can call editor/getMessage, editor/patchMessage, etc.
// Extension logs progress/errors to stderr
```

### 4. Configure in Hermes

Add your extension to Hermes settings:

```json
{
  "extensions": [
    {
      "path": "/path/to/your/extension",
      "args": ["--optional", "arguments"],
      "enabled": true
    }
  ]
}
```

## Documentation Structure

| Document                             | Description                                    |
|--------------------------------------|------------------------------------------------|
| [Protocol](protocol.md)              | JSON-RPC 2.0 transport specification           |
| [Lifecycle](lifecycle.md)            | Startup, handshake, runtime, and shutdown      |
| [Messages](messages/README.md)       | Complete message reference                     |
| [Types](types.md)                    | Type definitions for all data structures       |
| [Schema](schema.md)                  | Schema override format                         |
| [Errors](errors.md)                  | Error codes and handling                       |
| [Examples](examples/README.md)       | Working extension examples                     |

## Message Quick Reference

### Hermes → Extension

| Method              | Description                              |
|---------------------|------------------------------------------|
| `initialize`        | Handshake request at startup             |
| `shutdown`          | Request to terminate gracefully          |
| `command/execute`   | Execute a command (e.g., button clicked) |

### Extension → Hermes

| Method                | Description                                      |
|-----------------------|--------------------------------------------------|
| `editor/getMessage`   | Get the current message in HL7, JSON, YAML, TOML |
| `editor/patchMessage` | Modify specific fields in the message            |
| `editor/setMessage`   | Replace the entire message                       |
| `ui/openWindow`       | Open a new window with a URL                     |
| `ui/closeWindow`      | Close a window opened by the extension           |
| `ui/showMessage`      | Show info/warning/error message dialog           |
| `ui/showConfirm`      | Show yes/no or ok/cancel confirmation dialog     |
| `ui/openFile`         | Show file open dialog (single file)              |
| `ui/openFiles`        | Show file open dialog (multiple files)           |
| `ui/saveFile`         | Show file save dialog                            |
| `ui/selectDirectory`  | Show directory selection dialog                  |

### Notifications (Hermes → Extension)

| Method                | Description                                      |
|-----------------------|--------------------------------------------------|
| `window/closed`       | Notifies when a window is closed                 |


## Design Principles

1. **Language Agnostic:** Extensions can be written in any language that can read/write stdio
2. **Stateless Commands:** Each command execution is independent; extensions manage their own state
3. **Fail Safe:** Extension crashes don't affect Hermes stability
4. **Minimal Footprint:** Extensions only load what they need via capabilities

## Important Notes

**Window management:** JavaScript's `window.close()` does not work in Hermes extension
windows. Extensions must track the `windowId` returned from `ui/openWindow` and call
`ui/closeWindow` to close windows programmatically. See [ui.md](messages/ui.md) for
details.

## Current Limitations

The following features are not yet supported:

| Limitation                          | Workaround                                       |
|-------------------------------------|--------------------------------------------------|
| No menu item registration           | Use toolbar buttons instead                      |
| No keyboard shortcut registration   | Toolbar buttons only                             |
| No cursor/selection context         | Extensions don't know where user's cursor is     |
| No extension-to-extension communication | Extensions cannot discover or interact with each other |
| No automatic restart after crash    | User must restart Hermes or reload extensions    |

These limitations may be addressed in future API versions.

## Next Steps

- Read the [Protocol](protocol.md) specification to understand message framing
- Review the [Lifecycle](lifecycle.md) documentation for startup/shutdown details
- Browse [Examples](examples/README.md) for working code in various languages
