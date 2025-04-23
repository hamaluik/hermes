<script lang="ts">
  import MessageEditor from "$lib/message_editor.svelte";
  import CursorBreadcrumbs from "$lib/cursor_breadcrumbs.svelte";

  let message: string | null = $state(null);
  let cursorPos: number | null = $state(null);
</script>

<main class="container">
  <MessageEditor
    --message-editor-flex="1"
    onchange={(m) => {
      message = m;
    }}
    oncursorchange={(pos) => {
      cursorPos = pos;
    }}
  />
  <CursorBreadcrumbs {message} {cursorPos} />
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
