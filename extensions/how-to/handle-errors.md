# Handle Errors

This guide shows how to handle different types of errors gracefully in your
extension.

## Check for Errors in Responses

Always check for the `error` field before accessing `result`:

```python
response = send_request("editor/getMessage", {"format": "hl7"})

if "error" in response:
    # handle the error
    error = response["error"]
    log(f"Error {error['code']}: {error['message']}")

    # optionally check error details
    if "data" in error:
        log(f"Details: {error['data']}")

    return

# safe to access result now
message = response["result"]["message"]
```

## Categorise Errors by Type

Handle different error categories appropriately:

```python
def handle_error(error):
    """Handle an error based on its category."""
    code = error["code"]
    message = error["message"]

    if code == -32003:
        # user error: no message open
        send_request("ui/showMessage", {
            "message": "Please open a message first.",
            "title": "No Message",
            "kind": "warning"
        })

    elif code == -32005:
        # user error: invalid path syntax
        send_request("ui/showMessage", {
            "message": f"Invalid field path:\n{error.get('data', message)}",
            "title": "Invalid Path",
            "kind": "error"
        })

    elif code == -32011:
        # user error: validation failed
        details = error.get("data", {})
        field = details.get("field", "unknown")
        send_request("ui/showMessage", {
            "message": f"Validation failed for {field}:\n{message}",
            "title": "Validation Error",
            "kind": "error"
        })

    elif code == -32000:
        # system error: might be transient
        log(f"System error: {message}")
        send_request("ui/showMessage", {
            "message": f"Operation failed:\n{message}\n\nPlease try again.",
            "title": "Error",
            "kind": "error"
        })

    else:
        # programming error or unknown
        log(f"Unexpected error {code}: {message}")
        if "data" in error:
            log(f"Error details: {error['data']}")

        send_request("ui/showMessage", {
            "message": "An unexpected error occurred. Check the extension logs for details.",
            "title": "Error",
            "kind": "error"
        })
```

## Error Categories

### User Errors (Recoverable)

These errors occur due to user actions and can be fixed by the user:

```python
# -32003: No message open
if error["code"] == -32003:
    send_request("ui/showMessage", {
        "message": "Please open a message first.",
        "kind": "warning"
    })
    return

# -32005: Invalid path
if error["code"] == -32005:
    log(f"User provided invalid path: {error.get('data')}")
    send_request("ui/showMessage", {
        "message": "The field path is invalid. Please check the format.",
        "kind": "error"
    })
    return

# -32006: Path not found
if error["code"] == -32006:
    send_request("ui/showMessage", {
        "message": f"Field not found: {error.get('data')}",
        "kind": "error"
    })
    return

# -32011: Validation error
if error["code"] == -32011:
    details = error.get("data", {})
    send_request("ui/showMessage", {
        "message": f"Invalid value for {details.get('field', 'field')}",
        "kind": "error"
    })
    return
```

### System Errors (May Need Retry)

Transient issues that might succeed on retry:

```python
# -32000: General error (database, network, etc.)
if error["code"] == -32000:
    log(f"System error: {error['message']}")

    # optionally retry
    retry_count = 0
    max_retries = 3

    while retry_count < max_retries:
        log(f"Retrying... attempt {retry_count + 1}")
        response = send_request("some/operation", params)

        if "error" not in response:
            break  # success

        retry_count += 1
        time.sleep(1)  # wait before retry

    if retry_count == max_retries:
        send_request("ui/showMessage", {
            "message": "Operation failed after multiple attempts.",
            "kind": "error"
        })

# -32008: Window error
if error["code"] == -32008:
    log(f"Window error: {error['message']}")
    send_request("ui/showMessage", {
        "message": "Failed to open window. Please try again.",
        "kind": "error"
    })
```

### Programming Errors (Fix Code)

Bugs in the extension that should be fixed:

```python
# -32001: Not initialised
if error["code"] == -32001:
    log("ERROR: Attempted to use API before initialisation")
    # this is a bug, fix the code

# -32002: Already initialised
if error["code"] == -32002:
    log("ERROR: Called initialise twice")
    # this is a bug, fix the code

# -32601: Method not found
if error["code"] == -32601:
    log(f"ERROR: Unknown method: {error.get('data')}")
    # check spelling, check API version

# -32602: Invalid params
if error["code"] == -32602:
    log(f"ERROR: Invalid parameters: {error.get('data')}")
    # check parameter types and required fields
```

## Handle Partial Success in Patches

The `patchMessage` method applies patches in a best-effort manner:

```python
response = send_request("editor/patchMessage", {
    "patches": [
        {"path": "PID.5.1", "value": "DOE"},
        {"path": "PID.5.2", "value": "JOHN"},
        {"path": "XYZ.1", "value": "INVALID"}  # this will fail
    ]
})

result = response["result"]

if result["success"]:
    # all patches applied
    log(f"Applied all {result['patchesApplied']} patches")
else:
    # some patches failed
    log(f"Applied {result['patchesApplied']} out of {len(patches)} patches")

    for error in result.get("errors", []):
        log(f"Patch {error['index']} ({error['path']}): {error['message']}")

    # decide whether to show an error to the user
    if result["patchesApplied"] == 0:
        # nothing was applied, show error
        send_request("ui/showMessage", {
            "message": "Failed to update message. No changes were made.",
            "kind": "error"
        })
    else:
        # some patches worked, show warning
        send_request("ui/showMessage", {
            "message": f"Partial update: {result['patchesApplied']} fields updated, {len(result.get('errors', []))} failed.",
            "kind": "warning"
        })
```

## Distinguish Dialog Errors from Cancellation

User cancellation is **not an error**:

```python
response = send_request("ui/openFile", {
    "title": "Select File"
})

# check for system error first
if "error" in response:
    # system failed to show the dialog
    log(f"Dialog error: {response['error']['message']}")
    send_request("ui/showMessage", {
        "message": "Failed to show file selection dialog.",
        "kind": "error"
    })
    return

# check for user cancellation
path = response["result"]["path"]
if path is None:
    # user clicked Cancel - this is normal, not an error
    log("User cancelled file selection")
    return

# user selected a file
log(f"User selected: {path}")
process_file(path)
```

## Log Errors for Debugging

Always log errors to stderr for debugging:

```python
def handle_error(error):
    """Log and display an error appropriately."""
    code = error["code"]
    message = error["message"]
    data = error.get("data")

    # always log to stderr (visible in extension logs)
    log(f"Error {code}: {message}")
    if data:
        log(f"Error details: {data}")

    # then decide how to present to the user
    if code in USER_ERROR_CODES:
        show_user_friendly_message(code, message, data)
    elif code in SYSTEM_ERROR_CODES:
        show_retry_message(message)
    else:
        show_generic_error(message)
```

## Wrap Operations in Try-Except

Protect against unexpected exceptions:

```python
def execute_command(params):
    """Execute a command with error handling."""
    try:
        # get the message
        response = send_request("editor/getMessage", {"format": "json"})

        if "error" in response:
            handle_error(response["error"])
            return

        # process the message
        message = json.loads(response["result"]["message"])
        result = process_message(message)

        # update the message
        response = send_request("editor/patchMessage", {
            "patches": result["patches"]
        })

        if "error" in response:
            handle_error(response["error"])
            return

        # show success
        send_request("ui/showMessage", {
            "message": "Operation completed successfully.",
            "kind": "info"
        })

    except json.JSONDecodeError as e:
        log(f"JSON parse error: {e}")
        send_request("ui/showMessage", {
            "message": "Failed to parse message content.",
            "kind": "error"
        })

    except KeyError as e:
        log(f"Missing required field: {e}")
        send_request("ui/showMessage", {
            "message": "Message is missing required fields.",
            "kind": "error"
        })

    except Exception as e:
        log(f"Unexpected error: {e}")
        import traceback
        traceback.print_exc(file=sys.stderr)

        send_request("ui/showMessage", {
            "message": "An unexpected error occurred. Check logs for details.",
            "kind": "error"
        })
```

## Error Response Helper

Create a reusable helper for error responses:

```python
def error_response(code, message, data=None):
    """Create an error response."""
    error = {
        "code": code,
        "message": message
    }
    if data is not None:
        error["data"] = data

    return {
        "jsonrpc": "2.0",
        "id": None,  # set by caller
        "error": error
    }

# use in handlers
def handle_custom_request(request_id, params):
    command = params.get("command")

    if command not in KNOWN_COMMANDS:
        response = error_response(
            -32009,
            "Command not found",
            f"Unknown command: {command}"
        )
        response["id"] = request_id
        return response

    # handle the command
    # ...
```

## Show User-Friendly Messages

Convert technical errors to user-friendly messages:

```python
ERROR_MESSAGES = {
    -32003: "Please open a message before using this feature.",
    -32005: "The field path you entered is invalid. Please check the format (e.g., PID.5.1).",
    -32006: "The specified field does not exist in the message.",
    -32011: "The value you entered does not meet validation requirements.",
}

def show_user_error(error):
    """Show a user-friendly error message."""
    code = error["code"]
    message = ERROR_MESSAGES.get(code, error["message"])

    send_request("ui/showMessage", {
        "message": message,
        "title": "Error",
        "kind": "error"
    })
```

## Error Code Quick Reference

| Code   | Category    | Typical Resolution                  |
| ------ | ----------- | ----------------------------------- |
| -32003 | User        | Open a message first                |
| -32005 | User        | Fix the path syntax                 |
| -32006 | User        | Use a path that exists              |
| -32011 | User        | Use a valid value                   |
| -32000 | System      | Retry or check configuration        |
| -32008 | System      | Close other windows, retry          |
| -32001 | Programming | Wait for initialise before commands |
| -32002 | Programming | Don't call initialise twice         |
| -32601 | Programming | Check method name spelling          |
| -32602 | Programming | Check parameter types               |

## Viewing Logs

All stderr output from your extension is captured by Hermes and displayed in the
Extension Logs modal (Settings > Extensions > View Logs). If your log messages
include common prefixes, Hermes categorises them automatically:

```python
# these are parsed and displayed with appropriate log levels
print("[ERROR] Connection failed", file=sys.stderr)  # shown as Error
print("ERROR: Invalid response", file=sys.stderr)    # shown as Error
print("[WARN] Retrying...", file=sys.stderr)         # shown as Warn
print("INFO: Processing", file=sys.stderr)           # shown as Info
print("plain message", file=sys.stderr)              # shown as Info (default)
```

This makes debugging straightforward: use your language's normal stderr output and
check the Extension Logs modal to see what your extension printed.

## Related Documentation

- [Reference: Error Codes](../reference/errors.md)
- [How-To: Show Dialogs](show-dialogs.md)
- [Explanation: Extension Lifecycle](../explanation/lifecycle.md)
