<script lang="ts">
  import { message as tauriMessage } from "@tauri-apps/plugin-dialog";
  import {
    wizardSearchPatients,
    wizardApplyPatient,
    type WizardPatient,
  } from "../../backend/wizards/patient_wizard";
  import type { WizardDatabase } from "../../backend/wizards/wizard_database";
  import IconWizard from "$lib/icons/IconWizard.svelte";
  import type { Settings } from "../../settings";
  import DatabaseConnection from "$lib/forms/database_connection.svelte";
  import Modal from "$lib/components/modal.svelte";
  import ModalHeader from "$lib/components/modal_header.svelte";
  import ModalFooter from "$lib/components/modal_footer.svelte";
  import WizardLoading from "./wizard_loading.svelte";
  import WizardNoResults from "./wizard_no_results.svelte";
  import WizardResults from "./wizard_results.svelte";
  import WizardToggle from "./wizard_toggle.svelte";
  import WizardSearchButton from "./wizard_search_button.svelte";
  import WizardTooltip from "./wizard_tooltip.svelte";

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

  // Form is valid if database connection is valid AND at least one search field is filled
  const searchFormValid = $derived(
    dbFormValid &&
      (patientName.trim() !== "" ||
        patientId.trim() !== "" ||
        patientMrn.trim() !== ""),
  );

  const close = () => {
    show = false;
  };

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

  // Reset wizard state when modal opens (but keep database settings)
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
        <legend>Patient Search</legend>
        <!-- TODO: inform the user that at least one field is required, not all -->
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
</style>
