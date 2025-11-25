<!--
  Keyboard Shortcuts Modal

  Displays all available keyboard shortcuts organised by category.
  Accessible via View menu or Cmd+/.
-->
<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";
  import ModalFooter from "$lib/components/modal_footer.svelte";
  import Button from "$lib/components/button.svelte";

  let { show = $bindable(false) }: { show: boolean } = $props();

  const shortcuts = [
    {
      category: "File",
      items: [
        { keys: ["Cmd", "N"], action: "New" },
        { keys: ["Cmd", "O"], action: "Open" },
        { keys: ["Cmd", "S"], action: "Save" },
        { keys: ["Cmd", "Shift", "S"], action: "Save As" },
      ],
    },
    {
      category: "Edit",
      items: [
        { keys: ["Cmd", "Z"], action: "Undo" },
        { keys: ["Cmd", "Shift", "Z"], action: "Redo" },
        { keys: ["Cmd", "F"], action: "Find" },
        { keys: ["Cmd", "H"], action: "Find and Replace" },
        { keys: ["Cmd", "J"], action: "Jump to Field" },
        { keys: ["Cmd", "Shift", "K"], action: "Delete Segment" },
        { keys: ["Cmd", "Shift", "\u2191"], action: "Move Segment Up" },
        { keys: ["Cmd", "Shift", "\u2193"], action: "Move Segment Down" },
        { keys: ["Cmd", "Shift", "D"], action: "Duplicate Segment" },
      ],
    },
    {
      category: "View",
      items: [
        { keys: ["Cmd", "="], action: "Zoom In" },
        { keys: ["Cmd", "-"], action: "Zoom Out" },
        { keys: ["Cmd", "0"], action: "Reset Zoom" },
        { keys: ["Cmd", "/"], action: "Keyboard Shortcuts" },
      ],
    },
    {
      category: "Tools",
      items: [
        { keys: ["Cmd", "T"], action: "Send Message" },
        { keys: ["Cmd", "L"], action: "Listen for Messages" },
        { keys: ["Cmd", "Shift", "V"], action: "Validate Message" },
        { keys: ["Cmd", "D"], action: "Compare Messages" },
        { keys: ["Cmd", "G"], action: "Generate Control ID" },
        { keys: ["Cmd", "Shift", "T"], action: "Insert Timestamp" },
      ],
    },
    {
      category: "Editor",
      items: [
        { keys: ["Tab"], action: "Next Field" },
        { keys: ["Shift", "Tab"], action: "Previous Field" },
        { keys: ["Cmd", "Enter"], action: "Open Send Drawer" },
      ],
    },
  ];

  const handleClose = () => {
    show = false;
  };
</script>

<Modal bind:show width="90vw" height="85vh">
  <ModalHeader onclose={handleClose}>Keyboard Shortcuts</ModalHeader>
  <main>
    {#each shortcuts as section}
      <section>
        <h3>{section.category}</h3>
        <dl>
          {#each section.items as shortcut}
            <div class="shortcut-row">
              <dt>
                {#each shortcut.keys as key, i}
                  <kbd>{key}</kbd>{#if i < shortcut.keys.length - 1}<span class="plus">+</span>{/if}
                {/each}
              </dt>
              <dd>{shortcut.action}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/each}
  </main>
  <ModalFooter>
    {#snippet right()}
      <Button variant="secondary" onclick={handleClose}>Close</Button>
    {/snippet}
  </ModalFooter>
</Modal>

<style>
  main {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem 3rem;
    padding: 1rem 2ch;
    color: var(--col-text);
    overflow-y: auto;
    min-height: 0;
  }

  section {
    h3 {
      font-size: 0.85em;
      font-weight: 600;
      color: var(--col-subtle);
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin: 0 0 0.5rem 0;
      padding-bottom: 0.25rem;
      border-bottom: 1px solid var(--col-highlightMed);
    }
  }

  dl {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .shortcut-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.25rem 0;
  }

  dt {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  dd {
    margin: 0;
    color: var(--col-text);
  }

  kbd {
    display: inline-block;
    padding: 0.2em 0.5em;
    font-family: ui-monospace, Menlo, Monaco, monospace;
    font-size: 0.8em;
    background-color: var(--col-overlay);
    border: 1px solid var(--col-highlightHigh);
    border-radius: 4px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
    min-width: 1.5em;
    text-align: center;
  }

  .plus {
    color: var(--col-muted);
    font-size: 0.8em;
  }
</style>
