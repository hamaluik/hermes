<!--
  Listen Modal

  Modal dialog for managing the HL7 MLLP listen server. When active, the application
  listens on a configured port for incoming HL7 messages from external systems.

  ## Current State

  This modal is currently a placeholder. The listen functionality is accessible via
  the toolbar button (currently commented out in +page.svelte), but the modal UI
  doesn't yet provide configuration options or message viewing.

  ## Why is auto-start commented out?

  The commented code (lines 24-33) would automatically start the listen server when
  the modal opens. This was disabled because:
  1. We need to add UI controls for host/port configuration first
  2. Auto-starting could be confusing if the user just wants to view settings
  3. The listen server should be explicitly started by user action, not implicitly
     by opening a modal

  When the listen UI is implemented, it should include:
  - Host/port configuration (similar to message_send_modal.svelte)
  - Start/Stop buttons
  - List of received messages with read/unread status
  - Message viewing/copying capabilities
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { get, type Writable } from "svelte/store";
  import { startListening, stopListening } from "./listen";
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";

  let {
    show = $bindable(false),
    listening,
    listenedMessages: _listenedMessages,
  }: {
    show: boolean;
    listening: Writable<boolean>;
    listenedMessages: Writable<
      {
        message: string;
        unread: boolean;
      }[]
    >;
  } = $props();

  onMount(() => {
    /*
     * Auto-start code (DISABLED - see component documentation above)
     *
     * This code would automatically start listening when the modal opens, but is
     * commented out pending implementation of proper configuration UI.
     */
    /*if (!get(listening)) {
      startListening(null, 2575, listening)
        .then(() => {
          listening.set(true);
        })
        .catch((error) => {
          console.error("Error starting listening:", error);
          listening.set(false);
        });
    }*/

    /**
     * Cleanup on unmount
     *
     * When the modal is closed/unmounted, we stop the listen server to free the
     * network port. This prevents port conflicts if the modal is reopened or if
     * the application is restarted.
     *
     * We set listening to false first to update UI state, then call stopListening
     * to actually close the server socket on the Rust backend.
     */
    return () => {
      if (get(listening)) {
        listening.set(false);
      }
      stopListening(listening);
    };
  });

  const handleClose = () => {
    show = false;
  };
</script>

<Modal bind:show>
  <ModalHeader onclose={handleClose}>Listen</ModalHeader>
  <main>
    <form></form>
  </main>
</Modal>

<style>
  main {
    padding: 1rem;
  }
</style>
