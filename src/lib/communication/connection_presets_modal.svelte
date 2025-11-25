<!--
  Connection Presets Modal

  Manages saved host/port combinations for quick switching between environments.
  Allows adding, editing, and deleting presets.
-->
<script lang="ts">
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";
  import ModalFooter from "$lib/components/modal_footer.svelte";
  import Button from "$lib/components/button.svelte";
  import IconAdd from "$lib/icons/IconAdd.svelte";
  import IconTrash from "$lib/icons/IconTrash.svelte";
  import type { ConnectionPreset } from "./connection_preset";
  import { createPreset } from "./connection_preset";

  let {
    show = $bindable(false),
    presets,
    onSave,
  }: {
    show: boolean;
    presets: ConnectionPreset[];
    onSave: (presets: ConnectionPreset[]) => void;
  } = $props();

  // Local copy for editing
  let localPresets: ConnectionPreset[] = $state([]);

  // Form state for adding/editing
  let editingId: string | null = $state(null);
  let formName: string = $state("");
  let formHostname: string = $state("");
  let formPort: number = $state(2575);

  // Sync local copy when modal opens
  $effect(() => {
    if (show) {
      localPresets = presets.map((p) => ({ ...p }));
      resetForm();
    }
  });

  function resetForm() {
    editingId = null;
    formName = "";
    formHostname = "";
    formPort = 2575;
  }

  function startEdit(preset: ConnectionPreset) {
    editingId = preset.id;
    formName = preset.name;
    formHostname = preset.hostname;
    formPort = preset.port;
  }

  const hostPattern = /^[a-zA-Z0-9]([a-zA-Z0-9\-\.:]*[a-zA-Z0-9])?$/;
  const isFormValid = $derived(
    formName.trim().length > 0 &&
      formHostname.length >= 1 &&
      formHostname.length <= 255 &&
      hostPattern.test(formHostname) &&
      formPort >= 1 &&
      formPort <= 65535,
  );

  function savePreset() {
    if (!isFormValid) return;

    if (editingId) {
      // update existing
      localPresets = localPresets.map((p) =>
        p.id === editingId
          ? { ...p, name: formName.trim(), hostname: formHostname, port: formPort }
          : p,
      );
    } else {
      // add new
      const newPreset = createPreset(formName.trim(), formHostname, formPort);
      localPresets = [...localPresets, newPreset];
    }
    resetForm();
  }

  function deletePreset(id: string) {
    localPresets = localPresets.filter((p) => p.id !== id);
    if (editingId === id) {
      resetForm();
    }
  }

  function handleSave() {
    onSave(localPresets);
    show = false;
  }

  function handleClose() {
    show = false;
  }
</script>

<Modal bind:show maxWidth="700px">
  <ModalHeader onclose={handleClose}>Connection Presets</ModalHeader>

  <div class="content">
    <div class="left-panel">
      <div class="panel-header">Saved Presets</div>
      <div class="preset-list">
        {#if localPresets.length === 0}
          <div class="empty-state">No presets saved yet</div>
        {:else}
          {#each localPresets as preset (preset.id)}
            <div class="preset-item" class:editing={editingId === preset.id}>
              <button class="preset-info" onclick={() => startEdit(preset)}>
                <span class="preset-name">{preset.name}</span>
                <span class="preset-address">{preset.hostname}:{preset.port}</span>
              </button>
              <Button
                variant="danger"
                iconOnly
                onclick={() => deletePreset(preset.id)}
                title="Delete preset"
              >
                <IconTrash />
              </Button>
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <div class="divider"></div>

    <div class="right-panel">
      <div class="panel-header">
        {editingId ? "Edit Preset" : "Add Preset"}
      </div>

      <div class="form-row">
        <label for="preset-name">Name</label>
        <input
          type="text"
          id="preset-name"
          bind:value={formName}
          placeholder="e.g. Production"
          autocomplete="off"
        />
      </div>

      <div class="form-row">
        <label for="preset-hostname">Host</label>
        <input
          type="text"
          id="preset-hostname"
          bind:value={formHostname}
          placeholder="127.0.0.1"
          autocomplete="off"
        />
      </div>

      <div class="form-row">
        <label for="preset-port">Port</label>
        <input
          type="number"
          id="preset-port"
          bind:value={formPort}
          min="1"
          max="65535"
          placeholder="2575"
        />
      </div>

      <div class="form-actions">
        <Button variant="primary" onclick={savePreset} disabled={!isFormValid}>
          <IconAdd />
          {editingId ? "Update" : "Add"}
        </Button>
      </div>
    </div>
  </div>

  <ModalFooter>
    {#snippet right()}
      <Button variant="ghost" onclick={handleClose}>Cancel</Button>
      <Button variant="primary" onclick={handleSave}>Save</Button>
    {/snippet}
  </ModalFooter>
</Modal>

<style>
  .content {
    padding: 1rem;
    display: flex;
    flex-direction: row;
    gap: 1rem;
  }

  .left-panel {
    flex: 0 0 45%;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }

  .divider {
    width: 1px;
    background: var(--col-highlightMed);
    flex-shrink: 0;
  }

  .panel-header {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--col-subtle);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.5rem;
    flex-shrink: 0;
  }

  .preset-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    overflow-y: auto;
    max-height: 200px;
    background: var(--col-surface);
    border-radius: 4px;
    padding: 0.5rem;
  }

  .empty-state {
    padding: 1rem;
    text-align: center;
    color: var(--col-muted);
    font-size: 0.875rem;
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .preset-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem;
    border-radius: 4px;
    background: var(--col-highlightLow);
    flex-shrink: 0;

    &.editing {
      background: var(--col-highlightMed);
    }
  }

  .preset-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.125rem;
    padding: 0.375rem 0.5rem;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--col-text);
    text-align: left;
    min-width: 0;

    &:hover {
      background: var(--col-highlightLow);
      border-radius: 4px;
    }
  }

  .preset-name {
    font-weight: 600;
    font-size: 0.875rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }

  .preset-address {
    font-size: 0.75rem;
    color: var(--col-subtle);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
  }


  .form-row {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;

    label {
      font-size: 0.75rem;
      color: var(--col-subtle);
    }

    input {
      width: 100%;
      padding: 0.375rem 0.5rem;
      font-size: 0.875rem;
      background: var(--col-surface);
      border: 1px solid var(--col-highlightMed);
      border-radius: 4px;
      color: var(--col-text);

      &:focus {
        outline: none;
        border-color: var(--col-iris);
      }
    }

    input[type="number"] {
      appearance: textfield;
      -moz-appearance: textfield;

      &::-webkit-inner-spin-button,
      &::-webkit-outer-spin-button {
        appearance: none;
        -webkit-appearance: none;
        margin: 0;
      }
    }
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
</style>
