<script lang="ts">
  import { onMount } from "svelte";
  import IconClose from "../icons/IconClose.svelte";
    import IconSearch from "$lib/icons/IconSearch.svelte";

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

  const close = () => {
    if (dialogElement) {
      dialogElement.close();
    }
    onclose?.();
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
    <h1>Header Wizard</h1>
    <button class="close" onclick={close}>
      <IconClose />
    </button>
  </header>
  <main>
    <form> <!-- TODO: call wizardQueryInterfaces on submit to search for interfaces -->
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
          <select id="messageType" name="messageType">
            <option value="ADT">ADT</option>
            <option value="ORM">ORM</option>
          </select>
        </div>
        <div class="form-group">
          <label for="triggerEvent">Trigger Event</label>
          <select id="triggerevent" name="messageType">
            <option value="A01">A01 (Admit/visit notification)</option>
            <option value="A02">A02 (Transfer a patient)</option>
            <option value="A03">A03 (Discharge/end visit)</option>
            <option value="A04">A04 (Register a patient)</option>
            <option value="A05">A05 (Pre-admit a patient)</option>
            <option value="A06"
              >A06 (Change an outpatient to an inpatient)</option
            >
            <option value="A07"
              >A07 (Change an inpatient to an outpatient)</option
            >
            <option value="A08">A08 (Update patient information)</option>
            <!-- TODO: swap out the options depending on whether the messageType is an ADT or ORM -->
            <option value="O01">O01 (Order message)</option>
          </select>
        </div>
      </fieldset>
      <fieldset>
        <legend>Interface</legend>
        <!-- TODO: gussy this up -->
        <div class="form-group">
          <input type="submit"><IconSearch /> <span>Get Interfaces</span></input>
        </div>
      </fieldset>
    </form>
    <!-- TODO: inform the user if their search comes up empty -->
    <div class="results"> <!-- TODO: only show if there are results -->
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
        <!-- TODO: show 1 row per result after search; let the user click on the row to "select" it and highlight it -->
        </tbody>
      </table>
    </div>
  </main>
  <footer>
    <!-- TODO: display cancel/apply buttons; de-activate apply button if there is nothing selected; call wizardApplyInterface on clicking apply and update the mesasge then call onchange callback before onclose -->
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

      input,
      select {
        width: 100%;
      }
    }

    footer {
      width: 100%;
      height: 2.5em;
      border-radius: 8px 8px 0 0;
      display: flex;
      flex-direction: row;
      align-items: stretch;
      justify-content: flex-end;
      margin: 0;
      padding: 0;
      background: none;
    }
  }
</style>
