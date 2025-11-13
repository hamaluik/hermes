<script lang="ts">
  import { onMount } from "svelte";
  import IconClose from "../icons/IconClose.svelte";
  import IconSearch from "$lib/icons/IconSearch.svelte";
  import {
    wizardQueryInterfaces,
    wizardApplyInterface,
    type WizardInterface,
  } from "../../backend/wizards/header_wizard";
  import type { WizardDatabase } from "../../backend/wizards/wizard_database";
  import IconWizard from "$lib/icons/IconWizard.svelte";

  let {
    onclose, // called when the user wants to close the wizard
    message, // the message as passed into the wizard
    onchange, // called when the wizard wants to change the message
  }: {
    onclose?: () => void;
    message?: string;
    onchange?: (message: string) => void;
  } = $props();

  let dialogElement: HTMLDialogElement | null = $state(null);

  let dbHost: string = $state("");
  let dbPort: number = $state(1433);
  let dbDatabase: string = $state("");
  let dbUser: string = $state("");
  let dbPass: string = $state("");
  let messageType: "ADT" | "ORM" = $state("ADT");
  let triggerEvent: string = $state("A01");

  let interfaces: WizardInterface[] = $state([]);
  let selectedInterface: WizardInterface | null = $state(null);
  let isSearching: boolean = $state(false);
  let hasSearched: boolean = $state(false);

  // Trigger event options based on message type
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

  // Form validation
  const isFormValid = $derived.by(() => {
    const hostPattern = /^[a-zA-Z0-9]([a-zA-Z0-9\-\.:]*[a-zA-Z0-9])?$/;
    const dbNamePattern = /^[a-zA-Z_@#][a-zA-Z0-9_@#$]{0,127}$/;
    const userPattern = /^[a-zA-Z_@#\\][a-zA-Z0-9_@#$\\]{0,127}$|^[a-zA-Z0-9_\\]+\\[a-zA-Z0-9_@#$]+$/;

    return (
      dbHost.length >= 3 &&
      dbHost.length <= 255 &&
      hostPattern.test(dbHost) &&
      dbPort >= 0 &&
      dbPort <= 65535 &&
      dbDatabase.length >= 1 &&
      dbDatabase.length <= 128 &&
      dbNamePattern.test(dbDatabase) &&
      dbUser.length >= 1 &&
      dbUser.length <= 128 &&
      userPattern.test(dbUser) &&
      dbPass.length >= 1 &&
      dbPass.length <= 128
    );
  });

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
    selectedInterface = null;

    try {
      const db: WizardDatabase = {
        host: dbHost,
        port: dbPort,
        database: dbDatabase,
        user: dbUser,
        password: dbPass,
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

  const handleApply = async () => {
    if (!selectedInterface || !message) return;

    try {
      const updatedMessage = await wizardApplyInterface(
        message,
        selectedInterface,
      );
      onchange?.(updatedMessage);
      close();
    } catch (error) {
      console.error("Error applying interface:", error);
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
    <h1><IconWizard /> Header Wizard</h1>
    <button class="close" onclick={close}>
      <IconClose />
    </button>
  </header>
  <main>
    <form onsubmit={handleSearch}>
      <fieldset>
        <legend>Database Connection</legend>
        <div class="form-group">
          <label for="dbHost">Host</label>
          <input
            type="text"
            id="dbHost"
            name="dbHost"
            bind:value={dbHost}
            minlength={3}
            maxlength={255}
            placeholder="localhost"
            required={true}
            pattern="^[a-zA-Z0-9]([a-zA-Z0-9\-\.:]*[a-zA-Z0-9])?$"
          />
        </div>
        <div class="form-group">
          <label for="dbPort">Port</label>
          <input
            type="number"
            id="dbPort"
            name="dbPort"
            bind:value={dbPort}
            min={0}
            max={65535}
            step={1}
            placeholder="1433"
            required={true}
          />
        </div>
        <div class="form-group">
          <label for="dbDatabase">Database</label>
          <input
            type="text"
            id="dbDatabase"
            name="dbDatabase"
            bind:value={dbDatabase}
            minlength={1}
            maxlength={128}
            placeholder="LAB"
            required={true}
            pattern={"^[a-zA-Z_@#][a-zA-Z0-9_@#$]{0,127}$"}
          />
        </div>
        <div class="form-group">
          <label for="dbUser">User</label>
          <input
            type="text"
            id="dbUser"
            name="dbUser"
            bind:value={dbUser}
            minlength={1}
            maxlength={128}
            placeholder="sa"
            required={true}
            pattern={"^[a-zA-Z_@#\\\\][a-zA-Z0-9_@#$\\\\]{0,127}$|^[a-zA-Z0-9_\\\\]+\\\\[a-zA-Z0-9_@#$]+$"}
          />
        </div>
        <div class="form-group">
          <label for="dbPass">Password</label>
          <input
            type="password"
            id="dbPass"
            name="dbPass"
            bind:value={dbPass}
            minlength={1}
            maxlength={128}
            placeholder="Password123"
            required={true}
          />
        </div>
      </fieldset>
      <fieldset>
        <legend>Message Options</legend>
        <div class="form-group">
          <label for="messageType">Message Type</label>
          <select
            id="messageType"
            name="messageType"
            bind:value={messageType}
            required
          >
            <option value="ADT">ADT</option>
            <option value="ORM">ORM</option>
          </select>
        </div>
        <div class="form-group">
          <label for="triggerEvent">Trigger Event</label>
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
        </div>
        <div class="form-group search-group">
          <button
            type="submit"
            class="search-button"
            disabled={isSearching || !isFormValid}
            title={isSearching ? "Searching..." : !isFormValid ? "Please fill out all required fields correctly" : "Get Interfaces"}
          >
            <IconSearch />
          </button>
        </div>
      </fieldset>
    </form>
    {#if hasSearched}
      {#if interfaces.length > 0}
        <div class="results">
          <table>
            <thead>
              <tr>
                <th>Interface</th>
                <th>Sending App</th>
                <th>Sending Facility</th>
                <th>Receiving App</th>
                <th>Receiving Facility</th>
              </tr>
            </thead>
            <tbody>
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
            </tbody>
          </table>
        </div>
      {:else}
        <div class="no-results">
          <p>No interfaces found matching your criteria.</p>
        </div>
      {/if}
    {/if}
  </main>
  <footer>
    <button class="cancel" onclick={close}>Cancel</button>
    <button class="apply" onclick={handleApply} disabled={!selectedInterface}>
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
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);

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
        flex-wrap: wrap;
      }

      .form-group {
        display: flex;
        flex-direction: column;
        flex: 1 1 0;
        min-width: 8ch;
      }

      .search-group {
        flex: 0 0 auto;
        min-width: auto;
      }

      input,
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

        &:hover {
          border-color: var(--col-highlightHigh);
        }

        &:focus {
          outline: none;
          border-color: var(--col-iris);
          background: var(--col-overlay);
        }

        &::placeholder {
          color: var(--col-subtle);
        }
      }

      select {
        cursor: pointer;
        appearance: none;
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23908caa' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
        background-repeat: no-repeat;
        background-position: right 0.75em center;
        background-size: 12px 12px;
        padding-right: 2.5em;

        &:hover {
          background-color: var(--col-surface);
          background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23e0def4' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
          background-repeat: no-repeat;
          background-position: right 0.75em center;
          background-size: 12px 12px;
        }

        &:focus {
          background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23c4a7e7' d='M6 9L1 4h10z'/%3E%3C/svg%3E");
          background-repeat: no-repeat;
          background-position: right 0.75em center;
          background-size: 12px 12px;
        }
      }

      input[type="number"]::-webkit-inner-spin-button,
      input[type="number"]::-webkit-outer-spin-button {
        opacity: 1;
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
        margin-top: auto;

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
</style>
