<!--
  Visit Wizard Component

  Allows users to quickly populate PV1 (Patient Visit) segment data from the database visits.

  Purpose:
  The PV1 segment contains visit/encounter information (location, type, account number, dates).
  Instead of manually typing these values, this wizard:
  1. Extracts patient information from the current message's PID segment
  2. Queries the the database database for visits for that patient
  3. Lets users select from the patient's visits
  4. Populates PV1 fields with the selected visit's information

  Context-Aware Search:
  Unlike the patient wizard which requires explicit search input, the visit wizard
  automatically searches based on the patient in the current message. This provides
  a seamless workflow:
  1. User populates PID with Patient Wizard
  2. User opens Visit Wizard
  3. Visits for that patient automatically appear
  4. User selects appropriate visit

  Visit Type Mapping:
  The visitType helper function translates database codes to human-readable labels:
  - I → Inpatient
  - O → Outpatient
  - E → Emergency

  Flow:
  1. User opens wizard from toolbar
  2. Component automatically extracts patient info from message
  3. User clicks "Search Visits" (no additional criteria needed)
  4. Results display visits for the current patient
  5. User selects a visit row
  6. User clicks "Apply" to update PV1 segment
  7. Message is updated via onchange callback
-->
<script lang="ts">
  import { message as tauriMessage } from "@tauri-apps/plugin-dialog";
  import {
    wizardSearchVisits,
    wizardApplyVisit,
    type WizardVisit,
  } from "./visit_wizard";
  import type { WizardDatabase } from "../shared/wizard_database";
  import IconWizard from "$lib/icons/IconWizard.svelte";
  import type { Settings } from "../../../settings";
  import DatabaseConnection from "$lib/forms/database_connection.svelte";
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
  let overrideSegment: boolean = $state(true);

  let visits: WizardVisit[] = $state([]);
  let selectedVisit: WizardVisit | null = $state(null);
  let isSearching: boolean = $state(false);
  let hasSearched: boolean = $state(false);

  const searchFormValid = $derived(dbFormValid);

  const close = () => {
    show = false;
  };

  /**
   * Translates visit type codes to human-readable labels
   *
   * the database stores visit types as single-character codes in the database.
   * This function makes the results table more user-friendly by showing
   * descriptive labels instead of cryptic codes.
   */
  const visitType = (code: string) => {
    switch (code) {
      case "I":
        return "Inpatient";
      case "O":
        return "Outpatient";
      case "E":
        return "Emergency";
      default:
        return "Unknown";
    }
  };

  /**
   * Searches for visits in the the database database
   *
   * Context-aware search: Passes the current message to the backend, which
   * extracts patient information (from PID segment) and queries for that
   * patient's visits.
   *
   * This eliminates the need for manual search input - the wizard automatically
   * finds visits for the patient already in the message. If no valid patient
   * is found in the message, the backend will return an empty result set.
   */
  const handleSearch = async (e: Event) => {
    e.preventDefault();
    isSearching = true;
    hasSearched = true;
    selectedVisit = null;

    try {
      const db: WizardDatabase = {
        host: settings.wizardDbHost,
        port: settings.wizardDbPort,
        database: settings.wizardDbDatabase,
        user: settings.wizardDbUser,
        password: settings.wizardDbPassword,
      };
      visits = await wizardSearchVisits(db, message || "");
    } catch (error) {
      console.error("Error searching visits:", error);
      visits = [];
    } finally {
      isSearching = false;
    }
  };

  const selectVisit = (visit: WizardVisit) => {
    selectedVisit = visit;
  };

  /**
   * Applies the selected visit to the message
   *
   * Calls backend to update PV1 segment with visit/encounter information.
   * Unlike other wizards, this requires database access during apply (not just search)
   * because visit data may reference additional database tables.
   *
   * Override mode determines whether to merge with or replace existing PV1 data.
   * On success, notifies parent via onchange callback and closes the wizard.
   * On error, shows user-friendly error dialog.
   */
  const handleApply = async () => {
    if (!selectedVisit || !message) return;

    try {
      const db: WizardDatabase = {
        host: settings.wizardDbHost,
        port: settings.wizardDbPort,
        database: settings.wizardDbDatabase,
        user: settings.wizardDbUser,
        password: settings.wizardDbPassword,
      };
      const updatedMessage = await wizardApplyVisit(
        db,
        message,
        selectedVisit,
        overrideSegment,
      );
      onchange?.(updatedMessage);
      close();
    } catch (error) {
      console.error("Error applying visit:", error);
      await tauriMessage("Failed to apply visit.\\n\\n" + error, {
        title: "Error",
        kind: "error",
      });
    }
  };

  /**
   * Reset wizard state when modal opens
   *
   * Ensures clean slate for each wizard session. Database connection settings
   * are preserved (managed in Settings), but search results are cleared.
   *
   * Unlike other wizards, there are no search form fields to reset because
   * the visit search is entirely context-aware (derives search from current message).
   */
  $effect(() => {
    if (show) {
      // Reset search results and selection
      visits = [];
      selectedVisit = null;
      isSearching = false;
      hasSearched = false;

      // Reset to default values
      overrideSegment = true;
    }
  });
</script>

<Modal bind:show maxWidth="90vw" maxHeight="90vh">
  <ModalHeader onclose={close}>
    <IconWizard /> Visit Wizard
  </ModalHeader>
  <main>
    <form onsubmit={handleSearch}>
      <DatabaseConnection {settings} bind:isValid={dbFormValid} />
      <p>
        Note: This wizard currently searches for patient visits based on the
        patient in your message, if such a patient exists in the database.
      </p>
      <div class="search-action">
        <WizardSearchButton
          disabled={isSearching || !searchFormValid}
          title={isSearching
            ? "Searching..."
            : !dbFormValid
              ? "Please configure database connection"
              : !searchFormValid
                ? "Please enter at least one search criteria"
                : "Search Visits"}
        />
      </div>
    </form>
    {#if isSearching}
      <WizardLoading message="Searching for visits..." />
    {/if}
    {#if hasSearched && !isSearching}
      {#if visits.length > 0}
        <WizardResults>
          {#snippet header()}
            <tr>
              <th>#</th>
              <th>Location</th>
              <th>Type</th>
              <th>Account Number</th>
              <th>Admission Date</th>
              <th>Discharge Date</th>
            </tr>
          {/snippet}
          {#snippet body()}
            {#each visits as visit}
              <tr
                class:selected={selectedVisit === visit}
                onclick={() => selectVisit(visit)}
              >
                <td>{visit.seqno}</td>
                <td>{visit.location_id}</td>
                <td>{visitType(visit.patient_type_cd)}</td>
                <td>{visit.account_number}</td>
                <td>{visit.admission_date}</td>
                <td>{visit.discharge_date}</td>
              </tr>
            {/each}
          {/snippet}
        </WizardResults>
      {:else}
        <WizardNoResults message="No visits found for your patient." />
      {/if}
    {/if}
  </main>
  <ModalFooter>
    {#snippet left()}
      <div class="override-toggle">
        <WizardToggle id="overrideSegment" bind:checked={overrideSegment} />
        <label for="overrideSegment">Override Segment</label>
        <WizardTooltip
          title="When enabled, this will completely overwrite the PID segment with the selected interface values"
        />
      </div>
    {/snippet}
    {#snippet right()}
      <button class="apply" onclick={handleApply} disabled={!selectedVisit}>
        Apply
      </button>
    {/snippet}
  </ModalFooter>
</Modal>

<style>
  main {
    padding: 1rem;
    overflow-y: auto;

    .search-action {
      display: flex;
      justify-content: flex-end;
      margin-top: 1rem;
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
