<script lang="ts">
  let {
    message = "Loading...",
  }: {
    message?: string;
  } = $props();
</script>

<div class="loading">
  <div class="spinner"></div>
  <p>{message}</p>
</div>

<style>
  .loading {
    margin-top: 1rem;
    padding: 1.5rem;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;

    p {
      color: var(--col-text);
      font-size: 0.95em;
    }
  }

  .spinner {
    width: 2.5em;
    height: 2.5em;
    border: 3px solid var(--col-highlightMed);
    border-top: 3px solid var(--col-iris);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>
