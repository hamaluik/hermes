<!--
  Header Wizard Component

  Allows users to quickly populate MSH (Message Header) segment data from the database interfaces.

  Purpose:
  The MSH segment contains routing information (sending/receiving application and facility).
  Instead of manually typing these values, this wizard:
  1. Queries the the database database for configured interfaces
  2. Lets users select from available interfaces
  3. Populates MSH fields with the selected interface's routing information

  Wizard Pattern:
  All wizards (Header, Patient, Visit) follow a consistent UX pattern:
  1. Database connection form (persisted in settings)
  2. Search criteria form (specific to wizard type)
  3. Search button (triggers backend query)
  4. Results display (loading state → results table or no results message)
  5. Selection (click to select row)
  6. Apply button (updates message and closes modal)
  7. Override toggle (whether to merge or replace segment data)

  Auto-Population:
  When the wizard opens, it attempts to auto-populate message type and trigger event
  from the current message. This reduces user input when the message already has
  valid MSH data.

  Flow:
  1. User opens wizard from toolbar
  2. Component auto-detects message type (ADT/ORM) and trigger event from existing message
  3. User clicks "Get Interfaces" to query database
  4. Results display in table
  5. User selects an interface row
  6. User clicks "Apply" to update MSH segment
  7. Message is updated via onchange callback
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { message as tauriMessage } from "@tauri-apps/plugin-dialog";
  import {
    wizardQueryInterfaces,
    wizardApplyInterface,
    type WizardInterface,
  } from "./header_wizard";
  import type { WizardDatabase } from "../shared/wizard_database";
  import IconWizard from "$lib/icons/IconWizard.svelte";
  import type { Settings } from "../../../settings";
  import DatabaseConnection from "$lib/forms/database_connection.svelte";
  import { getMessageTriggerEvent, getMessageType } from "$lib/shared/data";
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";
  import ModalFooter from "$lib/components/modal_footer.svelte";
  import WizardLoading from "../shared/wizard_loading.svelte";
  import WizardNoResults from "../shared/wizard_no_results.svelte";
  import WizardResults from "../shared/wizard_results.svelte";
  import WizardToggle from "../shared/wizard_toggle.svelte";
  import WizardSearchButton from "../shared/wizard_search_button.svelte";
  import WizardTooltip from "../shared/wizard_tooltip.svelte";

  let {
    show = $bindable(false),
    message, // the message as passed into the wizard
    onchange, // called when the wizard wants to change the message
    settings, // settings instance for persistent database connection
  }: {
    show: boolean;
    message?: string;
    onchange?: (message: string) => void;
    settings: Settings;
  } = $props();

  let dbFormValid: boolean = $state(false);
  let messageType: "ADT" | "ORM" = $state("ADT");
  let triggerEvent: string = $state("A01");
  let overrideSegment: boolean = $state(true);

  let interfaces: WizardInterface[] = $state([]);
  let selectedInterface: WizardInterface | null = $state(null);
  let isSearching: boolean = $state(false);
  let hasSearched: boolean = $state(false);

  /**
   * Trigger event options dynamically filtered by message type
   *
   * Different message types support different trigger events:
   * - ADT (Admit/Discharge/Transfer): A01-A08 events for patient movements
   * - ORM (Order Message): O01 for orders
   *
   * This ensures users only see valid trigger event options for their selected
   * message type, preventing invalid message configurations.
   */
  const triggerEventOptions = $derived(
    messageType === "ADT"
      ? [
          { value: "A01", label: "A01 (Admit/visit notification)" },
          { value: "A02", label: "A02 (Transfer a patient)" },
          { value: "A03", label: "A03 (Discharge/end visit)" },
          { value: "A04", label: "A04 (Register a patient)" },
          { value: "A05", label: "A05 (Pre-admit a patient)" },
          {
            value: "A06",
            label: "A06 (Change an outpatient to an inpatient)",
          },
          {
            value: "A07",
            label: "A07 (Change an inpatient to an outpatient)",
          },
          { value: "A08", label: "A08 (Update patient information)" },
        ]
      : [{ value: "O01", label: "O01 (Order message)" }],
  );

  const close = () => {
    show = false;
  };

  /**
   * Searches for interfaces in the the database database
   *
   * Queries the database for interfaces matching the selected message type.
   * Uses database connection settings from persistent user settings.
   * Clears any previous selection to avoid applying stale interface data.
   */
  const handleSearch = async (e: Event) => {
    e.preventDefault();
    isSearching = true;
    hasSearched = true;
    selectedInterface = null;

    try {
      const db: WizardDatabase = {
        host: settings.wizardDbHost,
        port: settings.wizardDbPort,
        database: settings.wizardDbDatabase,
        user: settings.wizardDbUser,
        password: settings.wizardDbPassword,
      };
      interfaces = await wizardQueryInterfaces(db, messageType);
    } catch (error) {
      console.error("Error querying interfaces:", error);
      interfaces = [];
    } finally {
      isSearching = false;
    }
  };

  const selectInterface = (iface: WizardInterface) => {
    selectedInterface = iface;
  };

  /**
   * Applies the selected interface to the message
   *
   * Calls backend to update MSH segment with interface routing information.
   * Override mode determines whether to merge with or replace existing MSH data.
   * On success, notifies parent via onchange callback and closes the wizard.
   * On error, shows user-friendly error dialog.
   */
  const handleApply = async () => {
    if (!selectedInterface || !message) return;

    try {
      const updatedMessage = await wizardApplyInterface(
        message,
        selectedInterface,
        messageType,
        triggerEvent,
        overrideSegment,
      );
      onchange?.(updatedMessage);
      close();
    } catch (error) {
      console.error("Error applying interface:", error);
      await tauriMessage("Failed to apply interface.\n\n" + error, {
        title: "Error",
        kind: "error",
      });
    }
  };

  onMount(() => {
    // Auto-populate message type and trigger event from the current message
    if (message) {
      Promise.all([
        getMessageType(message).catch(() => null),
        getMessageTriggerEvent(message).catch(() => null),
      ]).then(([msgType, trigEvent]) => {
        // Auto-populate message type if it matches available options
        if (msgType === "ADT" || msgType === "ORM") {
          messageType = msgType;
        }

        // Auto-populate trigger event if it matches available options
        if (trigEvent) {
          const validEvent = triggerEventOptions.find(
            (opt) => opt.value === trigEvent,
          );
          if (validEvent) {
            triggerEvent = trigEvent;
          }
        }
      });
    }
  });

  /**
   * Reset wizard state when modal opens
   *
   * This effect fires when show becomes true, ensuring the wizard starts fresh
   * each time it opens. Database connection settings are preserved (managed in Settings),
   * but search results and selections are cleared to avoid confusion.
   *
   * Auto-population: After resetting to defaults, attempts to populate message type
   * and trigger event from the current message. This provides smart defaults while
   * still allowing users to override if needed.
   */
  $effect(() => {
    if (show) {
      // Reset search results and selection
      interfaces = [];
      selectedInterface = null;
      isSearching = false;
      hasSearched = false;

      // Reset to default values
      messageType = "ADT";
      triggerEvent = "A01";
      overrideSegment = true;

      // Re-populate from current message if available
      if (message) {
        Promise.all([
          getMessageType(message).catch(() => null),
          getMessageTriggerEvent(message).catch(() => null),
        ]).then(([msgType, trigEvent]) => {
          if (msgType === "ADT" || msgType === "ORM") {
            messageType = msgType;
          }
          if (trigEvent) {
            const validEvent = triggerEventOptions.find(
              (opt) => opt.value === trigEvent,
            );
            if (validEvent) {
              triggerEvent = trigEvent;
            }
          }
        });
      }
    }
  });
</script>

<Modal bind:show maxWidth="90vw" maxHeight="90vh">
  <ModalHeader onclose={close}>
    <IconWizard /> Header Wizard
  </ModalHeader>
  <main>
    <form onsubmit={handleSearch}>
      <DatabaseConnection {settings} bind:isValid={dbFormValid} />
      <fieldset>
        <legend>Message Options</legend>
        <label for="messageType">Message Type</label>
        <label for="triggerEvent">Trigger Event</label>
        <select
          id="messageType"
          name="messageType"
          bind:value={messageType}
          required
        >
          <option value="ADT">ADT</option>
          <option value="ORM">ORM</option>
        </select>
        <select
          id="triggerEvent"
          name="triggerEvent"
          bind:value={triggerEvent}
          required
        >
          {#each triggerEventOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </fieldset>
      <div class="search-action">
        <WizardSearchButton
          disabled={isSearching || !dbFormValid}
          title={isSearching
            ? "Searching..."
            : !dbFormValid
              ? "Please fill out all required fields correctly"
              : "Get Interfaces"}
        />
      </div>
    </form>
    {#if isSearching}
      <WizardLoading message="Searching for interfaces..." />
    {/if}
    {#if hasSearched && !isSearching}
      {#if interfaces.length > 0}
        <WizardResults>
          {#snippet header()}
            <tr>
              <th>Interface</th>
              <th>Sending App</th>
              <th>Sending Facility</th>
              <th>Receiving App</th>
              <th>Receiving Facility</th>
            </tr>
          {/snippet}
          {#snippet body()}
            {#each interfaces as iface}
              <tr
                class:selected={selectedInterface === iface}
                onclick={() => selectInterface(iface)}
              >
                <td>{iface.name}</td>
                <td>{iface.sending_app}</td>
                <td>{iface.sending_fac}</td>
                <td>{iface.receiving_app}</td>
                <td>{iface.receiving_fac}</td>
              </tr>
            {/each}
          {/snippet}
        </WizardResults>
      {:else}
        <WizardNoResults message="No interfaces found matching your criteria." />
      {/if}
    {/if}
  </main>
  <ModalFooter>
    {#snippet left()}
      <div class="override-toggle">
        <WizardToggle id="overrideSegment" bind:checked={overrideSegment} />
        <label for="overrideSegment">Override Segment</label>
        <WizardTooltip
          title="When enabled, this will completely overwrite the MSH segment with the selected interface values"
        />
      </div>
    {/snippet}
    {#snippet right()}
      <button class="apply" onclick={handleApply} disabled={!selectedInterface}>
        Apply
      </button>
    {/snippet}
  </ModalFooter>
</Modal>

<style>
  main {
    padding: 1rem;
    overflow-y: auto;

      fieldset {
        display: grid;
        grid-template-columns: 1fr 1fr;
        grid-template-rows: auto auto;
        gap: 0.5rem 0.75rem;
        align-items: center;

        > label {
          font-size: 0.9em;
          font-weight: 500;
          color: var(--col-text);
          grid-row: 1;
        }

        > select {
          grid-row: 2;
        }
      }

      .search-action {
        display: flex;
        justify-content: flex-end;
        margin-top: 1rem;
      }

      select {
        width: 100%;
        background: var(--col-surface);
        color: var(--col-text);
        border: 1px solid var(--col-highlightMed);
        border-radius: 4px;
        padding: 0.5em 0.75em;
        font-size: 1em;
        font-family: inherit;
        transition:
          border-color 0.2s ease-in-out,
          background-color 0.2s ease-in-out;
        cursor: pointer;
        appearance: none;
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23908caa' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
        background-repeat: no-repeat;
        background-position: right 0.75em center;
        background-size: 12px 12px;
        padding-right: 2.5em;

        &:hover {
          border-color: var(--col-highlightHigh);
          background-color: var(--col-surface);
          background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23e0def4' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
          background-repeat: no-repeat;
          background-position: right 0.75em center;
          background-size: 12px 12px;
        }

        &:focus {
          outline: none;
          border-color: var(--col-iris);
          background: var(--col-overlay);
          background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23c4a7e7' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
          background-repeat: no-repeat;
          background-position: right 0.75em center;
          background-size: 12px 12px;
        }
      }

  }

  .override-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;

    label {
      font-size: 0.9em;
      font-weight: 500;
      color: var(--col-text);
      cursor: pointer;
    }
  }
</style>
