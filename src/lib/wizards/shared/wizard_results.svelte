<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    header,
    body,
  }: {
    header: Snippet<[]>;
    body: Snippet<[]>;
  } = $props();
</script>

<div class="results">
  <table>
    <thead>
      {@render header()}
    </thead>
    <tbody>
      {@render body()}
    </tbody>
  </table>
</div>

<style>
  .results {
    margin-top: 1rem;

    table {
      width: 100%;
      border-collapse: collapse;

      :global(th),
      :global(td) {
        padding: 0.5em;
        text-align: left;
        border-bottom: 1px solid var(--col-highlightMed);
      }

      :global(th) {
        background: var(--col-surface);
        font-weight: 600;
      }

      :global(tbody tr) {
        cursor: pointer;
        transition: background-color 0.2s;

        &:hover {
          background: var(--col-love);
          color: var(--col-base);
        }

        &.selected {
          background: var(--col-gold);
          color: var(--col-base);
        }
      }
    }
  }
</style>
