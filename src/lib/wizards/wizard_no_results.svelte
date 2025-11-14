<script lang="ts">
  let {
    message = "No results found.",
  }: {
    message?: string;
  } = $props();
</script>

<div class="no-results">
  <p>{message}</p>
</div>

<style>
  .no-results {
    margin-top: 1rem;
    padding: 1rem;
    text-align: center;
    color: var(--col-subtle);
    background: var(--col-highlightLow);
    border-radius: 4px;
  }
</style>
