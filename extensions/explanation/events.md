# Events

This document explains the event notification system in the Hermes Extension API,
including why events exist, how subscription works, debouncing behaviour, and the
design decisions behind content delivery options.

## Why Events Exist

Extensions can already query Hermes for the current message using
`editor/getMessage`. Why add a separate event system?

### The Polling Problem

Without events, an extension that needs to react to message changes has two
options:

**Option 1: Poll on a timer**
```python
while running:
    message = hermes.get_message()
    if message != last_message:
        process_change(message)
        last_message = message
    time.sleep(0.5)
```

**Problems:**
- Wastes resources polling when nothing changed
- 500ms delay between change and detection
- Faster polling increases overhead; slower polling increases latency
- Every interested extension polls independently, multiplying overhead

**Option 2: React only on command execution**
```python
def handle_command(command):
    message = hermes.get_message()
    process_message(message)
```

**Problems:**
- Extension only sees message state when user explicitly triggers it
- Can't provide real-time feedback, live previews, or background analysis
- User must remember to click the button after making changes

### The Push Solution

Events flip the model: instead of extensions asking "has anything changed?",
Hermes tells extensions "something changed."

```python
def handle_message_changed(params):
    # Hermes called us - we know there's a change
    message = params.get("message") or hermes.get_message()
    process_change(message)
```

**Benefits:**
- Zero overhead when nothing changes
- Instant notification when changes occur
- Single broadcast from Hermes, not N polls from N extensions
- Extensions can implement reactive patterns naturally

### Comparison to Commands

Both events and commands use JSON-RPC notifications (no response expected), but
they serve different purposes:

| Aspect          | Commands                  | Events                      |
|-----------------|---------------------------|-----------------------------|
| Trigger         | User action (button click)| Hermes state change         |
| Direction       | Hermes asks extension     | Hermes informs extension    |
| Timing          | On demand                 | When state changes          |
| Use case        | "Do this task"            | "This happened"             |

Commands are imperative ("search for patient"). Events are informational
("message was saved"). Extensions often combine both: events trigger analysis,
commands present results.

## The Subscription Model

Extensions declare which events they want during initialisation. This
subscription model has several design implications.

### Why Subscribe at Initialise Time?

Extensions specify event subscriptions in their `initialize` response:

```json
{
  "capabilities": {
    "events": [
      { "name": "message/changed" },
      { "name": "message/saved" }
    ]
  }
}
```

**Why not runtime subscription?** Alternatives considered:

**Runtime subscribe/unsubscribe requests:**
```json
{"method": "events/subscribe", "params": {"event": "message/changed"}}
{"method": "events/unsubscribe", "params": {"event": "message/changed"}}
```

This adds protocol complexity:
- Two new request types to implement and document
- State tracking: which extensions subscribe to what
- Race conditions: what if events fire between subscribe and acknowledgement?
- Lifecycle questions: do subscriptions survive extension restarts?

**Initialise-time declaration** is simpler:
- Single source of truth: the initialize response
- No state to track beyond what's already in extension metadata
- Subscription lifetime equals extension lifetime
- No race conditions: events only start after initialisation completes

**Trade-off:** Extensions can't dynamically change subscriptions without
restarting. In practice, this isn't limiting—extensions know at startup which
events they need.

### Opt-In Efficiency

Hermes only sends events to extensions that subscribed to them:

```
message/changed occurs
    ↓
Hermes checks subscription list
    ↓
Extension A subscribed? → Yes → Send notification
Extension B subscribed? → No  → Skip
Extension C subscribed? → Yes → Send notification
```

**Why opt-in rather than broadcast?**

Broadcasting all events to all extensions would:
- Waste I/O sending events extensions don't handle
- Require extensions to filter and ignore unwanted events
- Couple extensions to events they don't care about

Opt-in means:
- Extensions only receive relevant events
- Hermes can skip serialisation/transmission for uninterested extensions
- Adding new event types doesn't affect existing extensions

### Event Types Available

Three events are currently defined:

| Event             | When Sent                                   |
|-------------------|---------------------------------------------|
| `message/changed` | Editor content changes (debounced)          |
| `message/opened`  | File opened or new message created          |
| `message/saved`   | File saved to disk                          |

Each event has specific parameters documented in the
[reference](../reference/api/).

## Debouncing

The `message/changed` event uses a 500ms debounce window. This is a deliberate
design choice with specific trade-offs.

### The Typing Problem

When a user types in the editor, every keystroke changes the message:

```
User types "DOE"
    ↓
'd' pressed → message changed
'o' pressed → message changed
'e' pressed → message changed
```

Without debouncing, three keystrokes generate three events. Fast typing
generates events faster than extensions can process them, causing:

- Event queue backup
- Redundant processing (intermediate states are quickly obsolete)
- Resource waste (CPU, I/O, network if extensions sync externally)

### How Debouncing Works

Hermes waits for typing to pause before sending `message/changed`:

```
Time: 0ms    100ms   200ms   700ms   1200ms
      │       │       │       │       │
      d       o       e       ·       │
      │       │       │       │       │
      └───────┴───────┴───────┘       │
              500ms window            │
                     └────────────────┘
                     Event sent here (700ms)
```

**Algorithm:**
1. Change occurs at time T
2. Start a 500ms timer
3. If another change occurs before timer expires, restart the timer
4. When timer expires, send `message/changed` with current state

This coalesces rapid changes into a single event containing the final state.

### Why 500ms?

The debounce window balances responsiveness against efficiency:

**Too short (e.g., 100ms):**
- Events fire during typing bursts
- Still generates many events for moderate typing speed
- Minimal coalescing benefit

**Too long (e.g., 2000ms):**
- Noticeable delay between typing and event
- Live preview feels sluggish
- Poor user experience for real-time features

**500ms is a sweet spot:**
- Average typing pause between words/thoughts
- Long enough to coalesce burst typing
- Short enough for responsive feel
- Matches Hermes's auto-save debounce (consistency)

### Trade-offs

**Benefits:**
- Prevents event flooding during typing
- Extensions see stable states, not intermediate ones
- Reduces I/O and processing overhead

**Costs:**
- 500ms latency between change and notification
- Extensions can't see every intermediate state
- Fast automated edits might coalesce unexpectedly

For most use cases (analysis, sync, preview), the latency is acceptable and the
efficiency gain is significant. Extensions needing keystroke-level granularity
should poll `editor/getMessage` instead.

### Other Events Not Debounced

`message/opened` and `message/saved` don't need debouncing:

- **message/opened:** Discrete user action (File > Open, drag-and-drop). Can't
  occur rapidly in succession.
- **message/saved:** Discrete save action. Auto-save is already debounced (500ms
  after last edit), so save events are naturally rate-limited.

## Content Delivery Options

The `message/changed` event offers a choice: receive just the notification, or
receive the message content with it.

### Signal-Only Mode (Default)

By default, `message/changed` includes only metadata:

```json
{
  "method": "message/changed",
  "params": {
    "hasFile": true,
    "filePath": "/Users/user/messages/patient.hl7"
  }
}
```

The extension knows a change occurred but must call `editor/getMessage` to get
the content:

```python
def handle_message_changed(params):
    response = hermes.request("editor/getMessage", {"format": "hl7"})
    message = response["message"]
    process_change(message)
```

**When signal-only is appropriate:**
- Extension only cares about certain changes (checks before fetching)
- Extension needs the message in a specific format
- Extension might skip processing based on file path or other metadata

### Content-Included Mode

Extensions can request content delivery by specifying options:

```json
{
  "capabilities": {
    "events": [
      {
        "name": "message/changed",
        "options": {
          "includeContent": true,
          "format": "json"
        }
      }
    ]
  }
}
```

The event then includes the message:

```json
{
  "method": "message/changed",
  "params": {
    "message": "{\"MSH\":{\"1\":\"|\",\"2\":\"^~\\\\&\"...}}",
    "format": "json",
    "hasFile": true,
    "filePath": "/Users/user/messages/patient.hl7"
  }
}
```

**When content-included is appropriate:**
- Extension always processes every change
- Extension wants a specific format (JSON, YAML, TOML)
- Avoiding a round-trip improves latency

### Why Opt-In?

Content delivery defaults to off for several reasons:

**Message size:** HL7 messages can be large (thousands of characters for complex
orders or results). Including content in every event notification adds I/O
overhead.

**Format flexibility:** Different extensions want different formats. An
extension might want JSON for one operation and raw HL7 for another. Letting
extensions call `getMessage` with their desired format is more flexible.

**Backward compatibility:** Adding content to events by default would break
extensions that assume minimal payloads. Opt-in ensures existing extensions
continue working.

**Explicit intent:** When an extension subscribes with `includeContent: true`,
it's clear the extension expects and will use that content. Signal-only
subscriptions indicate the extension handles content fetching itself.

### Format Selection

When `includeContent` is true, the `format` option determines the content
format:

| Format | Output                     | Best for                            |
|--------|----------------------------|-------------------------------------|
| `hl7`  | Raw HL7 with `\r` segments | Parsing, sending to other systems   |
| `json` | Hierarchical JSON tree     | JavaScript extensions, data access  |
| `yaml` | Human-readable YAML        | Debugging, logging                  |
| `toml` | TOML format                | Configuration-style access          |

Default is `hl7` if not specified.

## Event Flow

Understanding how events flow through the system helps extension developers
design robust handlers.

### Single-Direction Flow

Events are notifications, not requests:

```
Hermes                                    Extension
  │                                           │
  │──── message/changed (notification) ──────>│
  │     (no id field)                         │
  │                                           │
  │     Extension does NOT respond            │
  │                                           │
```

**Why no response?**

Events inform extensions about state changes. The extension either:
- Ignores the event (uninteresting change)
- Processes the event (performs analysis, updates state)

Neither outcome requires acknowledgement. If the extension needs to act on
Hermes (e.g., patch the message), it sends a separate request:

```
Hermes                                    Extension
  │                                           │
  │──── message/changed ─────────────────────>│
  │                                           │
  │     ... extension processes ...           │
  │                                           │
  │<──── editor/patchMessage (request) ───────│
  │                                           │
  │──── patchMessage result ─────────────────>│
  │                                           │
```

### Event Ordering

Events are delivered in the order they occur, but extensions should not depend
on exact ordering guarantees:

- `message/opened` always precedes `message/changed` for a new file
- `message/saved` reflects state at save time (before or after pending changes)
- Multiple rapid changes coalesce into one `message/changed` (debouncing)

Extensions should treat each event as a point-in-time notification and fetch
current state if precise ordering matters.

### Concurrent Events

Extensions may receive events while processing previous ones. Design
considerations:

**Sequential processing:**
```python
event_queue = []

def handle_event(event):
    event_queue.append(event)
    if not processing:
        process_queue()
```

**Replace-on-new (common for message/changed):**
```python
pending_message = None

def handle_message_changed(params):
    pending_message = params.get("message")
    schedule_processing()

def process():
    msg = pending_message
    pending_message = None
    analyze(msg)
```

The right approach depends on the extension's semantics. For `message/changed`,
replace-on-new often makes sense since intermediate states are obsolete.

## Use Cases

Each event serves different extension patterns.

### message/opened

Triggered when a file is opened or a new message is created.

**Use cases:**
- Load extension state associated with the file
- Fetch metadata from external systems based on file path
- Reset analysis state for fresh message
- Display welcome dialog for new users

**Example: Project-aware extension**
```python
def handle_message_opened(params):
    file_path = params.get("filePath")
    if file_path:
        project = detect_project(file_path)
        load_project_settings(project)
    else:
        # new/untitled message
        reset_to_defaults()
```

### message/saved

Triggered when the message is saved to disk.

**Use cases:**
- Sync to external systems (database, API, version control)
- Trigger backup or archival
- Log audit trail
- Update file metadata

**Example: External sync**
```python
def handle_message_saved(params):
    file_path = params["filePath"]
    message = hermes.get_message(format="hl7")
    external_api.upload(file_path, message)
    log(f"Synced {file_path} to external system")
```

### message/changed

Triggered when editor content changes (debounced).

**Use cases:**
- Real-time validation or analysis
- Live preview in extension window
- Background processing (spell check, code completion)
- External system sync (with additional debouncing)

**Example: Live analysis**
```python
def handle_message_changed(params):
    message = params.get("message")
    if not message:
        message = hermes.get_message(format="json")

    issues = analyze(message)
    update_status_display(issues)
```

## Conclusion

The event system enables reactive extension patterns that would be inefficient
or impossible with polling. Key design decisions:

1. **Push over pull:** Events eliminate polling overhead and provide instant
   notification.

2. **Opt-in subscription:** Extensions declare interest at startup, enabling
   efficient selective delivery.

3. **Debouncing:** The 500ms window for `message/changed` balances
   responsiveness with efficiency.

4. **Flexible content delivery:** Signal-only by default, with opt-in content
   inclusion for extensions that need it.

5. **Fire-and-forget:** Events are notifications, not requests. Extensions
   process asynchronously without acknowledgement.

Understanding these patterns helps extension developers build responsive,
efficient integrations that react to user actions without unnecessary overhead.
