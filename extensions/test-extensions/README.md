# Test Extensions

Focused test scripts for verifying each area of the Hermes Extension API.
These extensions are designed for testing and validation rather than
production use.

## Purpose

Each test extension exercises a specific part of the API, logging results to
stderr so you can verify behaviour in the Extension Logs modal. Use these to:

- Verify API methods work correctly
- Test error handling and edge cases
- Understand API behaviour through examples
- Debug issues with your own extensions

## Usage

### Adding to Hermes

1. Open Hermes and go to Settings (Cmd+,)
2. Scroll to the Extensions section
3. Click "Add Extension"
4. Enter the full path to a test script, e.g.,
   `/path/to/hermes/extensions/test-extensions/echo.py`
5. Click "Add Extension"
6. Repeat for other test scripts you want to load

### Running Tests

1. Open the Extension Logs modal via Settings (click "View Logs")
2. Trigger commands via toolbar buttons (added by extensions)
3. Watch the logs for output and error messages
4. Filter logs by extension to focus on specific tests

### Interpreting Results

- **Info logs** - Normal operation, successful API calls
- **Error logs** - Something went wrong (check the error message)
- Output shows request parameters and response data

## Available Test Extensions

### echo.py

**Tests:** Basic extension lifecycle and communication

Minimal test that verifies extension loading and command execution work
correctly. Responds to initialize and provides a single command that logs
"pong" when triggered.

**Commands:**
- `echo/ping` - Logs "pong" to stderr

**Use this to:** Verify Hermes can start your extension and route commands.

### editor-ops.py

**Tests:** All editor API methods

Exercises `getMessage`, `patchMessage`, and `setMessage` with all supported
formats (HL7, JSON, YAML, TOML). Each command logs the request and response
for inspection.

**Commands:**
- `editor-test/get-hl7` - Get message in HL7 format
- `editor-test/get-json` - Get message in JSON format
- `editor-test/get-yaml` - Get message in YAML format
- `editor-test/get-toml` - Get message in TOML format
- `editor-test/patch` - Patch PID.5.1 to "TEST"
- `editor-test/set` - Replace message with simple test message

**Use this to:** Verify editor operations and format conversions work correctly.

### dialogs.py

**Tests:** All dialog API methods

Shows system-native dialogs for messages, confirmations, and file/directory
selection. Each command logs the dialog result.

**Commands:**
- `dialog-test/message-info` - Show info message
- `dialog-test/message-warn` - Show warning message
- `dialog-test/message-error` - Show error message
- `dialog-test/confirm` - Show confirmation dialog
- `dialog-test/open-file` - Open file selection dialog
- `dialog-test/open-files` - Open multiple file selection dialog
- `dialog-test/save-file` - Open save file dialog
- `dialog-test/select-dir` - Open directory selection dialog

**Use this to:** Test dialog behaviour and user interaction patterns.

### windows.py

**Tests:** Window management API

Opens browser windows at different URLs and closes them programmatically.
Starts a local HTTP server to serve test content.

**Commands:**
- `window-test/open` - Open a window, store its ID
- `window-test/close` - Close the previously opened window
- `window-test/open-modal` - Open a modal window

**Use this to:** Verify window creation, lifecycle, and the closeWindow API.

### schema-override.py

**Tests:** Schema override merging

Provides schema overrides in the initialize response. The overrides should
be visible in Hermes' field descriptions and validation behaviour.

**Schema overrides:**
- PID.3.1 - Custom note and validation pattern
- OBX.2 - Custom allowed values

**Commands:**
- `schema-test/verify` - Logs that extension loaded

**Use this to:** Verify schema merging works and custom validations appear.

## Requirements

- Python 3.x (no external dependencies required)
- Hermes with extension support enabled

## Troubleshooting

**Extension shows "Failed" status:**
- Check Extension Logs for startup errors
- Verify the Python interpreter can run the script
- Ensure the script has execute permissions (`chmod +x script.py`)

**Commands don't appear in toolbar:**
- Check that the extension is "Running" (green indicator)
- Verify the initialize response includes toolbar buttons
- Try reloading extensions (Settings > Reload Extensions)

**No output in Extension Logs:**
- Ensure you're triggering commands via toolbar buttons
- Check that logging goes to stderr (stdout is reserved for JSON-RPC)
- Verify the extension process is still running

## Writing Your Own Tests

Use these scripts as templates for your own extensions. Key patterns:

1. **Message I/O:** Use Content-Length framing for all JSON-RPC messages
2. **Request tracking:** Maintain unique IDs for outgoing requests
3. **Error handling:** Log errors to stderr for visibility
4. **Cleanup:** Handle shutdown gracefully, close resources
