<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    left,
    right,
  }: {
    left?: Snippet<[]>;
    right?: Snippet<[]>;
  } = $props();
</script>

<footer>
  <div class="left">
    {#if left}
      {@render left()}
    {/if}
  </div>
  <div class="right">
    {#if right}
      {@render right()}
    {/if}
  </div>
</footer>

<style>
  footer {
    width: 100%;
    border-radius: 0 0 8px 8px;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    background: none;

    .left,
    .right {
      display: flex;
      flex-direction: row;
      align-items: center;
      gap: 0.5rem;
    }

    .left {
      justify-content: flex-start;
    }

    .right {
      justify-content: flex-end;
    }

    :global(button) {
      padding: 0.5em 1.5em;
      border-radius: 4px;
      cursor: pointer;
      font-size: 0.9em;
      border: 1px solid var(--col-highlightHigh);
      transition: all 0.2s;

      &:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }
    }

    :global(button.cancel) {
      background: transparent;
      color: var(--col-text);

      &:hover:not(:disabled) {
        background: var(--col-highlightLow);
      }
    }

    :global(button.apply),
    :global(button.save) {
      background: var(--col-iris);
      color: var(--col-base);

      &:hover:not(:disabled) {
        background: var(--col-love);
      }
    }
  }
</style>
