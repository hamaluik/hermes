<!--
  Find/Replace Bar Component

  A bar for searching and replacing text in the HL7 message editor.

  Features:
  - Find text with match highlighting
  - Case-sensitive toggle
  - Previous/Next navigation
  - Replace single or all matches
  - Auto-populates from selection when opened
  - Keyboard navigation (Enter for next, Shift+Enter for previous)
  - Escape to close
-->
<script lang="ts">
  import IconChevronUp from "./icons/IconChevronUp.svelte";
  import IconChevronDown from "./icons/IconChevronDown.svelte";
  import IconCase from "./icons/IconCase.svelte";
  import IconClose from "./icons/IconClose.svelte";
  import IconReplace from "./icons/IconReplace.svelte";
  import IconReplaceAll from "./icons/IconReplaceAll.svelte";
  import type { SearchMatch } from "../backend/syntax_highlight";

  let {
    show = $bindable(false),
    message,
    initialSelection = "",
    onmatcheschange,
    onnavigate,
    onreplace,
    onclose,
  }: {
    show: boolean;
    message: string;
    initialSelection?: string;
    onmatcheschange?: (matches: SearchMatch[], currentIndex: number) => void;
    onnavigate?: () => void;
    onreplace?: (newMessage: string) => void;
    onclose?: () => void;
  } = $props();

  let searchInput: HTMLInputElement;
  let replaceInput: HTMLInputElement;
  let query = $state("");
  let replacement = $state("");
  let caseSensitive = $state(false);
  let matches: SearchMatch[] = $state([]);
  let currentMatchIndex = $state(0);
  let searchTimeout: ReturnType<typeof setTimeout> | undefined;

  // Find all matches when query or message changes (debounced for typing performance)
  $effect(() => {
    // Clear any pending search
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    if (!query || !message) {
      matches = [];
      currentMatchIndex = 0;
      onmatcheschange?.([], 0);
      return;
    }

    // Capture current values for the timeout callback
    const currentQuery = query;
    const currentMessage = message;
    const currentCaseSensitive = caseSensitive;

    // Debounce the search by 50ms to avoid lag while typing
    searchTimeout = setTimeout(() => {
      const newMatches: SearchMatch[] = [];
      const searchQuery = currentCaseSensitive
        ? currentQuery
        : currentQuery.toLowerCase();
      const searchText = currentCaseSensitive
        ? currentMessage
        : currentMessage.toLowerCase();

      let startIndex = 0;
      while (true) {
        const index = searchText.indexOf(searchQuery, startIndex);
        if (index === -1) break;
        newMatches.push({ start: index, end: index + currentQuery.length });
        startIndex = index + 1;
      }

      matches = newMatches;

      // Keep current index valid
      if (currentMatchIndex >= matches.length) {
        currentMatchIndex = matches.length > 0 ? 0 : 0;
      }

      onmatcheschange?.(matches, currentMatchIndex);
    }, 50);
  });

  // Focus search input and populate with selection when shown
  $effect(() => {
    if (show && searchInput) {
      if (initialSelection) {
        query = initialSelection;
      }
      // Use setTimeout to ensure DOM is ready
      setTimeout(() => {
        searchInput?.focus();
        searchInput?.select();
      }, 0);
    }
  });

  function goToNextMatch() {
    if (matches.length === 0) return;
    currentMatchIndex = (currentMatchIndex + 1) % matches.length;
    onmatcheschange?.(matches, currentMatchIndex);
    onnavigate?.();
  }

  function goToPreviousMatch() {
    if (matches.length === 0) return;
    currentMatchIndex =
      (currentMatchIndex - 1 + matches.length) % matches.length;
    onmatcheschange?.(matches, currentMatchIndex);
    onnavigate?.();
  }

  function replaceCurrentMatch() {
    if (matches.length === 0 || !onreplace) return;

    const match = matches[currentMatchIndex];
    const newMessage =
      message.slice(0, match.start) + replacement + message.slice(match.end);

    onreplace(newMessage);
  }

  function replaceAllMatches() {
    if (matches.length === 0 || !onreplace) return;

    // Replace from end to start to preserve indices
    let newMessage = message;
    for (let i = matches.length - 1; i >= 0; i--) {
      const match = matches[i];
      newMessage =
        newMessage.slice(0, match.start) +
        replacement +
        newMessage.slice(match.end);
    }

    onreplace(newMessage);
  }

  function handleSearchKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      if (event.shiftKey) {
        goToPreviousMatch();
      } else {
        goToNextMatch();
      }
    } else if (event.key === "Escape") {
      event.preventDefault();
      handleClose();
    }
  }

  function handleReplaceKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      replaceCurrentMatch();
    } else if (event.key === "Escape") {
      event.preventDefault();
      handleClose();
    }
  }

  function handleClose() {
    show = false;
    query = "";
    replacement = "";
    matches = [];
    currentMatchIndex = 0;
    onmatcheschange?.([], 0);
    onclose?.();
  }
</script>

{#if show}
  <div class="find-replace-bar" role="search" aria-label="Find and Replace">
    <div class="search-field">
      <input
        type="text"
        bind:this={searchInput}
        bind:value={query}
        onkeydown={handleSearchKeyDown}
        placeholder="Find"
        aria-label="Search query"
      />
      <button
        class="toggle-button"
        class:active={caseSensitive}
        onclick={() => (caseSensitive = !caseSensitive)}
        title="Match Case"
        aria-pressed={caseSensitive}
      >
        <IconCase />
      </button>
    </div>

    <div class="nav-buttons">
      <button
        onclick={goToPreviousMatch}
        disabled={matches.length === 0}
        title="Previous Match (Shift+Enter)"
        aria-label="Previous match"
      >
        <IconChevronUp />
      </button>
      <button
        onclick={goToNextMatch}
        disabled={matches.length === 0}
        title="Next Match (Enter)"
        aria-label="Next match"
      >
        <IconChevronDown />
      </button>
    </div>

    <span class="match-count" aria-live="polite">
      {#if matches.length > 0}
        {currentMatchIndex + 1} of {matches.length}
      {:else if query}
        No results
      {/if}
    </span>

    <div class="replace-field">
      <input
        type="text"
        bind:this={replaceInput}
        bind:value={replacement}
        onkeydown={handleReplaceKeyDown}
        placeholder="Replace"
        aria-label="Replacement text"
      />
    </div>

    <div class="replace-buttons">
      <button
        onclick={replaceCurrentMatch}
        disabled={matches.length === 0}
        title="Replace"
        aria-label="Replace current match"
      >
        <IconReplace />
      </button>
      <button
        onclick={replaceAllMatches}
        disabled={matches.length === 0}
        title="Replace All"
        aria-label="Replace all matches"
      >
        <IconReplaceAll />
      </button>
    </div>

    <button class="close-button" onclick={handleClose} title="Close (Escape)">
      <IconClose />
    </button>
  </div>
{/if}

<style>
  .find-replace-bar {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;

    background-color: var(--col-surface);
    border: 1px solid var(--col-highlightHigh);
    border-radius: 4px;
    padding: 0.5rem;
  }

  .search-field,
  .replace-field {
    flex: 1;
    display: flex;
    align-items: center;
    background-color: var(--col-base);
    border: 1px solid var(--col-highlightHigh);
    border-radius: 4px;
    overflow: hidden;
  }

  .search-field:focus-within,
  .replace-field:focus-within {
    border-color: var(--col-iris);
    box-shadow: 0 0 0 1px var(--col-iris);
  }

  .search-field input,
  .replace-field input {
    flex: 1;
    min-width: 0;
    padding: 0.25rem 0.5rem;
    border: none;
    background: transparent;
    color: var(--col-text);
    font-size: 0.875rem;
  }

  .search-field input:focus,
  .replace-field input:focus {
    outline: none;
    box-shadow: none;
  }

  .toggle-button {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.25rem;
    background: transparent;
    border: none;
    color: var(--col-muted);
    cursor: pointer;
  }

  .toggle-button:hover {
    color: var(--col-text);
  }

  .toggle-button.active {
    color: var(--col-iris);
  }

  .match-count {
    font-size: 0.75rem;
    color: var(--col-subtle);
    white-space: nowrap;
  }

  .nav-buttons,
  .replace-buttons {
    display: flex;
    gap: 0.125rem;
  }

  .nav-buttons button,
  .replace-buttons button,
  .close-button {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.25rem;
    background: transparent;
    border: none;
    color: var(--col-subtle);
    cursor: pointer;
    border-radius: 2px;
  }

  .nav-buttons button:hover:not(:disabled),
  .replace-buttons button:hover:not(:disabled),
  .close-button:hover {
    color: var(--col-text);
    background-color: var(--col-highlightLow);
  }

  .nav-buttons button:disabled,
  .replace-buttons button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .close-button {
    margin-left: auto;
  }

  /* Icon sizing */
  .find-replace-bar :global(svg) {
    width: 16px;
    height: 16px;
  }
</style>
