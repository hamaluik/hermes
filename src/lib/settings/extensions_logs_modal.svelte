<!--
  Extensions Logs Modal

  Displays detailed log entries from all extensions. Allows users to:
  - View logs from all extensions or filter to a specific extension
  - See timestamps, log levels (info/warn/error), and messages
  - Copy logs to clipboard for debugging
  - Auto-scroll to the bottom for real-time monitoring
-->
<script lang="ts">
  import { onMount } from "svelte";
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";
  import ModalFooter from "$lib/components/modal_footer.svelte";
  import Button from "$lib/components/button.svelte";
  import {
    getExtensions,
    getExtensionLogs,
    type ExtensionStatus,
    type ExtensionLog,
  } from "$lib/extensions/extensions";

  let {
    onclose,
  }: {
    onclose: () => void;
  } = $props();

  // modal visibility state for binding to Modal component
  let showModal: boolean = $state(true);

  // watch for showModal changes and trigger onclose when false
  $effect(() => {
    if (!showModal) {
      onclose();
    }
  });

  // extension list for filtering
  let extensions: ExtensionStatus[] = $state([]);
  let selectedExtensionId: string = $state("all");

  // logs from all extensions
  let allLogs: Array<ExtensionLog & { extensionId: string; extensionName: string }> = $state([]);
  let isLoading: boolean = $state(true);

  // filtered logs based on selection
  let filteredLogs = $derived(
    selectedExtensionId === "all"
      ? allLogs
      : allLogs.filter((log) => log.extensionId === selectedExtensionId),
  );

  async function loadLogs() {
    isLoading = true;
    try {
      extensions = await getExtensions();

      // fetch logs from each extension
      const logsPromises = extensions.map(async (ext) => {
        try {
          const logs = await getExtensionLogs(ext.id);
          return logs.map((log) => ({
            ...log,
            extensionId: ext.id,
            extensionName: ext.name,
          }));
        } catch (error) {
          console.error(`failed to load logs for ${ext.id}:`, error);
          return [];
        }
      });

      const logsArrays = await Promise.all(logsPromises);
      allLogs = logsArrays.flat();

      // sort by timestamp (newest first)
      allLogs.sort((a, b) => {
        const dateA = new Date(a.timestamp).getTime();
        const dateB = new Date(b.timestamp).getTime();
        return dateB - dateA;
      });
    } catch (error) {
      console.error("failed to load extensions:", error);
    } finally {
      isLoading = false;
    }
  }

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString();
  }

  function getLevelClass(level: string): string {
    switch (level) {
      case "info":
        return "level-info";
      case "warn":
        return "level-warn";
      case "error":
        return "level-error";
      default:
        return "";
    }
  }

  function copyLogsToClipboard() {
    const text = filteredLogs
      .map(
        (log) =>
          `[${formatTimestamp(log.timestamp)}] [${log.level.toUpperCase()}] ${log.extensionName}: ${log.message}`,
      )
      .join("\n");

    navigator.clipboard.writeText(text).catch((err) => {
      console.error("failed to copy logs:", err);
    });
  }

  onMount(() => {
    loadLogs();
  });
</script>

{#snippet modalContent()}
  <ModalHeader {onclose}>
    Extension Logs
  </ModalHeader>

  <div class="logs-content">
    <div class="logs-controls">
      <select bind:value={selectedExtensionId}>
        <option value="all">All Extensions</option>
        {#each extensions as ext}
          <option value={ext.id}>{ext.name}</option>
        {/each}
      </select>

      <Button variant="ghost" onclick={copyLogsToClipboard}>
        Copy to Clipboard
      </Button>

      <Button variant="ghost" onclick={loadLogs}>
        Refresh
      </Button>
    </div>

    <div class="logs-container">
      {#if isLoading}
        <div class="loading">Loading logs...</div>
      {:else if filteredLogs.length === 0}
        <div class="empty">No logs available</div>
      {:else}
        <div class="logs-list">
          {#each filteredLogs as log}
            <div class="log-entry {getLevelClass(log.level)}">
              <span class="log-timestamp">{formatTimestamp(log.timestamp)}</span>
              <span class="log-level">[{log.level.toUpperCase()}]</span>
              <span class="log-extension">{log.extensionName}:</span>
              <span class="log-message">{log.message}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <ModalFooter>
    {#snippet right()}
      <Button variant="ghost" onclick={onclose}>Close</Button>
    {/snippet}
  </ModalFooter>
{/snippet}

<Modal bind:show={showModal}>
  {@render modalContent()}
</Modal>

<style>
  .logs-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-height: 400px;
    max-height: 600px;
  }

  .logs-controls {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .logs-controls select {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid var(--col-highlightMed);
    border-radius: 4px;
    background: var(--col-surface);
    color: var(--col-text);
    font-family: monospace;
    font-size: 0.9rem;
  }


  .logs-container {
    flex: 1;
    overflow-y: auto;
    border: 1px solid var(--col-highlightMed);
    border-radius: 4px;
    background: var(--col-surface);
  }

  .loading,
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--col-subtle);
    font-style: italic;
  }

  .logs-list {
    display: flex;
    flex-direction: column;
  }

  .log-entry {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem;
    border-bottom: 1px solid var(--col-highlightMed);
    font-family: monospace;
    font-size: 0.85rem;
  }

  .log-entry:last-child {
    border-bottom: none;
  }

  .log-timestamp {
    color: var(--col-subtle);
    flex-shrink: 0;
  }

  .log-level {
    flex-shrink: 0;
    font-weight: bold;
  }

  .log-extension {
    flex-shrink: 0;
    color: var(--col-iris);
  }

  .log-message {
    flex: 1;
    word-break: break-word;
  }

  .level-info .log-level {
    color: var(--col-foam);
  }

  .level-warn .log-level {
    color: var(--col-gold);
  }

  .level-error .log-level {
    color: var(--col-love);
  }
</style>
