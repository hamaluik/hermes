# Tutorial: Your First Extension

In this tutorial, you'll build your first Hermes extension from scratch. By
the end, you'll have a working extension that adds a button to the toolbar and
modifies HL7 messages when clicked.

## What You'll Build

A simple extension that:
- Adds a "Set Sample Patient" button to Hermes' toolbar
- When clicked, sets PID.5 (patient name) to "DOE^JOHN"
- Logs messages that you can see in the extension logs

This is the "Hello World" of Hermes extensions—minimal but complete.

## What You'll Learn

- How to structure an extension's main loop
- Reading and writing JSON-RPC messages with Content-Length framing
- Handling the initialize handshake
- Registering toolbar buttons with SVG icons
- Executing commands when buttons are clicked
- Patching HL7 messages using field paths

## Step 1: Create the File

Create a new file called `my_first_extension.py`:

```bash
touch my_first_extension.py
chmod +x my_first_extension.py
```

The `chmod +x` makes it executable, which Hermes requires.

Open the file in your text editor and add the shebang line:

```python
#!/usr/bin/env python3
"""
My First Hermes Extension

Sets a sample patient name when the button is clicked.
"""
```

This tells the system to run the file with Python 3.

## Step 2: Import Libraries

Add these imports at the top of your file:

```python
import sys
import json
```

That's all you need! Extensions communicate via stdin/stdout using JSON, so
the standard library has everything we need.

## Step 3: Add Message Reading

Extensions communicate with Hermes using JSON-RPC messages wrapped in
Content-Length headers. Let's add functions to read and write these messages.

Add this code:

```python
# ============================================================================
# Message I/O
# ============================================================================

def read_message():
    """Read a JSON-RPC message from stdin."""
    # read headers until we hit a blank line
    headers = {}
    while True:
        line = sys.stdin.readline()
        if line == "\r\n" or line == "\n":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            headers[key.strip()] = value.strip()

    # get the content length
    content_length = int(headers.get("Content-Length", 0))
    if content_length == 0:
        return None

    # read exactly that many bytes
    content = sys.stdin.read(content_length)
    return json.loads(content)
```

This function:
1. Reads header lines until it finds a blank line (`\r\n`)
2. Extracts the `Content-Length` value
3. Reads exactly that many bytes
4. Parses the JSON

## Step 4: Add Message Writing

Now add the function to send messages back to Hermes:

```python
def write_message(msg):
    """Write a JSON-RPC message to stdout."""
    content = json.dumps(msg)
    content_bytes = content.encode("utf-8")
    sys.stdout.write(f"Content-Length: {len(content_bytes)}\r\n\r\n")
    sys.stdout.write(content)
    sys.stdout.flush()
```

**Important**: We calculate the byte length (not character count) because
Content-Length counts UTF-8 bytes. Always call `flush()` to ensure the message
is sent immediately.

## Step 5: Add a Logging Helper

Extensions should log to stderr (not stdout, which is reserved for JSON-RPC):

```python
def log(message):
    """Log to stderr (visible in Hermes logs)."""
    sys.stderr.write(f"[my-extension] {message}\n")
    sys.stderr.flush()
```

You'll see these logs in Hermes' extension logs (Settings > Extensions > View
Logs).

## Step 6: Handle the Initialize Request

When Hermes starts your extension, it sends an `initialize` request. You must
respond with your extension's name, version, and capabilities.

Add this handler:

```python
# ============================================================================
# Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle the initialize handshake."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "My First Extension",
            "version": "1.0.0",
            "description": "Sets a sample patient name",
            "capabilities": {
                "commands": ["myext/setPatient"]
            },
            "toolbarButtons": [
                {
                    "id": "myext-set-patient",
                    "label": "Set Sample Patient",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                        <circle cx="12" cy="7" r="4"/>
                    </svg>""",
                    "command": "myext/setPatient"
                }
            ]
        }
    }
```

Breaking this down:

- **name/version/description**: Metadata shown in settings
- **capabilities.commands**: List of command IDs this extension handles
- **toolbarButtons**: Buttons to add to Hermes' toolbar
  - **id**: Unique identifier for this button
  - **label**: Text shown on hover
  - **icon**: SVG markup (must use `currentColor` for proper theming)
  - **command**: Which command to trigger when clicked

## Step 7: Add Request Helper

Commands need to send requests to Hermes (like "patch this message"). Add this
helper to track request IDs:

```python
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

    # read messages until we get our response
    while True:
        msg = read_message()
        if msg is None:
            raise Exception("Connection closed")

        # is this a response to our request?
        if "result" in msg or "error" in msg:
            if msg.get("id") == request_id:
                return msg
```

This sends a request and waits for the matching response.

## Step 8: Handle Command Execution

When the user clicks your toolbar button, Hermes sends a `command/execute`
notification. Add the handler:

```python
def handle_command(params):
    """Handle a command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "myext/setPatient":
        execute_set_patient()
    else:
        log(f"Unknown command: {command}")
```

Commands are "fire-and-forget"—Hermes doesn't wait for a response.

## Step 9: Implement the Command Logic

Now implement what happens when the button is clicked:

```python
def execute_set_patient():
    """Set the patient name to DOE^JOHN."""
    log("Setting patient name...")

    # patch the message with sample data
    response = send_request("editor/patchMessage", {
        "patches": [
            {"path": "PID.5.1", "value": "DOE"},
            {"path": "PID.5.2", "value": "JOHN"}
        ]
    })

    # check for errors
    if "error" in response:
        log(f"Failed to patch message: {response['error']['message']}")
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        log(f"Patch failed: {error_msg}")
        return

    log("Patient name set to DOE^JOHN")
```

The `patches` array specifies HL7 field paths and values:
- `PID.5.1` = last name (family name)
- `PID.5.2` = first name (given name)

## Step 10: Handle Shutdown

When Hermes closes or disables your extension, it sends a `shutdown` request:

```python
def handle_shutdown(request_id, params):
    """Handle shutdown request."""
    log("Shutting down")
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {"success": True}
    }
```

Always respond to shutdown, then exit gracefully.

## Step 11: Add Message Router

Add a function to route incoming messages to the right handler:

```python
def handle_message(msg):
    """Route a message to the appropriate handler."""
    method = msg.get("method")
    request_id = msg.get("id")
    params = msg.get("params", {})

    # check if this is a notification (no id field)
    if request_id is None:
        if method == "command/execute":
            handle_command(params)
        else:
            log(f"Unknown notification: {method}")
        return None

    # handle requests (with id field)
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

Notifications (like `command/execute`) don't have an `id` field. Requests do.

## Step 12: Add the Main Loop

Finally, add the main loop that runs forever, reading messages:

```python
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

This loop:
1. Reads a message
2. Routes it to the appropriate handler
3. Sends back any response
4. Repeats until the connection closes or an error occurs

## Checkpoint: Test Your Extension

Your extension is now complete! Let's test it.

### Manual Test

Run your extension directly to verify the protocol works:

```bash
python3 my_first_extension.py
```

The extension starts and waits for input. Paste this initialize request
(including the blank line):

```
Content-Length: 95

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"hermesVersion":"1.0.0","apiVersion":"1.0.0","dataDirectory":"/tmp"}}
```

You should see a response with your toolbar button configuration. Press Ctrl+C
to exit.

### Test in Hermes

1. Open Hermes and go to Settings (File > Settings or Cmd+,)

2. Scroll to the Extensions section

3. In the command field, enter the full path to your extension:
   ```
   /full/path/to/my_first_extension.py
   ```

4. Click "Add Extension"

5. Click "Reload Extensions" at the bottom

6. Check the status—it should show "Running" in green

### Test the Button

1. Create or open an HL7 message with a PID segment

2. Look for your button in the toolbar (person icon)

3. Click it

4. Check that PID.5 now contains `DOE^JOHN`

5. Open the extension logs (Settings > Extensions > View Logs) to see your log
   messages

## What You've Built

Congratulations! You've built a complete Hermes extension. Here's what it does:

```
┌─────────────────────────────────────────────────────┐
│ Hermes                                              │
│                                                     │
│  Toolbar: [ ... ] [👤 Set Sample Patient]           │
│                           │                         │
│                           │ (click)                 │
│                           ▼                         │
│                   command/execute                   │
│                           │                         │
│  ┌────────────────────────┼─────────────────────┐   │
│  │        Extension Process                     │   │
│  │                        │                     │   │
│  │    handle_command ◄────┘                     │   │
│  │           │                                  │   │
│  │           ▼                                  │   │
│  │    execute_set_patient                       │   │
│  │           │                                  │   │
│  │           ├──► editor/patchMessage           │   │
│  │           │    [PID.5.1=DOE, PID.5.2=JOHN]   │   │
│  │           │                                  │   │
│  │           ◄──── success response             │   │
│  │                                              │   │
│  └──────────────────────────────────────────────┘   │
│                                                     │
│  Message: MSH|^~\&|...                              │
│           PID|||12345||DOE^JOHN||...                │
│                        ────────                     │
│                        updated!                     │
└─────────────────────────────────────────────────────┘
```

## Complete Code

Here's the complete extension in one piece:

```python
#!/usr/bin/env python3
"""
My First Hermes Extension

Sets a sample patient name when the button is clicked.
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
    sys.stderr.write(f"[my-extension] {message}\n")
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
# Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle the initialize handshake."""
    log(f"Initialising with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "My First Extension",
            "version": "1.0.0",
            "description": "Sets a sample patient name",
            "capabilities": {
                "commands": ["myext/setPatient"]
            },
            "toolbarButtons": [
                {
                    "id": "myext-set-patient",
                    "label": "Set Sample Patient",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                        <circle cx="12" cy="7" r="4"/>
                    </svg>""",
                    "command": "myext/setPatient"
                }
            ]
        }
    }


def handle_command(params):
    """Handle a command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    if command == "myext/setPatient":
        execute_set_patient()
    else:
        log(f"Unknown command: {command}")


def execute_set_patient():
    """Set the patient name to DOE^JOHN."""
    log("Setting patient name...")

    response = send_request("editor/patchMessage", {
        "patches": [
            {"path": "PID.5.1", "value": "DOE"},
            {"path": "PID.5.2", "value": "JOHN"}
        ]
    })

    if "error" in response:
        log(f"Failed to patch message: {response['error']['message']}")
        return

    if not response.get("result", {}).get("success"):
        errors = response.get("result", {}).get("errors", [])
        error_msg = errors[0]["message"] if errors else "Unknown error"
        log(f"Patch failed: {error_msg}")
        return

    log("Patient name set to DOE^JOHN")


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
        if method == "command/execute":
            handle_command(params)
        else:
            log(f"Unknown notification: {method}")
        return None

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

You now understand:

- **Message framing**: Content-Length headers wrap JSON-RPC messages
- **Request/response pattern**: Extensions and Hermes communicate
  bidirectionally
- **Initialize handshake**: Extensions declare their capabilities at startup
- **Toolbar buttons**: SVG icons and command registration
- **Command execution**: Fire-and-forget notifications trigger extension logic
- **Patching messages**: Field paths like `PID.5.1` modify specific fields

## Next Steps

Now that you have a working extension:

1. Try modifying different fields (like `PID.7` for date of birth)
2. Add more toolbar buttons with different icons
3. Read the message before patching it using `editor/getMessage`
4. Move on to [Adding Toolbar Buttons](toolbar-buttons.md) to learn more about
   icons and multiple commands

You're ready to build real extensions!
