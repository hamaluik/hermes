<!--
  Patient Wizard Component

  Allows users to quickly populate PID (Patient Identification) segment data from the database patients.

  Purpose:
  The PID segment contains patient demographic information (name, ID, MRN, DOB, gender, etc.).
  Instead of manually typing these values, this wizard:
  1. Queries the the database database for patients matching search criteria
  2. Lets users select from matching patients
  3. Populates PID fields with the selected patient's information

  Search Flexibility:
  Users can search by any combination of:
  - Patient Name (last name partial match)
  - Patient ID (exact match)
  - Medical Record Number (MRN, exact match)

  At least one field must be filled for the search to be valid. This prevents
  accidental "search all patients" queries that could return thousands of results.

  Form Validation:
  The searchFormValid derived value ensures:
  1. Database connection is properly configured
  2. At least one search field has a non-empty value

  Flow:
  1. User opens wizard from toolbar
  2. User fills in database connection (persisted from settings)
  3. User enters search criteria (name, ID, or MRN)
  4. User clicks "Search Patients"
  5. Results display in table
  6. User selects a patient row
  7. User clicks "Apply" to update PID segment
  8. Message is updated via onchange callback
-->
<script lang="ts">
  import { message as tauriMessage } from "@tauri-apps/plugin-dialog";
  import {
    wizardSearchPatients,
    wizardApplyPatient,
    type WizardPatient,
  } from "./patient_wizard";
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
  let patientName: string = $state("");
  let patientId: string = $state("");
  let patientMrn: string = $state("");
  let overrideSegment: boolean = $state(true);

  let patients: WizardPatient[] = $state([]);
  let selectedPatient: WizardPatient | null = $state(null);
  let isSearching: boolean = $state(false);
  let hasSearched: boolean = $state(false);

  /**
   * Search form validation
   *
   * Ensures users provide enough information for a meaningful search:
   * 1. Database connection must be valid (host, port, credentials)
   * 2. At least ONE search criterion must be provided (name, ID, or MRN)
   *
   * This prevents "search all" queries that would return too many results
   * and helps users narrow down to the specific patient they need.
   */
  const searchFormValid = $derived(
    dbFormValid &&
      (patientName.trim() !== "" ||
        patientId.trim() !== "" ||
        patientMrn.trim() !== ""),
  );

  const close = () => {
    show = false;
  };

  /**
   * Searches for patients in the the database database
   *
   * Passes all search criteria to backend, which builds a SQL query with
   * appropriate wildcards and filters. Empty strings are ignored by the backend.
   *
   * Search behaviour:
   * - Name: Partial match on last name (e.g., "Doe" finds "Doe", "Doerr", etc.)
   * - ID: Exact match on patient ID
   * - MRN: Exact match on medical record number
   *
   * Clears any previous selection to avoid applying stale patient data.
   */
  const handleSearch = async (e: Event) => {
    e.preventDefault();
    isSearching = true;
    hasSearched = true;
    selectedPatient = null;

    try {
      const db: WizardDatabase = {
        host: settings.wizardDbHost,
        port: settings.wizardDbPort,
        database: settings.wizardDbDatabase,
        user: settings.wizardDbUser,
        password: settings.wizardDbPassword,
      };
      patients = await wizardSearchPatients(
        db,
        patientName.trim() || "",
        patientId.trim() || "",
        patientMrn.trim() || "",
      );
    } catch (error) {
      console.error("Error searching patients:", error);
      patients = [];
    } finally {
      isSearching = false;
    }
  };

  const selectPatient = (patient: WizardPatient) => {
    selectedPatient = patient;
  };

  /**
   * Applies the selected patient to the message
   *
   * Calls backend to update PID segment with patient demographic information.
   * Override mode determines whether to merge with or replace existing PID data.
   * On success, notifies parent via onchange callback and closes the wizard.
   * On error, shows user-friendly error dialog.
   */
  const handleApply = async () => {
    if (!selectedPatient || !message) return;

    try {
      const updatedMessage = await wizardApplyPatient(
        message,
        selectedPatient,
        overrideSegment,
      );
      onchange?.(updatedMessage);
      close();
    } catch (error) {
      console.error("Error applying patient:", error);
      await tauriMessage("Failed to apply patient.\\n\\n" + error, {
        title: "Error",
        kind: "error",
      });
    }
  };

  /**
   * Reset wizard state when modal opens
   *
   * Ensures clean slate for each wizard session. Database connection settings
   * are preserved (managed in Settings), but search fields and results are cleared.
   *
   * Unlike header wizard, this doesn't auto-populate from the message because
   * patient search requires explicit user input (there's no single "current patient"
   * to extract from the message).
   */
  $effect(() => {
    if (show) {
      // Reset search results and selection
      patients = [];
      selectedPatient = null;
      isSearching = false;
      hasSearched = false;

      // Reset search fields
      patientName = "";
      patientId = "";
      patientMrn = "";

      // Reset to default values
      overrideSegment = true;
    }
  });
</script>

<Modal bind:show maxWidth="90vw" maxHeight="90vh">
  <ModalHeader onclose={close}>
    <IconWizard /> Patient Wizard
  </ModalHeader>
  <main>
    <form onsubmit={handleSearch}>
      <DatabaseConnection {settings} bind:isValid={dbFormValid} />
      <fieldset>
        <legend>Patient Search <span class="field-hint">(at least one field required)</span></legend>
        <label for="patientName">Patient Name</label>
        <label for="patientId">Patient ID</label>
        <label for="patientMrn">Patient MRN</label>
        <input
          type="text"
          id="patientName"
          name="patientName"
          bind:value={patientName}
          maxlength={81}
          placeholder="Doe"
        />
        <input
          type="text"
          id="patientId"
          name="patientId"
          bind:value={patientId}
          maxlength={10}
          placeholder="123456"
        />
        <input
          type="text"
          id="patientMrn"
          name="patientMrn"
          bind:value={patientMrn}
          maxlength={20}
          placeholder="MRN00123"
        />
      </fieldset>
      <div class="search-action">
        <WizardSearchButton
          disabled={isSearching || !searchFormValid}
          title={isSearching
            ? "Searching..."
            : !dbFormValid
              ? "Please configure database connection"
              : !searchFormValid
                ? "Please enter at least one search criteria"
                : "Search Patients"}
        />
      </div>
    </form>
    {#if isSearching}
      <WizardLoading message="Searching for patients..." />
    {/if}
    {#if hasSearched && !isSearching}
      {#if patients.length > 0}
        <WizardResults>
          {#snippet header()}
            <tr>
              <th>Patient ID</th>
              <th>MRN</th>
              <th>Last Name</th>
              <th>First Name</th>
              <th>DOB</th>
              <th>Sex</th>
            </tr>
          {/snippet}
          {#snippet body()}
            {#each patients as patient}
              <tr
                class:selected={selectedPatient === patient}
                onclick={() => selectPatient(patient)}
              >
                <td>{patient.id}</td>
                <td>{patient.mrn}</td>
                <td>{patient.lname}</td>
                <td>{patient.fname}</td>
                <td>{patient.dob || ""}</td>
                <td>{patient.gender || ""}</td>
              </tr>
            {/each}
          {/snippet}
        </WizardResults>
      {:else}
        <WizardNoResults message="No patients found matching your criteria." />
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
      <button class="apply" onclick={handleApply} disabled={!selectedPatient}>
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
        grid-template-columns: 1fr 1fr 1fr;
        grid-template-rows: auto auto;
        gap: 0.5rem 0.75rem;
        align-items: center;

        > label {
          font-size: 0.9em;
          font-weight: 500;
          color: var(--col-text);
          grid-row: 1;
        }

        > input {
          grid-row: 2;
        }
      }

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

  .field-hint {
    font-size: 0.85em;
    font-weight: normal;
    color: var(--col-subtle);
    font-style: italic;
  }
</style>
