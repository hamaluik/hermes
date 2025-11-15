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
│  │  Components (src/lib/)                                       │   │
│  │  • message_editor.svelte - Raw HL7 text editor               │   │
│  │  • tabs.svelte - Segment tab navigation                      │   │
│  │  • forms/segment_tab.svelte - Form-based segment editing     │   │
│  │  • wizards/*.svelte - Database wizard UIs                    │   │
│  │  • *_modal.svelte - Modals for send/receive/settings         │   │
│  │  • cursor_description.svelte - Field info display            │   │
│  │  • toolbar*.svelte - Toolbar buttons and controls            │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Backend Bridges (src/backend/)                              │   │
│  │  TypeScript modules that wrap Tauri command invocations:     │   │
│  │  • send_receive.ts - MLLP send operations                    │   │
│  │  • listen.ts - MLLP receive operations                       │   │
│  │  • data.ts - Message parsing/rendering                       │   │
│  │  • schema.ts - Schema queries                                │   │
│  │  • cursor.ts - Cursor position tracking                      │   │
│  │  • description.ts - Field descriptions                       │   │
│  │  • syntax_highlight.ts - Message highlighting                │   │
│  │  • wizards/*.ts - Wizard backend calls                       │   │
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
│  │  • Registers plugins (clipboard, fs, dialog, store, log)     │   │
│  │  • Initializes AppData managed state                         │   │
│  │  • Loads schema cache from messages.toml                     │   │
│  │  • Registers all Tauri commands                              │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Managed State (AppData)                                     │   │
│  │  • schema_cache: Arc<SchemaCache> - HL7 definitions          │   │
│  │  • listen_join: Mutex<Option<JoinHandle>> - Listener handle  │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Commands (src-tauri/src/commands/)                          │   │
│  │  Tauri-exposed functions:                                    │   │
│  │  • syntax_highlight.rs - Colorize HL7 messages               │   │
│  │  • locate_cursor.rs - Find cursor in HL7 structure           │   │
│  │  • field_description.rs - Get field metadata                 │   │
│  │  • schema.rs - Query message/segment schemas                 │   │
│  │  • data.rs - Parse/render HL7 messages                       │   │
│  │  • send_receive.rs - MLLP client (send messages)             │   │
│  │  • listen.rs - MLLP server (receive messages)                │   │
│  │  • wizards/ - Database query commands                        │   │
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
│  │  • std_spec.rs - Standard HL7 field descriptions             │   │
│  │  •  - standard specifications           │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  External Dependencies                                       │   │
│  │  • hl7-parser - Parse/serialize HL7 messages                 │   │
│  │  • hl7-definitions - Field definitions and metadata          │   │
│  │  • hl7-mllp-codec - MLLP protocol codec                      │   │
│  │  • tokio - Async runtime for networking                      │   │
│  │  • tiberius - SQL Server database client                     │   │
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
├── Tabs (Segment Navigation)
│   └── Tab (foreach segment)
├── SegmentTab (Active Segment Form)
│   ├── InputField (foreach field)
│   └── WizardButton (for MSH, PID, PV1)
├── MessageEditor (Raw HL7 Text)
│   └── Copy to Clipboard Button
├── CursorDescription (Field Info Panel)
├── MessageSendModal
├── ListenModal
└── SettingsModal
    └── DatabaseConnectionForm
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

Backend bridge modules in `src/backend/` provide type-safe wrappers around Tauri commands:

```typescript
// Example: src/backend/send_receive.ts
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
- Centralized error handling
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

## Database Integration

### Database Connection

Wizards connect to SQL Server using the `tiberius` crate:

```rust
use tiberius::{Client, Config, AuthMethod};

async fn connect_to_database(
    host: &str,
    port: u16,
    database: &str,
    username: &str,
    password: &str,
) -> Result<Client<Compat<TcpStream>>, Error> {
    let mut config = Config::new();
    config.host(host);
    config.port(port);
    config.database(database);
    config.authentication(AuthMethod::sql_server(username, password));

    let tcp = TcpStream::connect(config.get_addr()).await?;
    let client = Client::connect(config, tcp.compat()).await?;

    Ok(client)
}
```

### Wizard Queries

Each wizard executes SQL queries to fetch data:

```rust
// Example: Patient Wizard
pub async fn search_patients(
    name: Option<String>,
    id: Option<String>,
    mrn: Option<String>,
    db_config: DatabaseConfig,
) -> Result<Vec<Patient>, String> {
    let mut client = connect(&db_config).await?;

    let query = r#"
        SELECT PatientID, MRN, LastName, FirstName, DateOfBirth, Sex
        FROM Patients
        WHERE (@name IS NULL OR LastName LIKE @name)
          AND (@id IS NULL OR PatientID = @id)
          AND (@mrn IS NULL OR MRN = @mrn)
    "#;

    let rows = client
        .query(query, &[&name, &id, &mrn])
        .await?
        .into_first_result()
        .await?;

    // Map rows to Patient structs
    let patients = rows.into_iter()
        .map(|row| Patient {
            id: row.get("PatientID"),
            mrn: row.get("MRN"),
            // ...
        })
        .collect();

    Ok(patients)
}
```

### Data Population

After user selects a result, the wizard populates the HL7 segment:

```rust
pub fn populate_segment(
    segment: &mut Segment,
    patient: &Patient,
    override_segment: bool,
) {
    if override_segment {
        // Replace all fields
        segment.fields.clear();
    }

    // Set specific fields
    set_field(segment, "PID.3", &patient.mrn);
    set_field(segment, "PID.5.1", &patient.last_name);
    set_field(segment, "PID.5.2", &patient.first_name);
    // ...
}
```

## State Management

### Backend State (AppData)

```rust
pub struct AppData {
    /// Cached HL7 schemas loaded from messages.toml
    pub schema_cache: Arc<SchemaCache>,

    /// Join handle for the MLLP listener task (if running)
    pub listen_join: Mutex<Option<JoinHandle<()>>>,
}
```

- **`schema_cache`**: Shared across all commands via `Arc` for thread-safe access
- **`listen_join`**: Mutable state behind `Mutex` to start/stop listener

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

1. **Define command function** in appropriate module under `src-tauri/src/commands/`:

```rust
#[tauri::command]
pub async fn my_new_command(
    param: String,
    state: State<'_, AppData>,
) -> Result<ReturnType, String> {
    // Implementation
    Ok(result)
}
```

2. **Export from module**:

```rust
// In commands/mod.rs
pub mod my_module;
pub use my_module::my_new_command;
```

3. **Register in lib.rs**:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_new_command,
])
```

4. **Create TypeScript bridge** in `src/backend/`:

```typescript
// src/backend/my_module.ts
import { invoke } from '@tauri-apps/api/core';

export async function myNewCommand(param: string): Promise<ReturnType> {
  return await invoke('my_new_command', { param });
}
```

5. **Use from Svelte component**:

```svelte
<script lang="ts">
  import { myNewCommand } from '$backend/my_module';

  async function handleClick() {
    const result = await myNewCommand('value');
    // Handle result
  }
</script>
```

### Adding a New Wizard

1. **Backend**:
   - Create `src-tauri/src/commands/wizards/my_wizard.rs`
   - Implement search and populate functions
   - Register command in `lib.rs`

2. **Frontend**:
   - Create `src/lib/wizards/my_wizard.svelte`
   - Add search form and results table
   - Handle selection and apply logic
   - Create TypeScript bridge in `src/backend/wizards/my_wizard.ts`

3. **Integration**:
   - Add wizard button to relevant segment tab
   - Wire up modal visibility and data flow

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
