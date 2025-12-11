# Developer Documentation

This documentation covers the architecture and development practices for Hermes,
a Tauri + Svelte desktop application for composing, sending, and receiving HL7
messages.

## Architecture

Technical documentation covering system design and implementation details.

| Document                                       | Description                                  |
|------------------------------------------------|----------------------------------------------|
| [Overview](architecture/overview.md)           | System architecture and component diagram    |
| [Frontend](architecture/frontend.md)           | Svelte components, state, TypeScript bridges |
| [Backend](architecture/backend.md)             | Rust commands, app init, error handling      |
| [Communication](architecture/communication.md) | IPC patterns, events, native menus           |
| [HL7 & Schema](architecture/hl7-schema.md)     | Data models and schema system                |
| [Networking](architecture/networking.md)       | MLLP protocol, send/receive                  |

## Development

Guides for setting up, developing, and contributing to the project.

| Document                                           | Description                          |
|----------------------------------------------------|--------------------------------------|
| [Setup](development/setup.md)                      | Prerequisites and initial setup      |
| [Workflow](development/workflow.md)                | Dev mode, type checking, building    |
| [Project Structure](development/project-structure.md) | File organisation and naming      |
| [Adding Features](development/adding-features.md)  | Commands, components, HL7 types      |
| [Testing](development/testing.md)                  | Manual testing and checklists        |
| [Common Tasks](development/common-tasks.md)        | Dependencies, debugging, logs        |
| [Troubleshooting](development/troubleshooting.md)  | Common issues and solutions          |
| [Best Practices](development/best-practices.md)    | Quality, performance, security       |

### Coding Standards

Language-specific conventions and patterns.

| Document                                                   | Description                     |
|------------------------------------------------------------|---------------------------------|
| [TypeScript](development/coding-standards/typescript.md)   | TS/JS conventions               |
| [Svelte](development/coding-standards/svelte.md)           | Svelte 5 patterns               |
| [Rust](development/coding-standards/rust.md)               | Rust conventions + clippy lints |

## Related Documentation

- [Extension API](../extensions/) — Building extensions for Hermes
- [README](../README.md) — Project overview and quick start
