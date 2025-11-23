<!--
  Validation Panel Component

  Collapsible panel displaying validation issues for an HL7 message.
  Shows errors, warnings, and info messages with clickable items to
  navigate to the problematic field in the message editor.
-->
<script lang="ts">
  import type { ValidationResult, ValidationIssue, Severity } from "./validate";

  let {
    result = null,
    show = $bindable(false),
    onNavigate = () => {},
  }: {
    result: ValidationResult | null;
    show: boolean;
    onNavigate?: (issue: ValidationIssue) => void;
  } = $props();

  // derived state for display
  let hasIssues = $derived(result !== null && result.issues.length > 0);

  function getSeverityIcon(severity: Severity): string {
    switch (severity) {
      case "error": return "!";
      case "warning": return "?";
      case "info": return "i";
    }
  }

  function getSeverityClass(severity: Severity): string {
    switch (severity) {
      case "error": return "severity-error";
      case "warning": return "severity-warning";
      case "info": return "severity-info";
    }
  }

  function handleIssueClick(issue: ValidationIssue) {
    onNavigate(issue);
  }

  function togglePanel() {
    show = !show;
  }
</script>

{#if result}
  <div class="validation-panel" class:expanded={show}>
    <button type="button" class="panel-header" onclick={togglePanel}>
      <span class="panel-title">
        <span class="expand-icon">{show ? "▼" : "▶"}</span>
        Validation
      </span>
      <span class="panel-summary">
        {#if result.summary.errors > 0}
          <span class="count count-error">{result.summary.errors} error{result.summary.errors !== 1 ? "s" : ""}</span>
        {/if}
        {#if result.summary.warnings > 0}
          <span class="count count-warning">{result.summary.warnings} warning{result.summary.warnings !== 1 ? "s" : ""}</span>
        {/if}
        {#if result.summary.info > 0}
          <span class="count count-info">{result.summary.info} info</span>
        {/if}
        {#if result.issues.length === 0}
          <span class="count count-success">No issues</span>
        {/if}
      </span>
    </button>

    {#if show}
      <div class="panel-content">
        {#if result.issues.length === 0}
          <div class="no-issues">Message is valid</div>
        {:else}
          <div class="issues-list">
            {#each result.issues as issue}
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="issue-item {getSeverityClass(issue.severity)}"
                class:navigable={issue.range !== null}
                class:disabled={issue.range === null}
                ondblclick={() => handleIssueClick(issue)}
                title={issue.range === null ? "Cannot navigate - segment not present in message" : issue.actual_value ? `Value: "${issue.actual_value}"` : "Double-click to navigate"}
              >
                <span class="issue-icon">{getSeverityIcon(issue.severity)}</span>
                <span class="issue-path">{issue.path || "Message"}</span>
                <span class="issue-message">{issue.message}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .validation-panel {
    border: 1px solid var(--col-highlightMed);
    border-radius: 4px;
    overflow: hidden;
    background: var(--col-surface);
    font-size: 0.85rem;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.4rem 0.6rem;
    background: var(--col-surface);
    border: none;
    cursor: pointer;
    font-size: inherit;
    font-family: inherit;
    color: var(--col-text);
    text-align: left;
  }

  .panel-header:hover {
    background: var(--col-highlightLow);
  }

  .panel-title {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-weight: 600;
  }

  .expand-icon {
    font-size: 0.7rem;
    width: 1rem;
    text-align: center;
    color: var(--col-muted);
  }

  .panel-summary {
    display: flex;
    gap: 0.5rem;
    font-size: 0.8rem;
  }

  .count {
    padding: 0.1rem 0.4rem;
    border-radius: 3px;
    font-weight: 500;
  }

  .count-error {
    background: color-mix(in srgb, var(--col-love) 20%, transparent);
    color: var(--col-love);
  }

  .count-warning {
    background: color-mix(in srgb, var(--col-gold) 20%, transparent);
    color: var(--col-gold);
  }

  .count-info {
    background: color-mix(in srgb, var(--col-foam) 20%, transparent);
    color: var(--col-foam);
  }

  .count-success {
    background: color-mix(in srgb, var(--col-pine) 20%, transparent);
    color: var(--col-pine);
  }

  .panel-content {
    border-top: 1px solid var(--col-highlightMed);
    max-height: 200px;
    overflow-y: auto;
  }

  .no-issues {
    padding: 0.75rem;
    text-align: center;
    color: var(--col-pine);
    font-style: italic;
  }

  .issues-list {
    display: flex;
    flex-direction: column;
  }

  .issue-item {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.35rem 0.6rem;
    font-family: monospace;
    font-size: 0.8rem;
    text-align: left;
    background: none;
    border: none;
    border-bottom: 1px solid var(--col-highlightLow);
    color: var(--col-text);
    width: 100%;
    user-select: text;
    -webkit-user-select: text;
  }

  .issue-item.navigable {
    cursor: text;
  }

  .issue-item.disabled {
    cursor: text;
    opacity: 0.7;
  }

  .issue-item:last-child {
    border-bottom: none;
  }

  .issue-item.navigable:hover {
    background: var(--col-highlightLow);
  }

  .issue-icon {
    font-weight: bold;
    width: 1.2rem;
    height: 1.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    font-size: 0.7rem;
    flex-shrink: 0;
  }

  .severity-error .issue-icon {
    background: var(--col-love);
    color: var(--col-base);
  }

  .severity-warning .issue-icon {
    background: var(--col-gold);
    color: var(--col-base);
  }

  .severity-info .issue-icon {
    background: var(--col-foam);
    color: var(--col-base);
  }

  .issue-path {
    font-weight: 600;
    color: var(--col-iris);
    min-width: 8ch;
    flex-shrink: 0;
  }

  .issue-message {
    color: var(--col-subtle);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
