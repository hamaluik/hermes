<script lang="ts">
  import { goto } from "$app/navigation";
  import IconEditMessage from "$lib/icons/IconEditMessage.svelte";
  import IconHelp from "$lib/icons/IconHelp.svelte";
  import Toolbar from "$lib/toolbar.svelte";
  import ToolbarButton from "$lib/toolbar_button.svelte";
  import ToolbarSpacer from "$lib/toolbar_spacer.svelte";
  import { get } from "svelte/store";
  import type { PageProps } from "./$types";
  import { onMount } from "svelte";
  import MessageEditor from "$lib/message_editor.svelte";

  let { data }: PageProps = $props();

  let toolbarHeight: string | undefined = $state(undefined);
  let message: string = $state("");

  onMount(() => {
    message = get(data.message);
  });
</script>

<Toolbar bind:toolbarHeight>
  <ToolbarButton
    title="Edit Essage"
    onclick={() => {
      if (!document.startViewTransition) {
        goto("/");
        return;
      }
      document.startViewTransition(() => {
        goto("/");
      });
    }}
  >
    <IconEditMessage />
  </ToolbarButton>
  <ToolbarSpacer />
  <ToolbarButton title="Help">
    <IconHelp />
  </ToolbarButton>
</Toolbar>
<main class="main" style="--toolbar-height: {toolbarHeight ?? '1px'}">
  <div class="send-receive">
    <h1>Send/Receive</h1>
    <MessageEditor {message} readonly={true} />
  </div>
  <div class="listen">
    <h1>Listen</h1>
  </div>
</main>

<style>
  .main {
    display: flex;
    min-height: calc(100vh - var(--toolbar-height, 0px));

    flex-direction: column;
    @media (min-aspect-ratio: 5/4) {
      flex-direction: row;
    }

    > * {
      flex: 1;
    }
  }
</style>
