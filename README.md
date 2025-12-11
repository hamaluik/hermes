<h1 align="center">
  Hermes
  <br>
  <a href="https://github.com/hamaluik/hermes"><img alt="icon" width="128" height="128" src="src-tauri/icons/icon.png"></a>
</h1>
<div align="center">
    A desktop application for composing, sending, and receiving HL7 messages for
    HL7 system development and testing.
</div>
<br />
<div align="center">
  <a href="https://github.com/hamaluik/hermes/blob/main/LICENSE"><img alt="Apache 2.0 License" src="https://img.shields.io/badge/license-Apache%202.0-green?style=flat-square">
</div>

## Table of Contents

- [Overview](#overview)
- [Screenshot](#screenshot)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Development](#development)
- [Building](#building)
- [Key Technologies](#key-technologies)
- [User Documentation](#user-documentation)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)

## Overview

Hermes is a cross-platform desktop application built with Tauri and Svelte for
working with HL7 v2.x messages. It's designed for HL7 system development and
testing.

It enables you to edit both the raw HL7 message and individually extracted
fields, and to send and receive messages over MLLP. Hermes includes an extension
system that works like the Language Server Protocol, letting you add custom
functionality without recompiling the entire application.

## Screenshot

![Hermes Screenshot](./screenshot.png)

## Features

Hermes offers dual-mode message editing through form-based segment tabs and a
raw text editor with syntax highlighting for HL7 structure. Field descriptions
and validation feedback appear in real-time as you work.

Network communication happens over MLLP (Minimal Lower Layer Protocol). Hermes
can act as both a client (sending messages) and a server (listening for incoming
messages), with full response handling and logging.

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

This starts both the Vite dev server (frontend) and the Tauri application
(desktop wrapper).

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

See [docs/](./docs/) for detailed architecture and development documentation.

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

## Dependencies

### Frontend Stack

- Svelte 5
- SvelteKit
- Vite

### Backend Stack

- Tauri
- tokio
- hl7-parser
- hl7-definitions
- hl7-mllp-codec

### Tauri Plugins

Hermes uses several Tauri plugins for native functionality: `clipboard-manager`
for clipboard operations, `fs` and `dialog` for file system access and file
dialogs, `store` for persistent settings, `log` for application logging,
`persisted-scope` to preserve file access permissions across restarts, and
`opener` for opening URLs and files in external applications.

## Contributing

See [docs/development/](./docs/development/) for detailed development guidelines.

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

## License

Apache-2.0. See [LICENSE](./LICENSE) for details

---

**Note**: This is a developer's tool for HL7 system development and testing. It
is not intended for production use in clinical environments.
