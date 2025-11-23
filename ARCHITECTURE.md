# Hermes Architecture Documentation

This document provides detailed technical documentation of the Hermes application architecture, including design decisions, data flows, and implementation details.

## Table of Contents

- [System Architecture](#system-architecture)
- [Frontend Architecture](#frontend-architecture)
- [Backend Architecture](#backend-architecture)
- [Communication Patterns](#communication-patterns)
- [Data Models](#data-models)
- [HL7 Schema System](#hl7-schema-system)
- [Network Communication](#network-communication)
- [Database Integration](#database-integration)
- [State Management](#state-management)
- [Extension Points](#extension-points)

## System Architecture

Hermes is built as a desktop application using the Tauri framework, which combines a Rust backend with a web-based frontend. This architecture provides:

- **Native performance** for HL7 parsing and network operations (Rust)
- **Rich UI capabilities** for message editing and forms (Svelte)
- **Cross-platform compatibility** (macOS, Windows, Linux)
- **Small bundle size** compared to Electron alternatives

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                    FRONTEND (Svelte + TypeScript)                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Routes (SvelteKit)                                          │   │
│  │  • +page.svelte - Main application with message editor       │   │
│  │  • +layout.ts - App layout configuration                     │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Feature Modules (src/lib/)                                   │   │
│  │  Each feature folder contains components and Tauri bridges:   │   │
│  │  • communication/ - Drawer, send/listen tabs, presets         │   │
│  │  • editor/ - Message editor, cursor tracking, history         │   │
│  │  • diff/ - Message comparison                                 │   │
│  │  • find_replace/ - Search functionality                       │   │
│  │  • validation/ - Validation panel and logic                   │   │
│  │  • forms/ - Form inputs (input_field, toggle_switch)          │   │
│  │  • modals/ - Standalone modals (jump_to_field, timestamp)     │   │
│  │  • settings/ - Settings modal, theme toggle                   │   │
│  │  • tabs/ - Segment tab navigation                             │   │
│  │  • toolbar/ - Toolbar buttons and controls                    │   │
│  │  • components/ - Generic UI primitives (modal, modal_header)  │   │
│  │  • icons/ - SVG icon components                               │   │
│  │  • shared/ - Cross-feature utilities (schema.ts, data.ts)     │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Settings (src/settings.ts)                                  │   │
│  │  • Wraps Tauri store plugin for persistent settings          │   │
│  │  • Provides typed accessors for configuration values         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            │ IPC (Inter-Process Communication)
                            │ • invoke() for commands
                            │ • listen() for events
                            │
┌───────────────────────────▼─────────────────────────────────────────┐
│                      BACKEND (Rust + Tauri)                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Application Entry (src-tauri/src/lib.rs)                    │   │
│  │  • Sets up Tauri application                                 │   │
│  │  • Registers plugins (clipboard, fs, dialog, store, log,     │   │
│  │    persisted-scope, opener)                                  │   │
│  │  • Builds native File menu with keyboard shortcuts           │   │
│  │  • Initializes AppData managed state                         │   │
│  │  • Loads schema cache from messages.toml                     │   │
│  │  • Registers all Tauri commands                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Managed State (AppData)                                     │   │
│  │  • schema: SchemaCache - HL7 definitions                     │   │
│  │  • listen_join: Mutex<Option<JoinHandle>> - Listener handle  │   │
│  │  • save_menu_item: MenuItem - For dynamic enable/disable     │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Commands (src-tauri/src/commands/)                          │   │
│  │  Tauri-exposed functions organised by feature:               │   │
│  │  • communication/ - send.rs, listen.rs (MLLP operations)     │   │
│  │  • editor/ - cursor.rs, data.rs, syntax_highlight.rs         │   │
│  │  • validation/ - validate.rs, diff.rs                        │   │
│  │  • support/ - field_description.rs, schema.rs                │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Menu (src-tauri/src/menu/)                                  │   │
│  │  • mod.rs - Menu building and event routing                  │   │
│  │  • state.rs - Dynamic menu state (enable/disable items)      │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Schema Module (src-tauri/src/schema/)                       │   │
│  │  • cache.rs - SchemaCache with message/segment lookup        │   │
│  │  • message.rs - Message metadata                             │   │
│  │  • segment.rs - Segment metadata                             │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Spec Module (src-tauri/src/spec/)                           │   │
│  │  • std_spec.rs - Standard HL7 v2.5.1 field descriptions      │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  External Dependencies                                       │   │
│  │  • hl7-parser - Parse/serialize HL7 messages                 │   │
│  │  • hl7-definitions - Field definitions and metadata          │   │
│  │  • hl7-mllp-codec - MLLP protocol codec                      │   │
│  │  • tokio - Async runtime for networking                      │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Frontend Architecture

The frontend is built using Svelte 5 with SvelteKit, leveraging modern reactive programming patterns.

### Component Hierarchy

```
+page.svelte (Main Application)
├── Toolbar
│   ├── ToolbarButton (New, Open, Save, etc.)
│   ├── ToolbarSeparator
│   └── ToolbarSpacer
├── FindReplaceBar (Search/Replace)
├── Tabs (Segment Navigation)
│   └── Tab (foreach segment)
├── SegmentTab (Active Segment Form)
│   └── InputField (foreach field)
├── MessageEditor (Raw HL7 Text)
│   └── Copy to Clipboard Button
├── ValidationPanel (Validation results)
├── CursorDescription (Field Info Panel)
├── CommunicationDrawer
│   ├── SendTab (MLLP client)
│   └── ListenTab (MLLP server)
├── ConnectionPresetsModal (Preset management)
├── DiffModal (Message comparison)
├── JumpToFieldModal
├── InsertTimestampModal
└── SettingsModal
```

### State Management

Hermes uses Svelte's built-in reactivity system (runes in Svelte 5) for state management:

#### Local Component State
- Each component manages its own state using `$state()` rune
- Example: `let messageText = $state('')`

#### Settings State
- Managed via `src/settings.ts` wrapper around Tauri store plugin
- Provides typed getters/setters for configuration
- Automatically persists to disk on changes with debouncing
- Settings include:
  - Editor preferences (tabs follow cursor)
  - Database connection (host, port, database, credentials)
  - Network settings (host, port, timeouts)

#### Derived State
- Computed values use `$derived()` rune
- Example: Parsed message structure derived from raw text

### TypeScript Bridges

TypeScript bridges for Tauri commands are co-located with their components in feature directories under `src/lib/`. This keeps related code together:

```typescript
// Example: src/lib/communication/send_receive.ts
import { invoke } from '@tauri-apps/api/core';

export async function sendMessage(
  host: string,
  port: number,
  message: string,
  timeout: number
): Promise<void> {
  await invoke('send_message', { host, port, message, timeout });
}
```

Key benefits:
- Type safety for command parameters and return values
- Feature cohesion: UI and backend bridges in same directory
- Easier testing and mocking
- Documentation via TypeScript types

## Backend Architecture

The Rust backend provides high-performance HL7 processing and network operations.

### Application Initialization

```rust
// src-tauri/src/lib.rs - Simplified

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load HL7 schema from messages.toml
    let schema_cache = Arc::new(SchemaCache::from_file("messages.toml")?);

    // Initialize managed state
    let app_data = AppData {
        schema_cache: schema_cache.clone(),
        listen_join: Mutex::new(None),
    };

    // Build and run Tauri app
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(app_data)
        .invoke_handler(tauri::generate_handler![
            // ... all command functions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Command Structure

All Tauri commands follow this pattern:

```rust
#[tauri::command]
pub async fn command_name(
    param1: String,
    param2: i32,
    state: State<'_, AppData>,  // Optional: access managed state
    window: Window,              // Optional: emit events
) -> Result<ReturnType, String> {
    // 1. Extract needed state
    let schema_cache = &state.schema_cache;

    // 2. Perform operation
    let result = do_something(param1, param2, schema_cache)?;

    // 3. Optionally emit events
    window.emit("event-name", &result)?;

    // 4. Return result
    Ok(result)
}
```

### Error Handling

- Commands return `Result<T, String>` to propagate errors to frontend
- Internal errors use `color-eyre` for detailed error context
- Frontend displays errors in modals or notifications

## Communication Patterns

### Command-Response Pattern

**Synchronous Operations** (immediate response):

1. Frontend invokes command: `invoke('command_name', { params })`
2. Backend processes request
3. Backend returns result or error
4. Frontend handles response

Example: Getting field description

```typescript
// Frontend
const description = await getFieldDescription(path);

// Backend
#[tauri::command]
pub fn get_field_description(path: String) -> Result<String, String> {
    // ... lookup description
    Ok(description)
}
```

### Event-Driven Pattern

**Asynchronous Operations** (progress updates):

1. Frontend sets up event listeners
2. Frontend invokes command
3. Backend emits events with progress/results
4. Frontend updates UI reactively

Example: Sending HL7 message

```typescript
// Frontend
await listen('send-log', (event) => {
  console.log(event.payload);  // Progress messages
});

await listen('send-response', (event) => {
  const response = event.payload;  // Final result
});

await sendMessage(host, port, message, timeout);

// Backend
#[tauri::command]
pub async fn send_message(
    host: String,
    port: u16,
    message: String,
    timeout: u64,
    window: Window,
) -> Result<(), String> {
    window.emit("send-log", "Connecting...")?;

    // ... connect and send

    window.emit("send-response", response)?;
    Ok(())
}
```

### Long-Running Tasks

For operations that run indefinitely (like MLLP listener), Tauri spawns tokio tasks:

```rust
#[tauri::command]
pub async fn start_listening(
    port: u16,
    state: State<'_, AppData>,
    window: Window,
) -> Result<(), String> {
    // Spawn async task
    let handle = tokio::spawn(async move {
        // Run MLLP server
        listen_for_messages(port, window).await;
    });

    // Store handle to allow cancellation
    *state.listen_join.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
pub async fn stop_listening(
    state: State<'_, AppData>,
) -> Result<(), String> {
    if let Some(handle) = state.listen_join.lock().unwrap().take() {
        handle.abort();
    }
    Ok(())
}
```

### Native Menu Events

The application provides a native File menu with standard keyboard shortcuts. Menu item clicks
trigger events that the frontend listens to, ensuring consistent behaviour between menu and toolbar.

**Menu Structure:**
- File → New (Cmd/Ctrl+N)
- File → Open... (Cmd/Ctrl+O)
- File → Save (Cmd/Ctrl+S) - dynamically enabled/disabled
- File → Save As... (Cmd/Ctrl+Shift+S)

**Event Flow:**

1. User clicks menu item or uses keyboard shortcut
2. Tauri's `on_menu_event` handler emits corresponding event
3. Frontend listens to events and triggers file operations

```rust
// Backend - Menu event handling (lib.rs)
app.on_menu_event(move |app_handle, event| {
    let event_name = match event.id().as_ref() {
        "file-new" => Some("menu-file-new"),
        "file-open" => Some("menu-file-open"),
        "file-save" => Some("menu-file-save"),
        "file-save-as" => Some("menu-file-save-as"),
        _ => None,
    };

    if let Some(name) = event_name {
        let _ = app_handle.emit(name, ());
    }
});
```

```typescript
// Frontend - Menu event listeners (+page.svelte)
listen("menu-file-new", () => handleNew());
listen("menu-file-open", () => handleOpenFile());
listen("menu-file-save", () => handleSave?.());
listen("menu-file-save-as", () => handleSaveAs());
```

**Dynamic Menu State:**

The Save menu item's enabled state is synced with the toolbar save button using a dedicated
command. This ensures users can't invoke Save from the menu when there are no unsaved changes.

```rust
// Backend - menu.rs
#[tauri::command]
pub fn set_save_enabled(enabled: bool, state: State<'_, AppData>) -> Result<(), String> {
    state.save_menu_item.set_enabled(enabled)
        .map_err(|e| format!("Failed to set save menu enabled state: {e}"))
}
```

```typescript
// Frontend - Reactive effect syncs menu state with button state
$effect(() => {
    invoke("set_save_enabled", { enabled: handleSave !== undefined });
});
```

## Data Models

### HL7 Message Structure

```
Message
├── MSH Segment (required, first)
├── Additional Segments (0..n)
│   ├── Segment
│   │   ├── Field 1
│   │   ├── Field 2
│   │   │   ├── Component 1
│   │   │   ├── Component 2 (separated by ^)
│   │   │   └── ...
│   │   ├── Field 3 (can have repetitions ~)
│   │   └── ...
│   └── ...
└── ...
```

The `hl7-parser` crate parses messages into this structure:

```rust
// From hl7-parser crate
pub struct Message {
    pub separators: Separators,
    pub segments: Vec<Segment>,
}

pub struct Segment {
    pub id: String,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub components: Vec<Component>,
    pub repetitions: Vec<Repetition>,
}
```

### Schema Models

```rust
// src-tauri/src/schema/message.rs
pub struct MessageSchema {
    pub id: String,
    pub name: String,
    pub trigger_events: Vec<String>,
    pub segments: Vec<SegmentRef>,
}

// src-tauri/src/schema/segment.rs
pub struct SegmentSchema {
    pub id: String,
    pub name: String,
    pub fields: Vec<FieldSchema>,
}

pub struct FieldSchema {
    pub id: String,
    pub name: String,
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub data_type: String,
}
```

## HL7 Schema System

The schema system provides metadata about HL7 messages for validation, editing assistance, and documentation.

### Schema Loading

1. **Startup**: `lib.rs` loads `messages.toml` into `SchemaCache`
2. **Caching**: Schema cached in memory for fast lookups
3. **Lookup**: Commands query schema cache for message/segment info

### Schema File Format

`messages.toml` defines message structures:

```toml
[[message]]
id = "ADT_A01"
name = "Admit/Visit Notification"
trigger_events = ["A01"]

[[message.segment]]
id = "MSH"
required = true
repeating = false

[[message.segment]]
id = "PID"
required = true
repeating = false

[[message.segment]]
id = "PV1"
required = true
repeating = false
```

### Field Descriptions

The `spec` module combines:
- **Standard HL7 definitions** from `hl7-definitions` crate
- **HL7 system customizations** in ``

Priority: HL7 system specs override standard specs.

## Network Communication

### MLLP Protocol

MLLP (Minimal Lower Layer Protocol) is the standard transport for HL7 v2.x messages over TCP.

**Message Framing:**
```
<VT> message content <FS><CR>
```
- `<VT>` (0x0B): Start of message
- `<FS>` (0x1C): End of message
- `<CR>` (0x0D): Carriage return

### Sending Messages

```rust
// src-tauri/src/commands/send_receive.rs
use hl7_mllp_codec::MllpCodec;
use tokio_util::codec::Framed;

pub async fn send_message(
    host: String,
    port: u16,
    message: String,
    timeout: u64,
    window: Window,
) -> Result<(), String> {
    // 1. Connect to server
    let stream = timeout(
        Duration::from_millis(timeout),
        TcpStream::connect((host, port))
    ).await??;

    // 2. Wrap in MLLP codec
    let mut framed = Framed::new(stream, MllpCodec::new());

    // 3. Send message
    framed.send(message.into()).await?;

    // 4. Receive response
    let response = framed.next().await.unwrap()?;

    window.emit("send-response", response)?;
    Ok(())
}
```

### Listening for Messages

```rust
// src-tauri/src/commands/listen.rs
pub async fn listen_for_messages(
    port: u16,
    window: Window,
) -> Result<(), String> {
    // 1. Bind listener
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;

    loop {
        // 2. Accept connection
        let (stream, addr) = listener.accept().await?;

        // 3. Spawn handler for this connection
        tokio::spawn(handle_connection(stream, addr, window.clone()));
    }
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, window: Window) {
    let mut framed = Framed::new(stream, MllpCodec::new());

    while let Some(Ok(message)) = framed.next().await {
        // Emit received message to frontend
        window.emit("received-message", message).unwrap();

        // Send ACK
        let ack = generate_ack(&message);
        framed.send(ack.into()).await.unwrap();
    }
}
```

## State Management

### Backend State (AppData)

```rust
pub struct AppData {
    /// Cached HL7 schemas loaded from messages.toml
    pub schema: SchemaCache,

    /// Join handle for the MLLP listener task (if running)
    pub listen_join: Mutex<Option<JoinHandle<()>>>,

    /// Reference to the Save menu item for dynamic enable/disable
    pub save_menu_item: MenuItem<Wry>,
}
```

- **`schema`**: Cached HL7 definitions for fast lookups during message editing
- **`listen_join`**: Mutable state behind `Mutex` to start/stop listener
- **`save_menu_item`**: Reference to the native Save menu item, allowing the frontend to
  dynamically enable/disable it based on whether there are unsaved changes

### Frontend State

Settings are persisted using Tauri's store plugin:

```typescript
// src/settings.ts
import { Store } from '@tauri-apps/plugin-store';

const store = new Store('settings.json');

export async function getSetting<T>(key: string, defaultValue: T): Promise<T> {
  const value = await store.get(key);
  return value !== null ? (value as T) : defaultValue;
}

export async function setSetting(key: string, value: any): Promise<void> {
  await store.set(key, value);
  await store.save();  // Persist to disk
}
```

## Extension Points

### Adding a New Tauri Command

1. **Define command function** in the appropriate feature directory under `src-tauri/src/commands/`:

```rust
// src-tauri/src/commands/my_feature/my_command.rs
#[tauri::command]
pub async fn my_new_command(
    param: String,
    state: State<'_, AppData>,
) -> Result<ReturnType, String> {
    // Implementation
    Ok(result)
}
```

2. **Export from feature module**:

```rust
// In commands/my_feature/mod.rs
mod my_command;
pub use my_command::my_new_command;

// In commands/mod.rs
pub mod my_feature;
pub use my_feature::my_new_command;
```

3. **Register in lib.rs**:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_new_command,
])
```

4. **Create TypeScript bridge** co-located with the feature in `src/lib/`:

```typescript
// src/lib/my_feature/my_command.ts
import { invoke } from '@tauri-apps/api/core';

export async function myNewCommand(param: string): Promise<ReturnType> {
  return await invoke('my_new_command', { param });
}
```

5. **Use from Svelte component**:

```svelte
<script lang="ts">
  import { myNewCommand } from './my_command';
  // Or use $lib for cross-directory imports:
  // import { myNewCommand } from '$lib/my_feature/my_command';

  async function handleClick() {
    const result = await myNewCommand('value');
    // Handle result
  }
</script>
```

### Adding Support for New HL7 Versions

1. Update `hl7-definitions` dependency to include new version
2. Add new message definitions to `messages.toml`
3. Update schema cache to handle version-specific fields
4. Add version-specific specs to `spec` module if needed

---

## Summary

Hermes is designed with clean separation between frontend (UI/UX) and backend (processing/I/O), connected via Tauri's command and event system. Key architectural principles:

- **Type Safety**: TypeScript frontend + Rust backend = compile-time safety
- **Performance**: Rust for heavy lifting (parsing, networking)
- **Reactivity**: Svelte for responsive UI
- **Extensibility**: Modular command structure for easy feature additions
- **Maintainability**: Clear separation of concerns and well-defined interfaces

For specific implementation examples, refer to the source code with the file locations documented above.
