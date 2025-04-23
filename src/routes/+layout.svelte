<script>
  let { children } = $props();
</script>

<main>
  {@render children?.()}
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: center;
    padding: 1rem;
  }
</style>
