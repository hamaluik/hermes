# Hermes Extension Documentation

Extensions allow you to add functionality to Hermes without modifying the
core application. Extensions can add toolbar buttons, read and modify HL7
messages, show dialogues and windows, and customise field schemas for
organisation-specific validation.

## Documentation Sections

This documentation follows the [Diátaxis](https://diataxis.fr/) framework,
organising content by your needs:

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│                    Learning-oriented                    │
│                   ┌────────────────┐                    │
│                   │   TUTORIALS    │                    │
│                   │                │                    │
│    Step-by-step   │ Build your     │ Goal-oriented      │
│    guides for     │ first          │ recipes for        │
│    learning       │ extension      │ specific tasks     │
│                   │                │                    │
│                   └────────────────┘                    │
│  ┌────────────────┐                ┌────────────────┐  │
│  │  EXPLANATION   │                │   HOW-TO       │  │
│  │                │                │   GUIDES       │  │
│  │ Architecture   │                │ Read messages  │  │
│  │ Lifecycle      │                │ Show dialogues │  │
│  │ Schema merging │                │ Manage windows │  │
│  │                │                │ Handle errors  │  │
│  └────────────────┘                └────────────────┘  │
│                   ┌────────────────┐                    │
│                   │   REFERENCE    │                    │
│  Understanding-   │                │ Information-       │
│  oriented         │ API methods    │ oriented           │
│  concepts         │ Protocol       │ specifications     │
│                   │ Types          │                    │
│                   │ Error codes    │                    │
│                   └────────────────┘                    │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### New to Extensions? Start Here

**[Tutorials](tutorials/)** - Learning-oriented, step-by-step guides

Build your first extension from scratch. These guides teach you the
fundamentals by walking through complete, working examples.

### Trying to Do Something Specific?

**[How-To Guides](how-to/)** - Goal-oriented recipes

Jump straight to task-focused recipes showing you how to accomplish specific
goals: reading messages, showing dialogues, managing windows, providing schema
overrides, and handling errors.

### Want to Understand How It Works?

**[Explanation](explanation/)** - Understanding-oriented concepts

Learn *why* the extension system works the way it does. These documents explain
the architecture, lifecycle states, and schema merging behaviour.

### Looking Up Specifics?

**[Reference](reference/)** - Information-oriented specifications

Complete API documentation: all methods, request/response formats, error codes,
type definitions, and schema properties.

## Quick Links

### Most Common Destinations

- **Building your first extension?** [First Extension
  Tutorial](tutorials/first-extension.md)
- **Need an API method?** [API Quick Reference](reference/README.md#api-methods)
- **Handling an error?** [Error Code Reference](reference/errors.md)
- **Customising schemas?** [Schema Properties](reference/schema-properties.md)

### Complete API Method List

| Method                                                      | Purpose                       |
|-------------------------------------------------------------|-------------------------------|
| [initialize](reference/api/initialize.md)                   | Startup handshake             |
| [shutdown](reference/api/shutdown.md)                       | Graceful termination          |
| [command/execute](reference/api/command-execute.md)         | Execute command               |
| [editor/getMessage](reference/api/editor-get-message.md)    | Retrieve current message      |
| [editor/patchMessage](reference/api/editor-patch-message.md)| Modify specific fields        |
| [editor/setMessage](reference/api/editor-set-message.md)    | Replace entire message        |
| [ui/openWindow](reference/api/ui-open-window.md)            | Open browser window           |
| [ui/closeWindow](reference/api/ui-close-window.md)          | Close window                  |
| [ui/showMessage](reference/api/ui-show-message.md)          | Display message dialogue      |
| [ui/showConfirm](reference/api/ui-show-confirm.md)          | Display confirmation dialogue |
| [ui/openFile](reference/api/ui-open-file.md)                | Single file picker            |
| [ui/openFiles](reference/api/ui-open-files.md)              | Multiple file picker          |
| [ui/saveFile](reference/api/ui-save-file.md)                | Save file dialogue            |
| [ui/selectDirectory](reference/api/ui-select-directory.md)  | Directory picker              |

## Test Extensions

The `test-extensions/` directory contains Python scripts that exercise the
extension API for testing and verification. These are used during development
to ensure the protocol implementation is correct.

## Getting Help

If you're stuck:

1. Check the [How-To Guides](how-to/) for task-specific recipes
2. Read the [Explanation](explanation/) docs to understand the system
3. Consult the [Reference](reference/) for API details
4. Look at the [test extensions](test-extensions/) for working code
