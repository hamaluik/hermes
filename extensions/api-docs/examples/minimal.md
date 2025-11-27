# Minimal Extension Example

This example demonstrates the bare minimum needed to create a working Hermes extension. It adds a toolbar button that, when clicked, sets a sample patient name in the current message.

## What It Does

1. Registers a single toolbar button with a person icon
2. When clicked, patches the PID segment with "DOE^JOHN"
3. Shows a success message to the user

## Python Implementation

Save this as `minimal_extension.py` and make it executable (`chmod +x minimal_extension.py`):

```python
#!/usr/bin/env python3
"""
Minimal Hermes Extension Example

Adds a toolbar button that sets a sample patient name.
"""

import sys
import json

# ============================================================================
# Message I/O
# ============================================================================

def read_message():
    """Read a JSON-RPC message from stdin."""
    # read headers
    headers = {}
    while True:
        line = sys.stdin.readline()
        if line == "\r\n" or line == "\n":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            headers[key.strip()] = value.strip()

    # read content
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
    sys.stderr.write(f"[minimal-ext] {message}\n")
    sys.stderr.flush()


# ============================================================================
# Request Helpers
# ============================================================================

# Track our outgoing request IDs
_next_id = 1
_pending = {}


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
            else:
                # response to a different request, store it
                _pending[msg.get("id")] = msg
        else:
            # this is a request from Hermes, we need to handle it
            # (shouldn't happen during our request, but handle it)
            response = handle_message(msg)
            if response:
                write_message(response)

    return _pending.pop(request_id)


# ============================================================================
# Handlers
# ============================================================================

def handle_initialize(request_id, params):
    """Handle the initialize handshake."""
    log(f"Initializing with Hermes {params.get('hermesVersion')}")

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "name": "Minimal Extension",
            "version": "1.0.0",
            "description": "A minimal example extension",
            "capabilities": {
                "commands": ["minimal/setPatient"]
            },
            "toolbarButtons": [
                {
                    "id": "minimal-set-patient",
                    "label": "Set Sample Patient",
                    "icon": """<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                        <circle cx="12" cy="7" r="4"/>
                    </svg>""",
                    "command": "minimal/setPatient"
                }
            ]
        }
    }


def handle_command(params):
    """Handle a command execution notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    # check if we recognise this command
    if command != "minimal/setPatient":
        log(f"Unknown command: {command}")
        return

    # execute the command directly
    execute_set_patient()


def execute_set_patient():
    """Execute the setPatient command."""
    # patch the message with sample data
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

    # check if this is a notification (no id field)
    if request_id is None:
        # handle notifications
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
                "message": "Method not found",
                "data": f"Unknown method: {method}"
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

## Node.js Implementation

Save this as `minimal_extension.js`:

```javascript
#!/usr/bin/env node
/**
 * Minimal Hermes Extension Example (Node.js)
 */

const readline = require('readline');

// ============================================================================
// Message I/O
// ============================================================================

let buffer = '';
let contentLength = null;

function parseMessages() {
  const messages = [];

  while (true) {
    if (contentLength === null) {
      const headerEnd = buffer.indexOf('\r\n\r\n');
      if (headerEnd === -1) break;

      const header = buffer.slice(0, headerEnd);
      const match = header.match(/Content-Length: (\d+)/);
      if (!match) throw new Error('Invalid header');

      contentLength = parseInt(match[1], 10);
      buffer = buffer.slice(headerEnd + 4);
    }

    if (buffer.length < contentLength) break;

    const content = buffer.slice(0, contentLength);
    messages.push(JSON.parse(content));
    buffer = buffer.slice(contentLength);
    contentLength = null;
  }

  return messages;
}

function writeMessage(msg) {
  const content = JSON.stringify(msg);
  const contentLength = Buffer.byteLength(content, 'utf8');
  process.stdout.write(`Content-Length: ${contentLength}\r\n\r\n`);
  process.stdout.write(content);
}

function log(message) {
  process.stderr.write(`[minimal-ext] ${message}\n`);
}

// ============================================================================
// Request Helpers
// ============================================================================

let nextId = 1;
const pending = new Map();

async function sendRequest(method, params) {
  const requestId = nextId++;

  return new Promise((resolve) => {
    pending.set(requestId, resolve);
    writeMessage({
      jsonrpc: '2.0',
      id: requestId,
      method,
      params
    });
  });
}

// ============================================================================
// Handlers
// ============================================================================

function handleInitialize(requestId, params) {
  log(`Initializing with Hermes ${params.hermesVersion}`);

  return {
    jsonrpc: '2.0',
    id: requestId,
    result: {
      name: 'Minimal Extension',
      version: '1.0.0',
      description: 'A minimal example extension',
      capabilities: { commands: ['minimal/setPatient'] },
      toolbarButtons: [{
        id: 'minimal-set-patient',
        label: 'Set Sample Patient',
        icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
          <circle cx="12" cy="7" r="4"/>
        </svg>`,
        command: 'minimal/setPatient'
      }]
    }
  };
}

function handleCommand(params) {
  const { command } = params;
  log(`Executing command: ${command}`);

  // check if we recognise this command
  if (command !== 'minimal/setPatient') {
    log(`Unknown command: ${command}`);
    return;
  }

  // execute the command directly (async)
  executeSetPatient();
}

async function executeSetPatient() {
  const response = await sendRequest('editor/patchMessage', {
    patches: [
      { path: 'PID.5.1', value: 'DOE' },
      { path: 'PID.5.2', value: 'JOHN' }
    ]
  });

  if (response.error || !response.result?.success) {
    log('Failed to patch message');
    return;
  }

  log('Patient name set to DOE^JOHN');
}

function handleShutdown(requestId) {
  log('Shutting down');
  writeMessage({ jsonrpc: '2.0', id: requestId, result: { success: true } });
  process.exit(0);
}

function handleMessage(msg) {
  // check if this is a response to one of our requests
  if (msg.result !== undefined || msg.error !== undefined) {
    const resolver = pending.get(msg.id);
    if (resolver) {
      pending.delete(msg.id);
      resolver(msg);
    }
    return null;
  }

  // handle incoming message
  const { method, id, params = {} } = msg;

  // check if this is a notification (no id field)
  if (id === undefined) {
    // handle notifications
    if (method === 'command/execute') {
      handleCommand(params);
    } else {
      log(`Unknown notification: ${method}`);
    }
    return null;
  }

  // handle requests (with id field)
  switch (method) {
    case 'initialize':
      return handleInitialize(id, params);
    case 'shutdown':
      return handleShutdown(id);
    default:
      return {
        jsonrpc: '2.0',
        id,
        error: { code: -32601, message: 'Method not found' }
      };
  }
}

// ============================================================================
// Main
// ============================================================================

log('Starting');

process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => {
  buffer += chunk;
  const messages = parseMessages();

  for (const msg of messages) {
    const response = handleMessage(msg);
    if (response) {
      writeMessage(response);
    }
  }
});

process.stdin.on('end', () => {
  log('Connection closed');
  process.exit(0);
});
```

## Testing

### Manual Test

1. Run the extension directly:
   ```bash
   ./minimal_extension.py
   ```

2. Paste this initialize request:
   ```
   Content-Length: 95

   {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"hermesVersion":"1.0.0","apiVersion":"1.0.0","dataDirectory":"/tmp"}}
   ```

3. You should see a response with the toolbar button configuration.

### In Hermes

1. Add to settings:
   ```json
   {
     "extensions": [
       { "path": "/path/to/minimal_extension.py", "enabled": true }
     ]
   }
   ```

2. Restart Hermes

3. Look for the person icon in the toolbar

4. Open or create an HL7 message with a PID segment

5. Click the button—PID.5 should be set to "DOE^JOHN"

## Key Concepts Demonstrated

### 1. Message Framing

```python
def write_message(msg):
    content = json.dumps(msg)
    content_bytes = content.encode("utf-8")
    sys.stdout.write(f"Content-Length: {len(content_bytes)}\r\n\r\n")
    sys.stdout.write(content)
    sys.stdout.flush()
```

### 2. Initialize Response

```python
return {
    "name": "Minimal Extension",
    "version": "1.0.0",
    "capabilities": { "commands": ["minimal/setPatient"] },
    "toolbarButtons": [...]
}
```

### 3. Making Requests to Hermes

```python
response = send_request("editor/patchMessage", {
    "patches": [
        {"path": "PID.5.1", "value": "DOE"}
    ]
})
```

### 4. Fire-and-Forget Commands

Commands use a fire-and-forget model: receive notification, handle it, no response needed.

```python
def handle_command(params):
    """Handle a command/execute notification."""
    command = params.get("command")
    log(f"Executing command: {command}")

    # check if we recognise this command
    if command != "minimal/setPatient":
        log(f"Unknown command: {command}")
        return

    # execute the command directly
    execute_set_patient()

def execute_set_patient():
    """Execute the setPatient command."""
    # ... do work ...
    log("Patient name set to DOE^JOHN")
```

## Next Steps

- Add more commands for different operations
- Read the current message before modifying it
- Add schema overrides for custom fields
- Progress to the [Wizard Example](wizard.md) for a complete web UI extension
