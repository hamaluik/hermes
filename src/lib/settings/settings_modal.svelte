<!--
  Settings Modal

  Modal dialog for configuring application preferences. Organized into sections:

  ## General Section
  - Theme: 3-way toggle for Light/Auto/Dark mode with live preview
  - Auto-Save: Automatically save files after changes (also accessible via File menu)
  - Tabs Follow Cursor: Auto-switch segment tabs when cursor moves in raw editor

  ## Extensions Section
  - Manage third-party extensions (add, enable/disable, remove)
  - View extension runtime status
  - Reload extensions to apply configuration changes

  ## Save/Cancel Workflow

  This modal uses a "staging" pattern for general settings changes:
  1. Settings are read from the Settings object into local state variables
  2. User modifies the local state via form controls
  3. On Save: Local state is written back to the Settings object
  4. On Cancel/Close: Local state is discarded, Settings object unchanged

  This pattern provides two benefits:
  - Users can experiment with settings and cancel if they don't like the result
  - Settings don't change mid-interaction, avoiding confusion (e.g., tabs suddenly
    jumping around while the settings modal is still open)

  The Settings object itself handles persistence to Tauri's store, so we don't need
  explicit save-to-disk calls here - just updating the Settings properties is sufficient.

  **Note:** Extension settings persist immediately when modified (no Save button needed)
  because extension configuration changes require a "Reload Extensions" action to take
  effect, providing a natural commit boundary.

  ## Theme Live Preview

  Unlike other settings, the theme change is applied immediately when the user toggles
  it, providing a live preview of each theme option. This is handled specially:

  1. On modal open: Store the original theme in `originalTheme`
  2. An $effect watches `themeSetting` and applies it to `document.documentElement.dataset.theme`
  3. On Save: The already-applied theme is persisted to settings
  4. On Cancel: The theme is reverted to `originalTheme`

  This approach lets users see the actual theme before committing, which is important
  for a visual preference where the impact isn't obvious from just reading the option name.

  ## Auto-Save Menu Sync

  When Auto-Save is changed via this modal and saved, the Settings setter triggers
  the onAutoSaveChanged callback, which updates the File menu's checkable Auto-Save
  item. This ensures the menu always reflects the current setting state regardless
  of where it was changed (menu or modal).

  ## Layout Structure

  The modal content is divided into sections with:
  - Section headers (`<h3>`) with bottom borders for visual separation
  - Scrollable content area (`overflow-y: auto`) to accommodate growing extension lists
  - Consistent spacing (`gap: 2rem`) between sections
-->
<script lang="ts">
  import type { Settings } from "../../settings";
  import IconSave from "$lib/icons/IconSave.svelte";
  import ToggleSwitch from "$lib/forms/toggle_switch.svelte";
  import ThemeToggle from "./theme_toggle.svelte";
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";
  import ModalFooter from "$lib/components/modal_footer.svelte";
  import ExtensionsSettings from "./extensions_settings.svelte";

  let {
    settings,
    show = $bindable(false),
  }: {
    settings: Settings;
    show: boolean;
  } = $props();

  // Local staging state for settings changes
  let tabsFollowCursor: boolean = $state(settings.tabsFollowCursor);
  let autoSaveEnabled: boolean = $state(settings.autoSaveEnabled);
  let themeSetting: "light" | "dark" | "auto" = $state(settings.themeSetting);

  // Store original theme when modal opens, for reverting on cancel
  let originalTheme: "light" | "dark" | "auto" = $state(settings.themeSetting);

  // Apply theme preview immediately when toggle changes
  $effect(() => {
    document.documentElement.dataset.theme = themeSetting;
  });

  // Reset staging state when modal opens
  $effect(() => {
    if (show) {
      tabsFollowCursor = settings.tabsFollowCursor;
      autoSaveEnabled = settings.autoSaveEnabled;
      themeSetting = settings.themeSetting;
      originalTheme = settings.themeSetting;
    }
  });

  const saveSettings = () => {
    settings.tabsFollowCursor = tabsFollowCursor;
    settings.autoSaveEnabled = autoSaveEnabled;
    settings.themeSetting = themeSetting;
  };

  const handleSave = () => {
    saveSettings();
    show = false;
  };

  const handleClose = () => {
    // Revert theme preview to original
    document.documentElement.dataset.theme = originalTheme;
    show = false;
  };
</script>

<Modal bind:show width="min(40rem, 90vw)" height="min(36rem, 85vh)">
  <ModalHeader onclose={handleClose}>Settings</ModalHeader>
  <main>
    <section class="general-settings">
      <h3>General</h3>
      <form method="dialog">
        <label for="themeSetting">Theme</label>
        <ThemeToggle id="themeSetting" bind:value={themeSetting} />
        <label for="autoSaveEnabled">Auto-Save</label>
        <ToggleSwitch id="autoSaveEnabled" bind:checked={autoSaveEnabled} />
        <label for="tabsFollowCursor">Tabs Follow Cursor</label>
        <ToggleSwitch id="tabsFollowCursor" bind:checked={tabsFollowCursor} />
      </form>
    </section>

    <section class="extensions-section">
      <ExtensionsSettings {settings} />
    </section>
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
    overflow-y: auto;
    gap: 2rem;

    section {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;

      h3 {
        margin: 0;
        font-size: 1.2rem;
        color: var(--col-text);
        padding-bottom: 0.5rem;
        border-bottom: 1px solid var(--col-highlightMed);
      }
    }

    .general-settings form {
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

  :global(html[data-theme="dark"]) button.save {
    color: var(--col-text);
  }

  @media (prefers-color-scheme: dark) {
    :global(html[data-theme="auto"]) button.save {
      color: var(--col-text);
    }
  }
</style>
