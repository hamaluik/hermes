/**
 * Settings Modal
 *
 * Modal dialog for configuring application preferences. Currently provides UI for
 * a single setting (Tabs Follow Cursor), but structured to easily accommodate
 * additional settings in the future.
 *
 * ## Save/Cancel Workflow
 *
 * This modal uses a "staging" pattern for settings changes:
 * 1. Settings are read from the Settings object into local state variables
 * 2. User modifies the local state via form controls
 * 3. On Save: Local state is written back to the Settings object
 * 4. On Cancel/Close: Local state is discarded, Settings object unchanged
 *
 * This pattern provides two benefits:
 * - Users can experiment with settings and cancel if they don't like the result
 * - Settings don't change mid-interaction, avoiding confusion (e.g., tabs suddenly
 *   jumping around while the settings modal is still open)
 *
 * The Settings object itself handles persistence to Tauri's store, so we don't need
 * explicit save-to-disk calls here - just updating the Settings properties is sufficient.
 */
<script lang="ts">
  import type { Settings } from "../settings";
  import IconSave from "./icons/IconSave.svelte";
  import ToggleSwitch from "./forms/toggle_switch.svelte";
  import Modal from "./components/modal.svelte";
  import ModalHeader from "./components/modal_header.svelte";
  import ModalFooter from "./components/modal_footer.svelte";

  let {
    settings,
    show = $bindable(false),
  }: {
    settings: Settings;
    show: boolean;
  } = $props();

  // Local staging state for settings changes
  let tabsFollowCursor: boolean = $state(settings.tabsFollowCursor);

  const saveSettings = () => {
    settings.tabsFollowCursor = tabsFollowCursor;
  };

  const handleSave = () => {
    saveSettings();
    show = false;
  };

  const handleClose = () => {
    show = false;
  };
</script>

<Modal bind:show>
  <ModalHeader onclose={handleClose}>Settings</ModalHeader>
  <main>
    <form method="dialog">
      <label for="tabsFollowCursor">Tabs Follow Cursor</label>
      <ToggleSwitch
        id="tabsFollowCursor"
        checked={tabsFollowCursor}
        onchange={() => {
          tabsFollowCursor = !tabsFollowCursor;
        }}
      />
    </form>
  </main>
  <ModalFooter>
    {#snippet right()}
      <button class="cancel" onclick={handleClose}>Cancel</button>
      <button class="save" onclick={handleSave}>
        <IconSave />
        <span>Save</span>
      </button>
    {/snippet}
  </ModalFooter>
</Modal>

<style>
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: stretch;
    padding: 0.5rem 2ch;
    color: var(--col-text);

    form {
      display: grid;
      grid-template-columns: 1fr auto;
      gap: 0.8lh 1ch;
      align-items: center;
    }
  }

  button.save {
    display: inline-flex;
    flex-direction: row;
    align-items: center;
    gap: 1ch;
    background: var(--col-pine);
    color: var(--col-base);

    &:hover {
      background: var(--col-gold);
      color: var(--col-base);
    }
  }

  @media (prefers-color-scheme: dark) {
    button.save {
      color: var(--col-text);
    }
  }
</style>
