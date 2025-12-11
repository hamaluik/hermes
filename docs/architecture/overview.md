# System Architecture

Hermes is built as a desktop application using the Tauri framework, which
combines a Rust backend with a web-based frontend. This architecture provides
native performance for HL7 parsing and network operations, rich UI capabilities
for message editing and forms, cross-platform compatibility, and a small bundle
size compared to Electron alternatives.

## Why Tauri

HL7 message parsing is CPU-intensive, particularly for large messages with
deeply nested components. Running this work in Rust rather than JavaScript
yields substantial performance gains. The small bundle size also benefits
healthcare deployments where IT policies often scrutinise software closely. A
single-file binary with no external dependencies simplifies approval processes.

## Rust vs Webview Boundary

The architecture separates concerns by placing performance-critical and
security-sensitive operations in Rust while keeping UI rendering in the webview.

Rust handles all HL7 parsing, serialisation, and validation. These operations
involve intensive string manipulation and benefit from Rust's performance.
Network operations also run in Rust, where tokio provides a mature async runtime
for TCP connections. Schema loading happens at startup in Rust, caching HL7
definitions in memory for fast lookups.

The Svelte frontend manages all user interaction, form rendering, and state
display. UI responsiveness comes from Svelte's reactive system rather than
trying to optimise complex rendering in Rust. The webview also handles settings
persistence through Tauri's store plugin, keeping user preferences in
platform-appropriate locations.

This boundary means the frontend never parses HL7 directly. Instead, it sends
raw text to Rust for parsing, receives structured data back, and renders forms
from that structure. Changes flow back through Rust to ensure correct
serialisation.

## Feature-Based Organisation

Both frontend and backend organise code by feature rather than layer. The
`src/lib/` directory contains feature folders like `communication/`, `editor/`,
and `validation/`, each holding related components and their TypeScript bridges.
The `src-tauri/src/commands/` directory mirrors this structure, grouping Rust
command handlers by the same feature names.

This mirroring makes the codebase navigable by feature. When working on message
sending, for instance, the relevant frontend components, TypeScript bridges, and
Rust handlers all live in similarly-named directories. Adding a new feature
means creating corresponding folders in both layers rather than scattering files
across generic `components/`, `services/`, and `handlers/` directories.

## Startup Sequence

When Hermes launches, the Rust backend initialises first. It loads HL7 schemas
from embedded TOML files, building an in-memory cache that persists for the
application's lifetime. This avoids file I/O during normal operation.

After schema loading, Tauri registers plugins for clipboard access, file system
operations, dialogs, persistent storage, and logging. These plugins provide the
capabilities the frontend needs to implement user-facing features.

The native menu builds next, establishing keyboard shortcuts and menu items.
Menu item references are stored so the application can dynamically enable or
disable items based on state. The extension host initialises last, preparing to
communicate with any third-party extensions.

Once the backend is ready, the webview loads and the Svelte application renders.
The frontend connects event listeners to menu items and begins accepting user
input.

## Related Documentation

- [Frontend Architecture](frontend.md) — Component hierarchy and editing model
- [Backend Architecture](backend.md) — Managed state and command patterns
- [Communication](communication.md) — IPC patterns between frontend and backend
