# Handle Events

This guide shows how to subscribe to and handle event notifications from Hermes.

## Subscribe to Events

Declare event subscriptions in your `initialize` response:

```python
def handle_initialize(params):
    return {
        "name": "My Extension",
        "version": "1.0.0",
        "capabilities": {
            "commands": ["ext/my-command"],
            "events": [
                {"name": "message/changed"},
                {"name": "message/opened"},
                {"name": "message/saved"}
            ]
        }
    }
```

Only listed events are delivered to your extension.

## Handle Message Changes

Set up a handler for `message/changed` notifications:

```python
def handle_notification(method, params):
    if method == "message/changed":
        handle_message_changed(params)
    elif method == "message/opened":
        handle_message_opened(params)
    elif method == "message/saved":
        handle_message_saved(params)

def handle_message_changed(params):
    has_file = params.get("hasFile", False)
    file_path = params.get("filePath")

    log(f"Message changed (file: {file_path if has_file else 'untitled'})")

    # fetch current message to process it
    response = send_request("editor/getMessage", {"format": "json"})
    if "error" in response:
        return

    message = json.loads(response["result"]["message"])
    analyse_message(message)
```

The `message/changed` event is debounced (500ms), so rapid keystrokes coalesce
into a single notification.

## Receive Content with Change Events

Request content delivery to avoid a separate `getMessage` call:

```python
def handle_initialize(params):
    return {
        "name": "My Extension",
        "version": "1.0.0",
        "capabilities": {
            "events": [
                {
                    "name": "message/changed",
                    "options": {
                        "includeContent": True,
                        "format": "json"
                    }
                }
            ]
        }
    }

def handle_message_changed(params):
    # content is included in the notification
    message_json = params.get("message")
    format = params.get("format")  # "json"

    if message_json:
        message = json.loads(message_json)
        analyse_message(message)
```

Available formats: `hl7`, `json`, `yaml`, `toml`. Default is `hl7`.

## Handle File Open Events

React when a file is opened or a new message is created:

```python
def handle_message_opened(params):
    is_new = params.get("isNew", False)
    file_path = params.get("filePath")

    if is_new:
        log("New untitled message created")
        reset_extension_state()
    else:
        log(f"Opened file: {file_path}")
        load_project_settings(file_path)
```

The `message/opened` event fires for:
- File > Open
- Drag-and-drop
- File > New
- File > New from Template

## Handle Save Events

React when a message is saved to disk:

```python
def handle_message_saved(params):
    file_path = params["filePath"]
    save_as = params.get("saveAs", False)

    if save_as:
        log(f"Message saved as: {file_path}")
    else:
        log(f"Message saved: {file_path}")

    # sync to external system
    response = send_request("editor/getMessage", {"format": "hl7"})
    if "error" not in response:
        message = response["result"]["message"]
        upload_to_external_system(file_path, message)
```

The `saveAs` field distinguishes Save (false) from Save As (true). Auto-save
triggers this event with `saveAs: false`.

## Complete Event Loop Example

```python
import json
import sys

def main():
    while True:
        message = read_message()
        if message is None:
            break

        if "method" in message and "id" not in message:
            # notification (no id field)
            handle_notification(message["method"], message.get("params", {}))
        elif "method" in message and "id" in message:
            # request (has id field)
            handle_request(message["id"], message["method"], message.get("params", {}))
        elif "result" in message or "error" in message:
            # response to our request
            handle_response(message)

def handle_notification(method, params):
    if method == "message/changed":
        handle_message_changed(params)
    elif method == "message/opened":
        handle_message_opened(params)
    elif method == "message/saved":
        handle_message_saved(params)
    elif method == "command/execute":
        handle_command(params)

def handle_message_changed(params):
    log(f"Message changed, hasFile={params.get('hasFile')}")

def handle_message_opened(params):
    if params.get("isNew"):
        log("New message created")
    else:
        log(f"Opened: {params.get('filePath')}")

def handle_message_saved(params):
    log(f"Saved: {params['filePath']} (saveAs={params.get('saveAs')})")
```

## Live Preview Pattern

Use `message/changed` with content delivery for real-time UI updates:

```python
def handle_initialize(params):
    return {
        "name": "Live Preview",
        "version": "1.0.0",
        "capabilities": {
            "commands": ["ext/show-preview"],
            "events": [
                {
                    "name": "message/changed",
                    "options": {"includeContent": True, "format": "json"}
                }
            ]
        }
    }

preview_window_id = None

def handle_command(params):
    global preview_window_id
    if params["command"] == "ext/show-preview":
        # open preview window
        response = send_request("ui/openWindow", {
            "url": "file:///path/to/preview.html",
            "title": "Live Preview",
            "width": 600,
            "height": 400
        })
        if "result" in response:
            preview_window_id = response["result"]["windowId"]

def handle_message_changed(params):
    if preview_window_id is None:
        return

    message = params.get("message")
    if message:
        # update preview window via your preferred mechanism
        update_preview(json.loads(message))
```

## External Sync Pattern

Use `message/saved` to sync with external systems:

```python
import requests

def handle_message_saved(params):
    file_path = params["filePath"]

    # fetch the saved message
    response = send_request("editor/getMessage", {"format": "hl7"})
    if "error" in response:
        log(f"Failed to get message: {response['error']['message']}")
        return

    message = response["result"]["message"]

    # upload to external API
    try:
        api_response = requests.post(
            "https://api.example.com/messages",
            json={"path": file_path, "content": message},
            timeout=10
        )
        api_response.raise_for_status()
        log(f"Synced {file_path} to external system")
    except requests.RequestException as e:
        log(f"Sync failed: {e}")
        send_request("ui/showMessage", {
            "message": f"Failed to sync to external system:\n{e}",
            "title": "Sync Error",
            "kind": "warning"
        })
```

## Project-Aware Extension Pattern

Use `message/opened` to load project-specific configuration:

```python
import os

project_config = None

def handle_message_opened(params):
    global project_config

    if params.get("isNew"):
        project_config = None
        return

    file_path = params.get("filePath")
    if not file_path:
        return

    # look for project config in parent directories
    directory = os.path.dirname(file_path)
    while directory != os.path.dirname(directory):  # stop at root
        config_path = os.path.join(directory, ".hermes-project.json")
        if os.path.exists(config_path):
            with open(config_path, "r") as f:
                project_config = json.load(f)
            log(f"Loaded project config from {config_path}")
            return
        directory = os.path.dirname(directory)

    project_config = None
    log("No project config found")
```

## Handling Concurrent Events

Events may arrive while processing previous ones. For `message/changed`, newer
events supersede older ones:

```python
import threading

pending_message = None
processing_lock = threading.Lock()
process_scheduled = False

def handle_message_changed(params):
    global pending_message, process_scheduled

    with processing_lock:
        pending_message = params.get("message")

        if not process_scheduled:
            process_scheduled = True
            threading.Thread(target=process_pending).start()

def process_pending():
    global pending_message, process_scheduled

    while True:
        with processing_lock:
            message = pending_message
            pending_message = None

            if message is None:
                process_scheduled = False
                return

        # process outside the lock
        analyse_message(json.loads(message))
```

## Events Summary

| Event             | Trigger                         | Key Fields                   |
|-------------------|---------------------------------|------------------------------|
| `message/changed` | Editor content modified         | `hasFile`, `filePath`, `message`* |
| `message/opened`  | File opened or new message      | `isNew`, `filePath`          |
| `message/saved`   | Message saved to disk           | `filePath`, `saveAs`         |

\* Only present if `includeContent: true` in subscription options.

## Related Documentation

- [Explanation: Events](../explanation/events.md) - Design decisions and
  debouncing behaviour
- [Reference: message/changed](../reference/api/message-changed.md)
- [Reference: message/opened](../reference/api/message-opened.md)
- [Reference: message/saved](../reference/api/message-saved.md)
