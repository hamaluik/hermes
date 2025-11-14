<script lang="ts">
  import { onMount } from "svelte";
  import { message as tauriMessage } from "@tauri-apps/plugin-dialog";
  import IconClose from "../icons/IconClose.svelte";
  import IconSearch from "$lib/icons/IconSearch.svelte";
  import {
    wizardSearchPatients,
    wizardApplyPatient,
    type WizardPatient,
  } from "../../backend/wizards/patient_wizard";
  import type { WizardDatabase } from "../../backend/wizards/wizard_database";
  import IconWizard from "$lib/icons/IconWizard.svelte";
  import type { Settings } from "../../settings";
  import DatabaseConnection from "$lib/forms/database_connection.svelte";

  let {
    onclose, // called when the user wants to close the wizard
    message, // the message as passed into the wizard
    onchange, // called when the wizard wants to change the message
    settings, // settings instance for persistent database connection
  }: {
    onclose?: () => void;
    message?: string;
    onchange?: (message: string) => void;
    settings: Settings;
  } = $props();

  let dialogElement: HTMLDialogElement | null = $state(null);

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
    if (dialogElement) {
      dialogElement.close();
    }
    onclose?.();
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

  onMount(() => {
    dialogElement?.showModal();
    dialogElement?.addEventListener("close", () => {
      close();
    });

    return () => {
      close();
    };
  });
</script>

<dialog class="modal" closedby="any" bind:this={dialogElement}>
  <header>
    <h1><IconWizard /> Patient Wizard</h1>
    <button class="close" onclick={close}>
      <IconClose />
    </button>
  </header>
  <main>
    <form onsubmit={handleSearch}>
      <DatabaseConnection {settings} bind:isValid={dbFormValid} />
      <fieldset>
        <legend>Patient Search</legend>
        <!-- TODO: inform the user that at least one field is required, not all -->
        <label for="patientName">Patient Name</label>
        <label for="patientId">Patient ID</label>
        <label for="patientMrn">Patient MRN</label>
        <div class="label-with-tooltip">
          <span>Override Segment</span>
          <span
            class="tooltip-icon"
            title="When enabled, this will completely overwrite the PID segment with the selected interface values"
            >ⓘ</span
          >
        </div>
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
        <label class="toggle-switch">
          <input
            type="checkbox"
            id="overrideSegment"
            bind:checked={overrideSegment}
          />
          <span class="toggle-slider"></span>
        </label>
        <button
          type="submit"
          class="search-button"
          disabled={isSearching || !searchFormValid}
          title={isSearching
            ? "Searching..."
            : !dbFormValid
              ? "Please configure database connection"
              : !searchFormValid
                ? "Please enter at least one search criteria"
                : "Search Patients"}
        >
          <IconSearch />
        </button>
      </fieldset>
    </form>
    {#if isSearching}
      <div class="loading">
        <div class="spinner"></div>
        <p>Searching for patients...</p>
      </div>
    {/if}
    {#if hasSearched && !isSearching}
      {#if patients.length > 0}
        <div class="results">
          <table>
            <thead>
              <tr>
                <th>Patient ID</th>
                <th>MRN</th>
                <th>Last Name</th>
                <th>First Name</th>
                <th>DOB</th>
                <th>Sex</th>
              </tr>
            </thead>
            <tbody>
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
            </tbody>
          </table>
        </div>
      {:else}
        <div class="no-results">
          <p>No patients found matching your criteria.</p>
        </div>
      {/if}
    {/if}
  </main>
  <footer>
    <button class="cancel" onclick={close}>Cancel</button>
    <button class="apply" onclick={handleApply} disabled={!selectedPatient}>
      Apply
    </button>
  </footer>
</dialog>

<style>
  .modal {
    display: none;
    &[open] {
      display: flex;
    }

    isolation: isolate;
    z-index: 2000;

    background: var(--col-overlay);
    border: 1px solid var(--col-highlightHigh);
    outline: none;
    color: var(--col-text);
    border-radius: 0.5em;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
    padding: 0;
    margin: 0;

    max-width: 90vw;
    max-height: 90vh;

    &::backdrop {
      background: rgba(0, 0, 0, 0.1);
      backdrop-filter: blur(5px);
    }

    position: fixed;
    inset: 0;
    margin: auto;

    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;

    header {
      width: 100%;
      height: 2.5em;
      border-radius: 8px 8px 0 0;
      display: flex;
      flex-direction: row;
      align-items: stretch;
      justify-content: space-between;
      margin: 0;
      padding: 0;
      background: none;

      h1 {
        font-size: medium;
        font-weight: 700;
        padding: 0.5em 1ch;

        display: inline-flex;
        flex-direction: row;
        align-items: center;
        gap: 1ch;
      }

      button.close {
        background: transparent;
        border: none;
        cursor: pointer;
        color: var(--col-text);
        padding: 0.25em 1ch;

        &:hover {
          color: var(--col-love);
        }
      }
    }

    main {
      padding: 1rem;
      overflow-y: auto;

      fieldset {
        display: grid;
        grid-template-columns: 1fr 1fr 1fr 1fr auto;
        grid-template-rows: auto auto;
        gap: 0.5rem 0.75rem;
        align-items: center;

        > .search-button {
          grid-row: 1 / 3;
          grid-column: 5;
          align-self: center;
        }
      }

      .search-button {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 3.5em;
        height: 3.5em;
        padding: 0;
        background: var(--col-iris);
        color: var(--col-base);
        border: 1px solid var(--col-highlightHigh);
        border-radius: 4px;
        cursor: pointer;
        font-size: 1em;

        &:hover:not(:disabled) {
          background: var(--col-love);
        }

        &:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        :global(svg) {
          width: 1.5em;
          height: 1.5em;
        }
      }

      .results {
        margin-top: 1rem;

        table {
          width: 100%;
          border-collapse: collapse;

          th,
          td {
            padding: 0.5em;
            text-align: left;
            border-bottom: 1px solid var(--col-highlightMed);
          }

          th {
            background: var(--col-surface);
            font-weight: 600;
          }

          tbody tr {
            cursor: pointer;
            transition: background-color 0.2s;

            &:hover {
              background: var(--col-love);
              color: var(--col-base);
            }

            &.selected {
              background: var(--col-gold);
              color: var(--col-base);
            }
          }
        }
      }

      .no-results {
        margin-top: 1rem;
        padding: 1rem;
        text-align: center;
        color: var(--col-subtle);
        background: var(--col-highlightLow);
        border-radius: 4px;
      }

      .loading {
        margin-top: 1rem;
        padding: 1.5rem;
        text-align: center;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;

        p {
          color: var(--col-text);
          font-size: 0.95em;
        }
      }

      .spinner {
        width: 2.5em;
        height: 2.5em;
        border: 3px solid var(--col-highlightMed);
        border-top: 3px solid var(--col-iris);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
      }

      .label-with-tooltip {
        display: flex;
        align-items: center;
        gap: 0.5ch;
      }

      .tooltip-icon {
        display: inline-block;
        color: var(--col-subtle);
        font-size: 0.9em;
        cursor: help;
        transition: color 0.2s ease-in-out;

        &:hover {
          color: var(--col-iris);
        }
      }

      .toggle-switch {
        position: relative;
        display: inline-block;
        width: 3em;
        height: 1.75em;

        input[type="checkbox"] {
          opacity: 0;
          width: 0;
          height: 0;

          &:checked + .toggle-slider {
            background-color: var(--col-iris);
          }

          &:checked + .toggle-slider::before {
            transform: translateX(1.25em);
          }

          &:focus + .toggle-slider {
            box-shadow: 0 0 0 2px var(--col-iris);
          }
        }

        .toggle-slider {
          position: absolute;
          cursor: pointer;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: var(--col-highlightMed);
          transition: 0.2s;
          border-radius: 1.75em;
          border: 1px solid var(--col-highlightHigh);

          &::before {
            position: absolute;
            content: "";
            height: 1.25em;
            width: 1.25em;
            left: 0.25em;
            bottom: 0.125em;
            background-color: var(--col-base);
            transition: 0.2s;
            border-radius: 50%;
          }

          &:hover {
            background-color: var(--col-highlightHigh);
          }
        }
      }
    }

    footer {
      width: 100%;
      border-radius: 0 0 8px 8px;
      display: flex;
      flex-direction: row;
      align-items: center;
      justify-content: flex-end;
      gap: 0.5rem;
      padding: 0.75rem 1rem;
      background: none;

      button {
        padding: 0.5em 1.5em;
        border-radius: 4px;
        cursor: pointer;
        font-size: 0.9em;
        border: 1px solid var(--col-highlightHigh);
        transition: all 0.2s;

        &.cancel {
          background: transparent;
          color: var(--col-text);

          &:hover {
            background: var(--col-highlightLow);
          }
        }

        &.apply {
          background: var(--col-iris);
          color: var(--col-base);

          &:hover:not(:disabled) {
            background: var(--col-love);
          }

          &:disabled {
            opacity: 0.5;
            cursor: not-allowed;
          }
        }
      }
    }
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>
