# Tutorial: Responding to Events

This tutorial builds on [Working with Messages](working-with-messages.md) to add
real-time event handling. Instead of only reacting when users click toolbar
buttons, your extension will respond to editor activity as it happens.

## What You'll Build

An "Activity Tracker" extension that:
- Detects when files are opened or created
- Knows when the user saves their work
- Reacts to message content changes in real-time
- Shows a summary of all activity on demand

## What You'll Learn

- Declaring event subscriptions in the initialize response
- Routing event notifications alongside commands
- Handling each event type (opened, saved, changed)
- Using `includeContent` to receive message content with change events
- Building reactive patterns that respond to user actions

## Prerequisites

Start with the complete code from [Working with Messages](working-with-messages.md).
You'll add event handling to that foundation.

## Why Events?

Without events, your extension only sees the message when the user explicitly
clicks a button. That works for on-demand tools, but what if you want to:

- Show a live preview that updates as the user types?
- Sync saved messages to an external system automatically?
- Reset extension state when a new file is opened?

Polling (asking "has anything changed?" repeatedly) wastes resources. Events
flip the model: Hermes tells your extension when something happens, so you can
react immediately with zero overhead when nothing changes.

## Step 1: Start Fresh

We'll simplify the extension from the previous tutorial to focus on events.
Replace your existing code with this starting point:

```python
#!/usr/bin/env python3
"""
Activity Tracker Extension

Tracks message open, save, and change events.
"""

import sys
import json

# ============================================================================
# Message I/O
# ============================================================================

def read_message():
    """Read a JSON-RPC message from stdin."""
    headers = {}
    while True:
        line = sys.stdin.readline()
        if line == "\r\n" or line == "\n":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            headers[key.strip()] = value.strip()

    content_length = int(headers.get("Content-Length", 0))
    if content_length == 0:
        return None

    content = sys.stdin.read(content_length)
    return json.loads(content)


def write_message(msg):
    """Write a JSON-RPC message to stdout."""
    content = json.dumps(msg)
    content_bytes = content.encode("utf-8")
    sys.stdout.write(f"Content-Length: {len(content_bytes)}\r\n\r\n")
    sys.stdout.write(content)
    sys.stdout.flush()


def log(message):
    """Log to stderr (visible in Hermes logs)."""
    sys.stderr.write(f"[activity-tracker] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Request Helpers
# ============================================================================

_next_id = 1

def send_request(method, params):
    """Send a request to Hermes and wait for the response."""
    global _next_id

    request_id = _next_id
    _next_id += 1

    write_message({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params
    })

    while True:
        msg = read_message()
        if msg is None:
            raise Exception("Connection closed")

        if "result" in msg or "error" in msg:
            if msg.get("id") == request_id:
                return msg


# ============================================================================
# Handlers (we'll add to these)
# ============================================================================

def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Activity Tracker",
            "version": "1.0.0",
            "description": "Tracks editor activity",
            "capabilities": {
                "commands": []
            }
        }
    }


def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    }


def handle_message(msg):
    """Route a message to the appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    if request_id is None:
        # notification - no response expected
        log(f"Unknown notification: {method}")
        return None

    # requests
    if method == "initialize":
        return handle_initialize(request_id, params)
    elif method == "shutdown":
        response = handle_shutdown(request_id, params)
        write_message(response)
        sys.exit(0)
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }


# ============================================================================
# Main Loop
# ============================================================================

def main():
    log("Starting")

    while True:
        try:
            msg = read_message()
            if msg is None:
                log("Connection closed")
                break

            response = handle_message(msg)
            if response:
                write_message(response)

        except Exception as e:
            log(f"Error: {e}")
            break

    log("Exiting")


if __name__ == "__main__":
    main()
```

This is a minimal extension that handles lifecycle but doesn't do anything
interesting yet. Test that it loads correctly in Hermes before continuing.

## Step 2: Subscribe to Events

Events are opt-in. Your extension must declare which events it wants in the
`initialize` response. Update `handle_initialize` to subscribe to all three
event types:

```python
def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Activity Tracker",
            "version": "1.0.0",
            "description": "Tracks editor activity",
            "capabilities": {
                "commands": [],
                "events": [
                    {"name": "message/opened"},
                    {"name": "message/saved"},
                    {"name": "message/changed"}
                ]
            }
        }
    }
```

The `events` array lists which notifications you want to receive:

| Event             | When Sent                                    |
|-------------------|----------------------------------------------|
| `message/opened`  | File opened via File > Open or drag-and-drop |
| `message/saved`   | Message saved to disk (including auto-save)  |
| `message/changed` | Editor content modified (debounced 500ms)    |

Hermes only sends events you've subscribed to. An extension that doesn't list
`message/saved` won't receive save notifications.

## Step 3: Route Event Notifications

Events arrive as JSON-RPC notifications—messages without an `id` field. Update
`handle_message` to route them:

```python
def handle_message(msg):
    """Route a message to the appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    if request_id is None:
        # notification - no response expected
        if method == "message/opened":
            handle_message_opened(params)
        elif method == "message/saved":
            handle_message_saved(params)
        elif method == "message/changed":
            handle_message_changed(params)
        else:
            log(f"Unknown notification: {method}")
        return None

    # requests
    if method == "initialize":
        return handle_initialize(request_id, params)
    elif method == "shutdown":
        response = handle_shutdown(request_id, params)
        write_message(response)
        sys.exit(0)
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }
```

Events are like commands—both are notifications. The difference is who
initiates them:

- **Commands:** User clicks a toolbar button → Hermes sends `command/execute`
- **Events:** Editor state changes → Hermes sends `message/opened`, etc.

## Step 4: Handle message/opened

When a file is opened or a new message is created, Hermes sends `message/opened`.
Add the handler and a counter to track activity:

```python
# ============================================================================
# Activity Tracking
# ============================================================================

activity = {
    "files_opened": 0,
    "new_messages": 0,
    "saves": 0,
    "changes": 0,
    "last_file": None
}


# ============================================================================
# Event Handlers
# ============================================================================

def handle_message_opened(params):
    """Handle message/opened event."""
    is_new = params.get("isNew", False)
    file_path = params.get("filePath")

    if is_new:
        activity["new_messages"] += 1
        activity["last_file"] = None
        log("New message created")
    else:
        activity["files_opened"] += 1
        activity["last_file"] = file_path
        log(f"File opened: {file_path}")
```

The event parameters tell you:
- `isNew` — True if File > New or File > New from Template
- `filePath` — Path to the opened file (omitted when `isNew` is true)

Use this event to:
- Reset extension state for a fresh message
- Load file-specific settings or cached data
- Connect to external systems based on file location

## Step 5: Handle message/saved

When the user saves (or auto-save triggers), Hermes sends `message/saved`. Add
the handler:

```python
def handle_message_saved(params):
    """Handle message/saved event."""
    file_path = params["filePath"]
    save_as = params.get("saveAs", False)

    activity["saves"] += 1
    activity["last_file"] = file_path

    operation = "Saved as" if save_as else "Saved"
    log(f"{operation}: {file_path}")
```

The event parameters tell you:
- `filePath` — Where the file was saved (always present)
- `saveAs` — True for Save As, false for regular Save or auto-save

Use this event to:
- Sync to external systems (APIs, databases, version control)
- Trigger backup or archival workflows
- Log audit trails

## Step 6: Handle message/changed (Signal Only)

The `message/changed` event fires when the editor content changes. By default,
you only get a notification that something changed—not the content itself:

```python
def handle_message_changed(params):
    """Handle message/changed event."""
    has_file = params.get("hasFile", False)
    file_path = params.get("filePath")

    activity["changes"] += 1

    file_info = file_path if has_file else "(untitled)"
    log(f"Message changed: {file_info}")
```

The event parameters (in signal-only mode):
- `hasFile` — Whether the message has an associated file
- `filePath` — Path if `hasFile` is true

**Important:** Hermes debounces this event with a 500ms window. Rapid keystrokes
coalesce into a single notification. This prevents flooding your extension
during typing.

If you need the actual content, call `editor/getMessage`:

```python
def handle_message_changed(params):
    """Handle message/changed event."""
    activity["changes"] += 1

    # fetch the message content
    response = send_request("editor/getMessage", {"format": "json"})
    if "error" in response:
        log(f"Failed to get message: {response['error']['message']}")
        return

    message = response["result"]["message"]
    log(f"Message changed ({len(message)} chars)")
```

But there's a better way...

## Step 7: Request Content with Events

Instead of making a separate request after each change, you can ask Hermes to
include the content in the event itself. Update your event subscription:

```python
def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Activity Tracker",
            "version": "1.0.0",
            "description": "Tracks editor activity",
            "capabilities": {
                "commands": [],
                "events": [
                    {"name": "message/opened"},
                    {"name": "message/saved"},
                    {
                        "name": "message/changed",
                        "options": {
                            "includeContent": True,
                            "format": "hl7"
                        }
                    }
                ]
            }
        }
    }
```

Now update the handler to use the delivered content:

```python
def handle_message_changed(params):
    """Handle message/changed event."""
    has_file = params.get("hasFile", False)
    file_path = params.get("filePath")
    message = params.get("message", "")
    msg_format = params.get("format", "hl7")

    activity["changes"] += 1

    # count segments in the message
    segment_count = len(message.split("\r")) if message else 0

    file_info = file_path if has_file else "(untitled)"
    log(f"Message changed: {file_info} ({segment_count} segments, {msg_format})")
```

Available formats:

| Format | Best For                                          |
|--------|---------------------------------------------------|
| `hl7`  | Raw HL7 text, line-by-line processing             |
| `json` | Parsing structure, field access in JavaScript     |
| `yaml` | Human-readable debugging                          |
| `toml` | Configuration-style access                        |

**When to use `includeContent`:**
- Extension always processes every change → include content (saves round-trip)
- Extension only cares about some changes → signal-only (filter first, fetch
  if needed)

## Step 8: Add a Status Command

Let's add a toolbar button that shows the activity summary. First, add a
command handler:

```python
def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "activity/showStatus":
        show_activity_status()
    else:
        log(f"Unknown command: {command}")


def show_activity_status():
    """Show activity summary in a dialog."""
    summary = f"""Activity Summary

Files opened: {activity['files_opened']}
New messages: {activity['new_messages']}
Saves: {activity['saves']}
Changes: {activity['changes']}

Last file: {activity['last_file'] or '(none)'}"""

    send_request("ui/showMessage", {
        "title": "Activity Tracker",
        "message": summary,
        "kind": "info"
    })
```

Update `handle_initialize` to register the command and toolbar button:

```python
def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Activity Tracker",
            "version": "1.0.0",
            "description": "Tracks editor activity",
            "capabilities": {
                "commands": ["activity/showStatus"],
                "events": [
                    {"name": "message/opened"},
                    {"name": "message/saved"},
                    {
                        "name": "message/changed",
                        "options": {
                            "includeContent": True,
                            "format": "hl7"
                        }
                    }
                ]
            },
            "toolbarButtons": [
                {
                    "id": "activity-status",
                    "label": "Show Activity",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
                    </svg>""",
                    "command": "activity/showStatus"
                }
            ]
        }
    }
```

Finally, update `handle_message` to route the command:

```python
def handle_message(msg):
    """Route a message to the appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    if request_id is None:
        # notification - no response expected
        if method == "command/execute":
            handle_command(params)
        elif method == "message/opened":
            handle_message_opened(params)
        elif method == "message/saved":
            handle_message_saved(params)
        elif method == "message/changed":
            handle_message_changed(params)
        else:
            log(f"Unknown notification: {method}")
        return None

    # requests
    if method == "initialize":
        return handle_initialize(request_id, params)
    elif method == "shutdown":
        response = handle_shutdown(request_id, params)
        write_message(response)
        sys.exit(0)
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }
```

## Testing

### Test message/opened

1. Load your extension in Hermes
2. Open a file via File > Open
3. Check the extension logs—you should see "File opened: /path/to/file"
4. Create a new message via File > New
5. Check logs—you should see "New message created"

### Test message/saved

1. Make a change to a message
2. Save via Cmd+S
3. Check logs—you should see "Saved: /path/to/file"
4. Use File > Save As to save with a new name
5. Check logs—you should see "Saved as: /path/to/newfile"

### Test message/changed

1. Type in the editor
2. Wait 500ms (the debounce window)
3. Check logs—you should see "Message changed: ..."
4. Type rapidly, then stop
5. Check logs—only one change event should appear (not one per keystroke)

### Test the Status Button

1. Click the Activity button in the toolbar
2. A dialog should show your activity counts
3. Perform more operations and check the button again

## What You've Built

Your extension now reacts to editor activity in real-time:

```
┌─────────────────────────────────────────────────────────────┐
│ Hermes                                                      │
│                                                             │
│  User opens file ─────────► message/opened ─────►┐          │
│                                                  │          │
│  User types ──(500ms)────► message/changed ─────►├──► Extension
│                                                  │          │
│  User saves ──────────────► message/saved ──────►┘          │
│                                                             │
│  User clicks [📈] ────────► command/execute ────►   (shows  │
│                                                     dialog) │
└─────────────────────────────────────────────────────────────┘
```

## Complete Code

```python
#!/usr/bin/env python3
"""
Activity Tracker Extension

Tracks message open, save, and change events.
"""

import sys
import json

# ============================================================================
# Message I/O
# ============================================================================

def read_message():
    """Read a JSON-RPC message from stdin."""
    headers = {}
    while True:
        line = sys.stdin.readline()
        if line == "\r\n" or line == "\n":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            headers[key.strip()] = value.strip()

    content_length = int(headers.get("Content-Length", 0))
    if content_length == 0:
        return None

    content = sys.stdin.read(content_length)
    return json.loads(content)


def write_message(msg):
    """Write a JSON-RPC message to stdout."""
    content = json.dumps(msg)
    content_bytes = content.encode("utf-8")
    sys.stdout.write(f"Content-Length: {len(content_bytes)}\r\n\r\n")
    sys.stdout.write(content)
    sys.stdout.flush()


def log(message):
    """Log to stderr (visible in Hermes logs)."""
    sys.stderr.write(f"[activity-tracker] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Request Helpers
# ============================================================================

_next_id = 1

def send_request(method, params):
    """Send a request to Hermes and wait for the response."""
    global _next_id

    request_id = _next_id
    _next_id += 1

    write_message({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params
    })

    while True:
        msg = read_message()
        if msg is None:
            raise Exception("Connection closed")

        if "result" in msg or "error" in msg:
            if msg.get("id") == request_id:
                return msg


# ============================================================================
# Activity Tracking
# ============================================================================

activity = {
    "files_opened": 0,
    "new_messages": 0,
    "saves": 0,
    "changes": 0,
    "last_file": None
}


# ============================================================================
# Event Handlers
# ============================================================================

def handle_message_opened(params):
    """Handle message/opened event."""
    is_new = params.get("isNew", False)
    file_path = params.get("filePath")

    if is_new:
        activity["new_messages"] += 1
        activity["last_file"] = None
        log("New message created")
    else:
        activity["files_opened"] += 1
        activity["last_file"] = file_path
        log(f"File opened: {file_path}")


def handle_message_saved(params):
    """Handle message/saved event."""
    file_path = params["filePath"]
    save_as = params.get("saveAs", False)

    activity["saves"] += 1
    activity["last_file"] = file_path

    operation = "Saved as" if save_as else "Saved"
    log(f"{operation}: {file_path}")


def handle_message_changed(params):
    """Handle message/changed event."""
    has_file = params.get("hasFile", False)
    file_path = params.get("filePath")
    message = params.get("message", "")
    msg_format = params.get("format", "hl7")

    activity["changes"] += 1

    # count segments in the message
    segment_count = len(message.split("\r")) if message else 0

    file_info = file_path if has_file else "(untitled)"
    log(f"Message changed: {file_info} ({segment_count} segments, {msg_format})")


# ============================================================================
# Command Handlers
# ============================================================================

def handle_command(params):
    """Handle command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "activity/showStatus":
        show_activity_status()
    else:
        log(f"Unknown command: {command}")


def show_activity_status():
    """Show activity summary in a dialog."""
    summary = f"""Activity Summary

Files opened: {activity['files_opened']}
New messages: {activity['new_messages']}
Saves: {activity['saves']}
Changes: {activity['changes']}

Last file: {activity['last_file'] or '(none)'}"""

    send_request("ui/showMessage", {
        "title": "Activity Tracker",
        "message": summary,
        "kind": "info"
    })


# ============================================================================
# Protocol Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle initialize request."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Activity Tracker",
            "version": "1.0.0",
            "description": "Tracks editor activity",
            "capabilities": {
                "commands": ["activity/showStatus"],
                "events": [
                    {"name": "message/opened"},
                    {"name": "message/saved"},
                    {
                        "name": "message/changed",
                        "options": {
                            "includeContent": True,
                            "format": "hl7"
                        }
                    }
                ]
            },
            "toolbarButtons": [
                {
                    "id": "activity-status",
                    "label": "Show Activity",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
                    </svg>""",
                    "command": "activity/showStatus"
                }
            ]
        }
    }


def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
    log(f"Final activity: {activity}")
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    }


def handle_message(msg):
    """Route a message to the appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    if request_id is None:
        # notification - no response expected
        if method == "command/execute":
            handle_command(params)
        elif method == "message/opened":
            handle_message_opened(params)
        elif method == "message/saved":
            handle_message_saved(params)
        elif method == "message/changed":
            handle_message_changed(params)
        else:
            log(f"Unknown notification: {method}")
        return None

    # requests
    if method == "initialize":
        return handle_initialize(request_id, params)
    elif method == "shutdown":
        response = handle_shutdown(request_id, params)
        write_message(response)
        sys.exit(0)
    else:
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        }


# ============================================================================
# Main Loop
# ============================================================================

def main():
    log("Starting")

    while True:
        try:
            msg = read_message()
            if msg is None:
                log("Connection closed")
                break

            response = handle_message(msg)
            if response:
                write_message(response)

        except Exception as e:
            log(f"Error: {e}")
            break

    log("Exiting")


if __name__ == "__main__":
    main()
```

## What You've Learned

- **Event subscriptions:** Declared in `capabilities.events` during initialize
- **Notification routing:** Events and commands both arrive as notifications
  (messages without an `id` field)
- **Event types:** `message/opened`, `message/saved`, `message/changed`
- **Content delivery:** Use `includeContent` and `format` options to receive
  message content with change events
- **Debouncing:** The 500ms window coalesces rapid keystrokes into single events
- **Reactive patterns:** Respond to user actions without polling

## Next Steps

Continue to [Building a Wizard with UI](wizard-with-ui.md) to learn how to
create rich web interfaces for complex extension workflows.

## Related Documentation

- [How-To: Handle Events](../how-to/handle-events.md) — Practical recipes for
  common event patterns
- [Explanation: Events](../explanation/events.md) — Design decisions and
  debouncing rationale
- [Reference: message/changed](../reference/api/message-changed.md)
- [Reference: message/opened](../reference/api/message-opened.md)
- [Reference: message/saved](../reference/api/message-saved.md)
