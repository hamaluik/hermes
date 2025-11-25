<!--
  Diff Modal Component

  Modal dialog for comparing two HL7 messages side-by-side with semantic diff highlighting.

  Features:
  - Side-by-side message display with syntax highlighting
  - Multiple input sources: editor, file, clipboard, or direct paste
  - Diff highlighting at segment, field, and component levels
  - Clickable diff summary list to navigate to changes

  The diff comparison is performed by the Rust backend which understands HL7 structure
  and can identify changes at any level of the message hierarchy.
-->
<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";
  import Button from "$lib/components/button.svelte";
  import { compareMessages, type MessageDiff, type FieldDiff, type DiffType } from "./diff";
  import { type DiffMatch } from "$lib/editor/syntax_highlight";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { readTextFile } from "@tauri-apps/plugin-fs";
  import IconEditMessage from "$lib/icons/IconEditMessage.svelte";
  import IconOpen from "$lib/icons/IconOpen.svelte";
  import MessageEditor from "$lib/editor/message_editor.svelte";

  let {
    show = $bindable(false),
    editorMessage = "",
  }: {
    show: boolean;
    editorMessage?: string;
  } = $props();

  // Message content for left and right panes
  let leftMessage: string = $state("");
  let rightMessage: string = $state("");

  // Diff result
  let diff: MessageDiff | null = $state(null);
  let diffError: string | null = $state(null);
  let isComparing: boolean = $state(false);

  // Derived diff highlights for each side
  let leftDiffHighlights: DiffMatch[] = $derived(getDiffRangesForSide("left"));
  let rightDiffHighlights: DiffMatch[] = $derived(getDiffRangesForSide("right"));

  // Reset state when modal opens
  $effect(() => {
    if (show) {
      leftMessage = "";
      rightMessage = "";
      diff = null;
      diffError = null;
    }
  });

  // Determine the visual diff type based on the actual values
  // - Empty to non-empty = "added"
  // - Non-empty to empty = "removed"
  // - Non-empty to different non-empty = "modified"
  function getVisualDiffType(field: FieldDiff, side: "left" | "right"): "added" | "removed" | "modified" {
    // Check if values are empty (null, undefined, or empty string)
    const leftVal = field.left_value ?? "";
    const rightVal = field.right_value ?? "";
    const leftEmpty = leftVal.trim() === "";
    const rightEmpty = rightVal.trim() === "";

    if (leftEmpty && !rightEmpty) {
      // Value was added (empty -> something)
      return side === "right" ? "added" : "modified";
    } else if (!leftEmpty && rightEmpty) {
      // Value was removed (something -> empty)
      return side === "left" ? "removed" : "modified";
    } else {
      // Value was modified (something -> something different)
      return "modified";
    }
  }

  // Get diff ranges for highlighting in syntax highlight
  function getDiffRangesForSide(side: "left" | "right"): DiffMatch[] {
    if (!diff) return [];

    const ranges: DiffMatch[] = [];

    for (const segment of diff.segments) {
      // Add segment-level highlights for added/removed segments
      if (segment.diff_type === "added" && side === "right" && segment.right_range) {
        ranges.push({
          start: segment.right_range[0],
          end: segment.right_range[1],
          diff_type: "added",
        });
      } else if (segment.diff_type === "removed" && side === "left" && segment.left_range) {
        ranges.push({
          start: segment.left_range[0],
          end: segment.left_range[1],
          diff_type: "removed",
        });
      } else if (segment.diff_type === "modified") {
        // Add field-level highlights for modified segments
        for (const field of segment.fields) {
          if (field.diff_type === "unchanged") continue;

          if (side === "left" && field.left_range) {
            ranges.push({
              start: field.left_range[0],
              end: field.left_range[1],
              diff_type: getVisualDiffType(field, "left"),
            });
          } else if (side === "right" && field.right_range) {
            ranges.push({
              start: field.right_range[0],
              end: field.right_range[1],
              diff_type: getVisualDiffType(field, "right"),
            });
          }
        }
      }
    }

    return ranges;
  }

  async function handleCompare() {
    if (!leftMessage.trim() || !rightMessage.trim()) {
      diffError = "Both messages are required for comparison";
      return;
    }

    isComparing = true;
    diffError = null;

    try {
      diff = await compareMessages(leftMessage, rightMessage);
      // Diff highlights are now computed as derived state from diff
    } catch (e) {
      diffError = String(e);
      diff = null;
    } finally {
      isComparing = false;
    }
  }

  async function loadFromEditor(side: "left" | "right") {
    if (side === "left") {
      leftMessage = editorMessage;
    } else {
      rightMessage = editorMessage;
    }
    diff = null;
  }

  async function loadFromFile(side: "left" | "right") {
    try {
      const result = await openDialog({
        multiple: false,
        filters: [{ name: "HL7 Files", extensions: ["hl7", "txt"] }],
      });

      if (result) {
        const content = await readTextFile(result);
        if (side === "left") {
          leftMessage = content;
        } else {
          rightMessage = content;
        }
        diff = null;
      }
    } catch (e) {
      console.error("Error loading file:", e);
    }
  }

  function handleClose() {
    show = false;
  }

  function getDiffTypeIcon(diffType: "added" | "removed" | "modified"): string {
    switch (diffType) {
      case "added": return "+";
      case "removed": return "-";
      case "modified": return "~";
      default: return "";
    }
  }

  function getDiffTypeClass(diffType: "added" | "removed" | "modified"): string {
    switch (diffType) {
      case "added": return "diff-added";
      case "removed": return "diff-removed";
      case "modified": return "diff-modified";
      default: return "";
    }
  }

  // Get the visual diff type for a field (for display in the list)
  function getFieldVisualDiffType(field: FieldDiff): "added" | "removed" | "modified" {
    const leftVal = field.left_value ?? "";
    const rightVal = field.right_value ?? "";
    const leftEmpty = leftVal.trim() === "";
    const rightEmpty = rightVal.trim() === "";

    if (leftEmpty && !rightEmpty) {
      return "added";
    } else if (!leftEmpty && rightEmpty) {
      return "removed";
    } else {
      return "modified";
    }
  }

  // Get all field diffs that represent actual changes
  function getChangedFields(): FieldDiff[] {
    if (!diff) return [];

    const changes: FieldDiff[] = [];
    for (const segment of diff.segments) {
      for (const field of segment.fields) {
        if (field.diff_type !== "unchanged") {
          changes.push(field);
        }
      }
    }
    return changes;
  }
</script>

<Modal bind:show maxWidth="95vw" maxHeight="90vh" height="80vh" width="90vw">
  <ModalHeader onclose={handleClose}>Compare Messages</ModalHeader>

  <main>
    <div class="panes">
      <!-- Left pane -->
      <div class="pane">
        <div class="pane-header">
          <span class="pane-title">Left (Original)</span>
          <div class="pane-actions">
            <Button variant="ghost" onclick={() => loadFromEditor("left")} disabled={!editorMessage} title="Load from Editor">
              <IconEditMessage />
              <span>Editor</span>
            </Button>
            <Button variant="ghost" onclick={() => loadFromFile("left")} title="Open File...">
              <IconOpen />
              <span>Open</span>
            </Button>
          </div>
        </div>
        <div class="message-container">
          <MessageEditor
            bind:message={leftMessage}
            diffHighlights={leftDiffHighlights}
            placeholder="Paste or load a message..."
            onchange={() => { diff = null; }}
          />
        </div>
      </div>

      <!-- Right pane -->
      <div class="pane">
        <div class="pane-header">
          <span class="pane-title">Right (Modified)</span>
          <div class="pane-actions">
            <Button variant="ghost" onclick={() => loadFromEditor("right")} disabled={!editorMessage} title="Load from Editor">
              <IconEditMessage />
              <span>Editor</span>
            </Button>
            <Button variant="ghost" onclick={() => loadFromFile("right")} title="Open File...">
              <IconOpen />
              <span>Open</span>
            </Button>
          </div>
        </div>
        <div class="message-container">
          <MessageEditor
            bind:message={rightMessage}
            diffHighlights={rightDiffHighlights}
            placeholder="Paste or load a message..."
            onchange={() => { diff = null; }}
          />
        </div>
      </div>
    </div>

    <div class="compare-section">
      <Button
        variant="primary"
        onclick={handleCompare}
        disabled={isComparing || !leftMessage.trim() || !rightMessage.trim()}
      >
        {isComparing ? "Comparing..." : "Compare"}
      </Button>
    </div>

    {#if diffError}
      <div class="error">{diffError}</div>
    {/if}

    {#if diff}
      <div class="diff-summary">
        <div class="diff-header">
          <span class="diff-title">
            Differences ({diff.summary.total_field_changes})
          </span>
          <span class="diff-stats">
            {#if diff.summary.segments_added > 0}
              <span class="stat added">+{diff.summary.segments_added} segments</span>
            {/if}
            {#if diff.summary.segments_removed > 0}
              <span class="stat removed">-{diff.summary.segments_removed} segments</span>
            {/if}
            {#if diff.summary.segments_modified > 0}
              <span class="stat modified">~{diff.summary.segments_modified} segments</span>
            {/if}
          </span>
        </div>

        <div class="diff-list">
          {#if diff.summary.total_field_changes === 0}
            <div class="no-changes">Messages are identical</div>
          {:else}
            {#each diff.segments as segment}
              {#if segment.diff_type === "added"}
                <div class="diff-item {getDiffTypeClass("added")}">
                  <span class="diff-icon">{getDiffTypeIcon("added")}</span>
                  <span class="diff-path">{segment.name}{segment.occurrence > 0 ? `[${segment.occurrence + 1}]` : ""}</span>
                  <span class="diff-desc">segment added</span>
                </div>
              {:else if segment.diff_type === "removed"}
                <div class="diff-item {getDiffTypeClass("removed")}">
                  <span class="diff-icon">{getDiffTypeIcon("removed")}</span>
                  <span class="diff-path">{segment.name}{segment.occurrence > 0 ? `[${segment.occurrence + 1}]` : ""}</span>
                  <span class="diff-desc">segment removed</span>
                </div>
              {:else if segment.diff_type === "modified"}
                {#each segment.fields as field}
                  {#if field.diff_type !== "unchanged"}
                    {@const visualType = getFieldVisualDiffType(field)}
                    <div class="diff-item {getDiffTypeClass(visualType)}">
                      <span class="diff-icon">{getDiffTypeIcon(visualType)}</span>
                      <span class="diff-path">{field.path}</span>
                      {#if visualType === "modified"}
                        <span class="diff-values">
                          <span class="old-value">"{field.left_value ?? ""}"</span>
                          <span class="arrow">-></span>
                          <span class="new-value">"{field.right_value ?? ""}"</span>
                        </span>
                      {:else if visualType === "added"}
                        <span class="diff-values">
                          <span class="new-value">"{field.right_value ?? ""}"</span>
                        </span>
                      {:else if visualType === "removed"}
                        <span class="diff-values">
                          <span class="old-value">"{field.left_value ?? ""}"</span>
                        </span>
                      {/if}
                    </div>
                  {/if}
                {/each}
              {/if}
            {/each}
          {/if}
        </div>
      </div>
    {/if}
  </main>
</Modal>

<style>
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 0.5rem 1rem 1rem;
    gap: 0.5rem;
    overflow: hidden;
    min-height: 0;
  }

  .panes {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    flex: 1;
    min-height: 200px;
  }

  .pane {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border: 1px solid var(--col-highlightMed);
    border-radius: 4px;
    overflow: hidden;
  }

  .pane-header {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem;
    background: var(--col-surface);
    border-bottom: 1px solid var(--col-highlightMed);
    flex-shrink: 0;
  }

  .pane-title {
    font-weight: 600;
    font-size: 0.9rem;
  }

  .pane-actions {
    display: flex;
    gap: 0.5rem;
  }


  .message-container {
    flex: 1;
    display: flex;
    min-height: 0;

    /* Override MessageEditor's default styling for diff view */
    :global(.message-editor) {
      border: none;
      border-radius: 0;
      --message-editor-flex: 1;
    }
  }

  .compare-section {
    display: flex;
    justify-content: center;
    padding: 0.5rem 0;
  }

  .error {
    color: var(--col-love);
    background: var(--col-highlightLow);
    padding: 0.5rem;
    border-radius: 4px;
    font-size: 0.85rem;
  }

  .diff-summary {
    border: 1px solid var(--col-highlightMed);
    border-radius: 4px;
    overflow: hidden;
    max-height: 200px;
    display: flex;
    flex-direction: column;
  }

  .diff-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem;
    background: var(--col-surface);
    border-bottom: 1px solid var(--col-highlightMed);
    flex-shrink: 0;
  }

  .diff-title {
    font-weight: 600;
    font-size: 0.9rem;
  }

  .diff-stats {
    display: flex;
    gap: 0.5rem;
    font-size: 0.8rem;
  }

  .stat.added {
    color: var(--col-foam);
  }

  .stat.removed {
    color: var(--col-love);
  }

  .stat.modified {
    color: var(--col-gold);
  }

  .diff-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.25rem;
    user-select: text;
    -webkit-user-select: text;
  }

  .no-changes {
    padding: 1rem;
    text-align: center;
    color: var(--col-muted);
    font-style: italic;
  }

  .diff-item {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.25rem 0.5rem;
    font-size: 0.8rem;
    font-family: monospace;
    border-radius: 2px;

    &:hover {
      background: var(--col-highlightLow);
    }
  }

  .diff-icon {
    font-weight: bold;
    width: 1rem;
    text-align: center;
  }

  .diff-item.diff-added .diff-icon {
    color: var(--col-foam);
  }

  .diff-item.diff-removed .diff-icon {
    color: var(--col-love);
  }

  .diff-item.diff-modified .diff-icon {
    color: var(--col-gold);
  }

  .diff-path {
    font-weight: 600;
    color: var(--col-iris);
    min-width: 8ch;
  }

  .diff-desc {
    color: var(--col-muted);
    font-style: italic;
  }

  .diff-values {
    display: flex;
    gap: 0.5ch;
    align-items: baseline;
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .old-value {
    color: var(--col-love);
    text-decoration: line-through;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 20ch;
  }

  .new-value {
    color: var(--col-foam);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 20ch;
  }

  .arrow {
    color: var(--col-muted);
    flex-shrink: 0;
  }
</style>
