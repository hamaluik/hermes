# API Reference

Quick lookup reference for the Hermes Extension API.

## API Methods

| Method              | Direction        | Type         | Purpose                       |
| ------------------- | ---------------- | ------------ | ----------------------------- |
| initialize          | Hermes→Extension | Request      | Startup handshake             |
| shutdown            | Hermes→Extension | Request      | Graceful termination          |
| command/execute     | Hermes→Extension | Notification | Execute command               |
| window/closed       | Hermes→Extension | Notification | Window closed event           |
| message/changed     | Hermes→Extension | Notification | Editor content changed        |
| message/opened      | Hermes→Extension | Notification | File opened/created           |
| message/saved       | Hermes→Extension | Notification | File saved to disk            |
| editor/getMessage   | Extension→Hermes | Request      | Retrieve current message      |
| editor/patchMessage | Extension→Hermes | Request      | Modify specific fields        |
| editor/setMessage   | Extension→Hermes | Request      | Replace entire message        |
| ui/openWindow       | Extension→Hermes | Request      | Open browser window           |
| ui/closeWindow      | Extension→Hermes | Request      | Close window                  |
| ui/showMessage      | Extension→Hermes | Request      | Display message dialogue      |
| ui/showConfirm      | Extension→Hermes | Request      | Display confirmation dialogue |
| ui/openFile         | Extension→Hermes | Request      | Single file picker            |
| ui/openFiles        | Extension→Hermes | Request      | Multiple file picker          |
| ui/saveFile         | Extension→Hermes | Request      | Save file dialogue            |
| ui/selectDirectory  | Extension→Hermes | Request      | Directory picker              |

## Reference Documents

- [protocol.md](protocol.md) - JSON-RPC 2.0 and message framing
- [types.md](types.md) - Type definitions and enumerations
- [errors.md](errors.md) - Error code catalogue
- [schema-properties.md](schema-properties.md) - Field property reference
- [api/](api/) - Individual method specifications

## API Method Details

### Lifecycle

- [initialize](api/initialize.md) - Extension startup handshake
- [shutdown](api/shutdown.md) - Graceful termination request

### Commands

- [command/execute](api/command-execute.md) - Command execution notification

### Window Events

- [window/closed](api/window-closed.md) - Window closed notification

### Message Events

- [message/changed](api/message-changed.md) - Editor content changed
- [message/opened](api/message-opened.md) - File opened or created
- [message/saved](api/message-saved.md) - File saved to disk

### Editor Operations

- [editor/getMessage](api/editor-get-message.md) - Get current message
- [editor/patchMessage](api/editor-patch-message.md) - Patch specific fields
- [editor/setMessage](api/editor-set-message.md) - Replace entire message

### UI Operations

- [ui/openWindow](api/ui-open-window.md) - Open browser window
- [ui/closeWindow](api/ui-close-window.md) - Close window
- [ui/showMessage](api/ui-show-message.md) - Message dialogue
- [ui/showConfirm](api/ui-show-confirm.md) - Confirmation dialogue
- [ui/openFile](api/ui-open-file.md) - Single file picker
- [ui/openFiles](api/ui-open-files.md) - Multiple file picker
- [ui/saveFile](api/ui-save-file.md) - Save file dialogue
- [ui/selectDirectory](api/ui-select-directory.md) - Directory picker
