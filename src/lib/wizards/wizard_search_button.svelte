<script lang="ts">
  import IconSearch from "$lib/icons/IconSearch.svelte";

  let {
    disabled = false,
    title,
  }: {
    disabled?: boolean;
    title?: string;
  } = $props();
</script>

<button type="submit" class="search-button" {disabled} {title}>
  <IconSearch />
</button>

<style>
  .search-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 3.5em;
    height: 3.5em;
    padding: 0;
    background: var(--col-iris);
    color: var(--col-base);
    border: 1px solid var(--col-highlightHigh);
    border-radius: 4px;
    cursor: pointer;
    font-size: 1em;

    &:hover:not(:disabled) {
      background: var(--col-love);
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }

    :global(svg) {
      width: 1.5em;
      height: 1.5em;
    }
  }
</style>
