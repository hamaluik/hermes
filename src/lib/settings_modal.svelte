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
