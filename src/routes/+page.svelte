<script lang="ts">
  import MessageEditor from "$lib/message_editor.svelte";
  import CursorDescription from "$lib/cursor_description.svelte";
  import PatientTab from "$lib/forms/patient_tab.svelte";

  let message: string = $state("");
  let cursorPos: number = $state(0);
</script>

<main>
  <PatientTab
    {message}
    onchange={(m) => {
      message = m;
    }}
  />

  <MessageEditor
    --message-editor-flex="1"
    {message}
    onchange={(m) => {
      message = m;
    }}
    oncursorchange={(pos) => {
      cursorPos = pos;
    }}
  />
  <CursorDescription {message} {cursorPos} />
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
    gap: 1rem;

    padding: 1rem;
    min-height: 100vh;
  }
</style>
