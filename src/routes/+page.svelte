<script lang="ts">
  import MessageEditor from "$lib/message_editor.svelte";
</script>

<main class="container">
  <h1>Hermes</h1>

  <MessageEditor --message-editor-flex="1" />
</main>

<style>
  .container {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
    padding: 1rem;
    min-height: calc(100vh - 2rem);
  }
</style>
