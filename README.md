# Hermes

A desktop application for composing, sending, and receiving HL7 messages for HL7 system development and testing.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Development](#development)
- [Building](#building)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Key Technologies](#key-technologies)
- [User Documentation](#user-documentation)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)

## Overview

Hermes is a cross-platform desktop application built with Tauri and Svelte that provides a comprehensive interface for working with HL7 v2.x messages. It's designed specifically for HL7 system development and testing, offering both visual form-based editing and raw message manipulation capabilities.

The application bridges the gap between manual HL7 message creation and automated testing, providing:
- **Visual message editing** through segment tabs with field validation
- **Raw text editing** with syntax highlighting
- **MLLP protocol support** for sending and receiving messages
- **Real-time field documentation** to help developers understand HL7 structure

## Features

### Message Editing
- Dual-mode editing: form-based segment tabs and raw text editor
- Syntax highlighting for HL7 message structure
- Real-time field descriptions and validation
- Tab navigation between fields
- Copy message to clipboard

### Network Communication
- Send HL7 messages via MLLP (Minimal Lower Layer Protocol)
- Listen for incoming HL7 messages
- Configurable host, port, and timeout settings
- Response handling and logging

### File Management
- Native File menu with standard keyboard shortcuts (Cmd/Ctrl+N, O, S)
- Open and save `.hl7` message files
- Track current file for quick saves
- Create new messages with default MSH segment

### Developer Features
- Schema caching from `messages.toml` for fast lookup
- Event-driven communication between frontend and backend
- Persistent application settings via Tauri store plugin
- Comprehensive logging via Tauri log plugin

## Prerequisites

### Required
- **Node.js** (v18 or later) - JavaScript runtime
- **pnpm** (v8 or later) - Package manager
- **Rust** (latest stable) - Backend language
- **Cargo** - Rust package manager (comes with Rust)

### Platform-Specific Requirements

**macOS:**
- Xcode Command Line Tools: `xcode-select --install`

**Linux:**
- Build essentials: `sudo apt-get install build-essential libssl-dev`
- WebKit2GTK: `sudo apt-get install webkit2gtk-4.0`

**Windows:**
- Microsoft Visual Studio C++ Build Tools
- WebView2 (usually pre-installed on Windows 10/11)

## Installation

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd hermes
   ```

2. **Install frontend dependencies:**
   ```bash
   pnpm install
   ```

3. **Build dependencies** (optional, happens automatically on first run):
   ```bash
   cd src-tauri
   cargo build
   ```

## Development

### Running the Application

**Development mode** (with hot-reload):
```bash
pnpm tauri dev
```

This starts both the Vite dev server (frontend) and the Tauri application (desktop wrapper).

**Frontend only** (for UI development):
```bash
pnpm dev
```

Note: Backend commands won't work in frontend-only mode.

### Type Checking

Run TypeScript type checking:
```bash
pnpm check
```

Watch mode for continuous type checking:
```bash
pnpm check:watch
```

### Code Organization

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed architecture documentation and [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines.

## Building

### Production Build

Build the application for your platform:
```bash
pnpm tauri build
```

This creates:
- **macOS**: `.dmg` and `.app` in `src-tauri/target/release/bundle/`
- **Windows**: `.msi` and `.exe` in `src-tauri/target/release/bundle/`
- **Linux**: `.deb`, `.AppImage` in `src-tauri/target/release/bundle/`

### Build Configuration

Tauri configuration is in `src-tauri/tauri.conf.json`:
- App name and version
- Window settings
- Bundle identifiers
- Plugin configurations

## Architecture

Hermes follows a command-response pattern with event-driven communication between the frontend (Svelte) and backend (Rust/Tauri).

### High-Level Architecture

```
┌─────────────────────────────────────────┐
│           Frontend (Svelte)             │
│  ┌───────────────────────────────────┐  │
│  │  Routes (SvelteKit)               │  │
│  │  - Main page with message editor  │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │  Components                       │  │
│  │  - Segment tabs                   │  │
│  │  - Message editor                 │  │
│  │  - Modals                         │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │  Backend Bridges (TypeScript)    │  │
│  │  - Invoke Tauri commands         │  │
│  │  - Listen to backend events      │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
                    ↕ (IPC)
┌─────────────────────────────────────────┐
│          Backend (Rust/Tauri)           │
│  ┌───────────────────────────────────┐  │
│  │  Commands (exposed to frontend)  │  │
│  │  - Message operations             │  │
│  │  - MLLP send/receive             │  │
│  │  - Schema queries                 │  │
│  │  - Syntax highlighting           │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │  Schema & Spec                    │  │
│  │  - HL7 definitions cache          │  │
│  │  - Message structure lookup       │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### Communication Flow

1. **Frontend → Backend**: User actions invoke Tauri commands via `@tauri-apps/api/core`
2. **Backend Processing**: Rust functions process requests, access schema cache, perform I/O
3. **Backend → Frontend**: Results emitted via Tauri events
4. **Frontend Reactivity**: Svelte stores update UI automatically

### Key Design Decisions

- **Dual Editor Modes**: Provides both structured (forms) and unstructured (text) editing
- **Schema Caching**: Pre-loads HL7 definitions from `messages.toml` at startup for instant lookups
- **Event-Driven I/O**: Network operations (send/receive) use events for progress updates
- **Persistent Settings**: Tauri store plugin saves user preferences across sessions

For detailed architecture documentation, see [ARCHITECTURE.md](./ARCHITECTURE.md).

## Project Structure

```
hermes/
├── src/                           # Frontend (Svelte/TypeScript)
│   ├── routes/                    # SvelteKit routes
│   │   └── +page.svelte          # Main application page
│   ├── lib/                       # Feature-based organisation
│   │   ├── communication/         # MLLP send/receive
│   │   │   ├── communication_drawer.svelte
│   │   │   ├── send_tab.svelte
│   │   │   ├── listen_tab.svelte
│   │   │   ├── connection_preset.ts
│   │   │   ├── connection_presets_modal.svelte
│   │   │   ├── send_receive.ts
│   │   │   └── listen.ts
│   │   ├── editor/                # Core editor
│   │   │   ├── message_editor.svelte
│   │   │   ├── cursor_description.svelte
│   │   │   ├── history.ts
│   │   │   ├── cursor.ts
│   │   │   ├── description.ts
│   │   │   └── syntax_highlight.ts
│   │   ├── diff/                  # Message comparison
│   │   ├── find_replace/          # Search functionality
│   │   ├── forms/                 # Form inputs
│   │   ├── validation/            # Validation UI and logic
│   │   ├── settings/              # Settings UI
│   │   ├── modals/                # Standalone modals
│   │   ├── shared/                # Cross-feature utilities
│   │   ├── tabs/                  # Tab navigation
│   │   ├── toolbar/               # Toolbar components
│   │   ├── components/            # Generic UI primitives
│   │   └── icons/                 # SVG icon components
│   └── settings.ts                # Settings persistence
│
├── src-tauri/                     # Backend (Rust/Tauri)
│   ├── src/
│   │   ├── lib.rs                # App setup, plugin config
│   │   ├── menu/                 # Native menu system
│   │   ├── commands/             # Tauri commands by feature
│   │   │   ├── communication/    # send.rs, listen.rs
│   │   │   ├── editor/           # cursor.rs, data.rs, syntax_highlight.rs
│   │   │   ├── validation/       # validate.rs, diff.rs
│   │   │   └── support/          # field_description.rs, schema.rs
│   │   ├── schema/               # HL7 schema caching
│   │   └── spec/                 # HL7 specifications
│   ├── data/                     # Segment schemas (*.toml)
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
│
├── static/                        # Static assets (help.html, global.css)
├── package.json                   # Node.js dependencies
├── vite.config.ts                # Vite configuration
├── svelte.config.js              # Svelte configuration
├── tsconfig.json                 # TypeScript configuration
├── README.md                      # This file
├── ARCHITECTURE.md                # Detailed architecture docs
├── CONTRIBUTING.md                # Development guidelines
├── CLAUDE.md                      # Claude Code instructions
└── HELP.md                        # User documentation
```

## Key Technologies

### Frontend Stack
- **Svelte 5**: Reactive UI framework with runes
- **SvelteKit**: Application framework and routing
- **TypeScript**: Type-safe JavaScript
- **Vite**: Build tool and dev server

### Backend Stack
- **Tauri 2**: Desktop application framework
- **Rust**: Systems programming language
- **tokio**: Async runtime for networking
- **hl7-parser**: HL7 message parsing
- **hl7-definitions**: HL7 field definitions
- **hl7-mllp-codec**: MLLP protocol implementation

### Tauri Plugins
- `tauri-plugin-clipboard-manager`: Clipboard operations
- `tauri-plugin-fs`: File system access
- `tauri-plugin-dialog`: File dialogs
- `tauri-plugin-store`: Persistent settings
- `tauri-plugin-log`: Application logging
- `tauri-plugin-persisted-scope`: Persists file access permissions across restarts
- `tauri-plugin-opener`: URL/file opening

## User Documentation

For end-user documentation on how to use Hermes, see [HELP.md](./HELP.md).

Topics covered:
- HL7 message structure primer
- Interface walkthrough
- Message editing (tabs and raw editor)
- File operations
- Settings configuration
- Keyboard shortcuts
- Tips and tricks

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed development guidelines.

Quick overview:
- **Code Style**: Follow existing patterns, use Prettier/Rustfmt
- **Type Safety**: Leverage TypeScript and Rust's type systems
- **Testing**: Manual testing required, automated tests welcome
- **Commits**: Clear, descriptive commit messages
- **Documentation**: Update relevant docs when adding features

### Adding New Features

1. **New Tauri Command**:
   - Add command function in appropriate feature directory under `src-tauri/src/commands/`
   - Export from feature's mod.rs and commands/mod.rs
   - Register command in `src-tauri/src/lib.rs`
   - Create TypeScript bridge co-located with feature in `src/lib/<feature>/`
   - Use from Svelte components

2. **New UI Component**:
   - Create `.svelte` file in appropriate feature directory under `src/lib/`
   - Import using `$lib/` for cross-directory or relative paths within feature
   - Follow existing component patterns for consistency

## Troubleshooting

### Build Issues

**Error: `tauri` command not found**
```bash
pnpm install  # Re-install dependencies
```

**Rust compilation errors after pulling changes**
```bash
cd src-tauri
cargo clean
cargo build
```

**WebView2 missing (Windows)**
- Download and install WebView2 Runtime from Microsoft

### Development Issues

**Frontend and backend out of sync**
- Restart `pnpm tauri dev`
- Clear browser cache if running in dev mode

**Changes not appearing**
- Vite hot-reload should work automatically
- If not, restart dev server
- Check console for errors

### Runtime Issues

**Application won't start**
- Check logs in the system console
- Verify `messages.toml` exists and is valid
- Try rebuilding: `pnpm tauri build --debug`

**MLLP send/receive failing**
- Verify host and port settings
- Check firewall rules
- Ensure remote server is listening
- Review timeout settings

### Getting Help

- Review [HELP.md](./HELP.md) for user-facing features
- Check [ARCHITECTURE.md](./ARCHITECTURE.md) for technical details
- Search existing issues in the repository
- Check Tauri documentation: https://tauri.app/
- Check Svelte documentation: https://svelte.dev/

## License

See LICENSE file for details

---

**Note**: This is a developer's tool for HL7 system development and testing. It
is not intended for production use in clinical environments.
