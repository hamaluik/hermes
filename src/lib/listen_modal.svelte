<script lang="ts">
  import { onMount } from "svelte";
  import { get, type Writable } from "svelte/store";
  import { startListening, stopListening } from "../backend/listen";
  import Modal from "./components/modal.svelte";
  import ModalHeader from "./components/modal_header.svelte";

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
    if (!get(listening)) {
      startListening(null, 2575, listening)
        .then(() => {
          listening.set(true);
        })
        .catch((error) => {
          console.error("Error starting listening:", error);
          listening.set(false);
        });
    }

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
