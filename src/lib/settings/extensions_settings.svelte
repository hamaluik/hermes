<!--
  Extensions Settings Component

  Manages third-party extension configuration within the settings modal. Provides
  controls for:
  - Adding new extensions by entering a command to execute
  - Enabling/disabling individual extensions
  - Viewing extension runtime status (running, failed, etc.)
  - Removing extensions
  - Reloading extensions to apply configuration changes

  ## Extension Configuration

  Each extension is configured with:
  - Path: Command to execute (e.g., "python3 /path/to/ext.py" or "/path/to/binary")
  - Args: Optional command-line arguments (not yet exposed in UI)
  - Env: Optional environment variables (not yet exposed in UI)
  - Enabled: Whether the extension should be started

  The "path" field is actually the full command to execute, allowing for interpreted
  scripts (python3, node, etc.) or compiled binaries.

  ## Runtime Status Display

  The component fetches extension statuses from the backend to show:
  - Current state (starting, initializing, running, stopping, stopped, failed)
  - Error messages for failed extensions
  - Visual indicators (colour-coded status badges)

  ## Workflow

  1. User enters extension command in text input (supports Enter key to add)
  2. Changes are saved to Settings object (auto-persisted to disk)
  3. User clicks "Reload Extensions" to restart the extension host
  4. Extension statuses update automatically via the extensions-changed event

  ## Command Input

  The input field accepts full commands like "python3 /path/to/script.py" or direct
  paths to executables. This provides flexibility for extensions written in any
  language. The input uses monospace font to aid in path readability.

  ## Implementation Notes

  **Inline toggle switch:** The toggle is implemented inline rather than using the
  ToggleSwitch component because ExtensionConfig.enabled can be undefined (defaults
  to true). The ToggleSwitch component requires a bindable boolean, which would cause
  TypeScript errors with potentially undefined values. The inline implementation uses
  a controlled approach with onchange handlers.

  **Status synchronisation:** Live status updates are achieved by listening to the
  `extensions-changed` event emitted by the backend whenever extension states change.
  This avoids polling and provides immediate feedback.

  **Empty state:** When no extensions are configured, a friendly message guides users
  to add their first extension, improving discoverability.
-->
<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import type { Settings, ExtensionConfig } from "../../settings";
  import {
    reloadExtensions,
    getExtensions,
    isExtensionRunning,
    getExtensionError,
    type ExtensionStatus,
  } from "$lib/extensions/extensions";
  import IconDelete from "$lib/icons/IconDelete.svelte";
  import Button from "$lib/components/button.svelte";
  import ExtensionLogsModal from "./extensions_logs_modal.svelte";

  let {
    settings,
  }: {
    settings: Settings;
  } = $props();

  // Extension runtime statuses fetched from backend
  let extensionStatuses: ExtensionStatus[] = $state([]);

  // Track reload operation state
  let isReloading: boolean = $state(false);

  // Track new extension command input
  let newExtensionCommand: string = $state("");

  // Track logs modal visibility
  let showLogsModal: boolean = $state(false);

  // local reactive state for extensions list
  let extensionsList: ExtensionConfig[] = $state(settings.extensions);

  // sync local state with settings object
  $effect(() => {
    extensionsList = settings.extensions;
  });

  async function loadExtensionStatuses() {
    try {
      extensionStatuses = await getExtensions();
    } catch (error) {
      console.error("Failed to load extension statuses:", error);
    }
  }

  function addExtension() {
    const command = newExtensionCommand.trim();
    if (!command) return;

    const newExtension: ExtensionConfig = {
      path: command,
      args: [],
      env: {},
      enabled: true,
    };
    settings.extensions = [...settings.extensions, newExtension];
    extensionsList = settings.extensions;
    newExtensionCommand = "";
  }

  function removeExtension(index: number) {
    settings.extensions = settings.extensions.filter((_, i) => i !== index);
    extensionsList = settings.extensions;
  }

  function setExtensionEnabled(index: number, enabled: boolean) {
    const updated = [...settings.extensions];
    updated[index] = {
      ...updated[index],
      enabled,
    };
    settings.extensions = updated;
    extensionsList = settings.extensions;
  }

  async function handleReload() {
    isReloading = true;
    try {
      await reloadExtensions(settings.extensions);
      // status will be updated via extensions-changed event
    } catch (error) {
      console.error("Failed to reload extensions:", error);
    } finally {
      isReloading = false;
    }
  }

  function getStatusForPath(path: string): ExtensionStatus | undefined {
    // match by path field (backend now includes original path in status)
    return extensionStatuses.find((s) => s.path === path);
  }

  function getStateBadgeClass(status: ExtensionStatus | undefined): string {
    if (!status) return "unknown";
    if (isExtensionRunning(status.state)) return "running";
    const error = getExtensionError(status.state);
    if (error) return "failed";
    return "stopped";
  }

  function getStateLabel(status: ExtensionStatus | undefined): string {
    if (!status) return "Not loaded";
    if (typeof status.state === "string") {
      // format: "starting" -> "Starting"
      return status.state.charAt(0).toUpperCase() + status.state.slice(1);
    } else if (status.state.failed) {
      return "Failed";
    }
    return "Unknown";
  }

  onMount(() => {
    // load initial statuses
    loadExtensionStatuses();

    // listen for extension status changes (batch updates after reload)
    const unlistenExtensionsChangedPromise = listen("extensions-changed", async () => {
      await loadExtensionStatuses();
    });

    // listen for individual extension status changes (real-time updates)
    const unlistenStatusChangedPromise = listen<ExtensionStatus>("extension-status-changed", (event) => {
      const updatedStatus = event.payload;
      // update or add status in the list
      const index = extensionStatuses.findIndex((s) => s.id === updatedStatus.id);
      if (index >= 0) {
        extensionStatuses[index] = updatedStatus;
      } else {
        extensionStatuses = [...extensionStatuses, updatedStatus];
      }
    });

    return () => {
      unlistenExtensionsChangedPromise.then((unlisten) => unlisten());
      unlistenStatusChangedPromise.then((unlisten) => unlisten());
    };
  });
</script>

<div class="extensions-settings">
  <div class="header">
    <h3>Extensions</h3>
    <p class="description">
      Manage third-party extensions that add functionality to Hermes.
    </p>
  </div>

  <div class="add-extension-form">
    <input
      type="text"
      class="command-input"
      bind:value={newExtensionCommand}
      placeholder="python3 /path/to/extension.py or /path/to/extension"
      onkeydown={(e) => e.key === "Enter" && addExtension()}
    />
    <Button variant="primary" onclick={addExtension} disabled={!newExtensionCommand.trim()}>
      Add Extension
    </Button>
  </div>

  {#if extensionsList.length === 0}
    <div class="empty-state">
      <p>No extensions configured.</p>
      <p class="hint">
        Enter a command to run your extension (e.g., "python3 /path/to/extension.py").
      </p>
    </div>
  {:else}
    <div class="extension-list">
      {#each extensionsList as ext, index}
        {@const status = getStatusForPath(ext.path)}
        <div class="extension-item">
          <div class="extension-main">
            <div class="extension-info">
              <div class="extension-path" title={ext.path}>
                {ext.path.split("/").pop() || ext.path}
              </div>
              <div class="extension-path-full">{ext.path}</div>
              {#if status}
                <div class="extension-meta">
                  <span class="extension-name">{status.name}</span>
                  <span class="extension-version">v{status.version}</span>
                </div>
              {/if}
            </div>

            <div class="extension-controls">
              <div class="status-badge {getStateBadgeClass(status)}">
                {getStateLabel(status)}
              </div>
              <label class="toggle-wrapper">
                <input
                  type="checkbox"
                  checked={ext.enabled ?? true}
                  onchange={(e) =>
                    setExtensionEnabled(index, e.currentTarget.checked)}
                />
                <span class="toggle-slider"></span>
              </label>
              <Button
                variant="danger"
                iconOnly
                onclick={() => removeExtension(index)}
                title="Remove extension"
              >
                <IconDelete />
              </Button>
            </div>
          </div>

          {#if status && getExtensionError(status.state)}
            <div class="extension-error">
              <strong>Error:</strong>
              {getExtensionError(status.state)}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <div class="actions">
    <Button variant="secondary" onclick={() => (showLogsModal = true)}>
      View Logs
    </Button>
    <Button variant="secondary" onclick={handleReload} disabled={isReloading}>
      {isReloading ? "Reloading..." : "Reload Extensions"}
    </Button>
  </div>
</div>

{#if showLogsModal}
  <ExtensionLogsModal onclose={() => (showLogsModal = false)} />
{/if}

<style>
  .extensions-settings {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 0.5rem 0;
  }

  .header {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;

    h3 {
      margin: 0;
      font-size: 1.2rem;
      color: var(--col-text);
    }

    .description {
      margin: 0;
      font-size: 0.9rem;
      color: var(--col-subtle);
    }
  }

  .add-extension-form {
    display: flex;
    flex-direction: row;
    gap: 0.5rem;
    align-items: stretch;

    .command-input {
      flex: 1;
      padding: 0.5rem 0.75rem;
      border: 1px solid var(--col-highlightMed);
      border-radius: 4px;
      background: var(--col-surface);
      color: var(--col-text);
      font-size: 0.9rem;
      font-family: monospace;

      &:focus {
        outline: none;
        border-color: var(--col-iris);
        box-shadow: 0 0 0 2px var(--col-iris-alpha);
      }

      &::placeholder {
        color: var(--col-subtle);
      }
    }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 2rem;
    text-align: center;
    color: var(--col-subtle);

    p {
      margin: 0;
    }

    .hint {
      font-size: 0.875rem;
    }
  }

  .extension-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .extension-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
    border: 1px solid var(--col-highlightMed);
    border-radius: 4px;
    background: var(--col-surface);
  }

  .extension-main {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .extension-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    flex: 1;
    min-width: 0;
  }

  .extension-path {
    font-weight: 600;
    color: var(--col-text);
    font-size: 0.95rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .extension-path-full {
    font-size: 0.75rem;
    color: var(--col-subtle);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .extension-meta {
    display: flex;
    flex-direction: row;
    gap: 1ch;
    font-size: 0.8rem;
    color: var(--col-subtle);

    .extension-name {
      font-weight: 500;
      color: var(--col-text);
    }
  }

  .extension-controls {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 1rem;
  }

  .status-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;

    &.running {
      background: var(--col-foam);
      color: var(--col-base);
    }

    &.failed {
      background: var(--col-love);
      color: var(--col-base);
    }

    &.stopped {
      background: var(--col-highlightMed);
      color: var(--col-subtle);
    }

    &.unknown {
      background: var(--col-highlightLow);
      color: var(--col-subtle);
    }
  }


  .extension-error {
    padding: 0.5rem;
    background: var(--col-highlightLow);
    border-left: 3px solid var(--col-love);
    font-size: 0.85rem;
    color: var(--col-text);

    strong {
      color: var(--col-love);
    }
  }

  .actions {
    display: flex;
    flex-direction: row;
    gap: 0.75rem;
    padding-top: 0.5rem;
  }


  .toggle-wrapper {
    position: relative;
    display: inline-block;
    width: 3em;
    height: 1.75em;

    input[type="checkbox"] {
      opacity: 0;
      width: 0;
      height: 0;

      &:checked + .toggle-slider {
        background-color: var(--col-iris);
      }

      &:checked + .toggle-slider::before {
        transform: translateX(1.25em);
      }

      &:focus + .toggle-slider {
        box-shadow: 0 0 0 2px var(--col-iris);
      }
    }

    .toggle-slider {
      position: absolute;
      cursor: pointer;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background-color: var(--col-highlightMed);
      transition: 0.2s;
      border-radius: 1.75em;
      border: 1px solid var(--col-highlightHigh);

      &::before {
        position: absolute;
        content: "";
        height: 1.25em;
        width: 1.25em;
        left: 0.25em;
        bottom: 0.125em;
        background-color: var(--col-base);
        transition: 0.2s;
        border-radius: 50%;
      }

      &:hover {
        background-color: var(--col-highlightHigh);
      }
    }
  }

</style>
