<!--
  Database Connection Component

  Form fieldset for configuring database connection settings used by
  the wizard features. Note: Wizards currently return sample data and
  do not require an actual database connection.

  ## Auto-Save Pattern

  Settings are saved on every input change (oninput handler). This provides immediate
  persistence without requiring a separate Save button. The user's connection settings
  are always up-to-date, even if they navigate away from the settings UI.

  ## Why onMount for Loading?

  Settings are loaded in onMount rather than initial state declarations to avoid race
  conditions with the Settings object initialization. The Settings class loads from
  Tauri's persistent store asynchronously, so accessing properties during component
  initialization could read stale/default values.

  ## Validation and isValid Binding

  The component validates all fields against SQL Server naming rules (e.g., database
  names must start with letter/underscore, usernames can be domain-qualified like
  "DOMAIN\user"). The isValid binding allows parent components to conditionally enable
  actions (like wizard search buttons) based on whether the connection settings are
  complete and properly formatted.

  Validation is reactive ($derived) so it updates automatically as the user types,
  providing immediate feedback on whether their connection settings are valid.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import type { Settings } from "../../settings";

  let {
    settings,
    isValid = $bindable(false),
  }: {
    settings: Settings;
    isValid?: boolean;
  } = $props();

  // Local reactive state for database connection
  let dbHost: string = $state("");
  let dbPort: number = $state(1433);
  let dbDatabase: string = $state("");
  let dbUser: string = $state("");
  let dbPassword: string = $state("");

  // Load settings values after component mounts (avoids race condition)
  onMount(() => {
    dbHost = settings.wizardDbHost;
    dbPort = settings.wizardDbPort;
    dbDatabase = settings.wizardDbDatabase;
    dbUser = settings.wizardDbUser;
    dbPassword = settings.wizardDbPassword;
  });

  // Save to settings when local values change
  const saveToSettings = () => {
    settings.wizardDbHost = dbHost;
    settings.wizardDbPort = dbPort;
    settings.wizardDbDatabase = dbDatabase;
    settings.wizardDbUser = dbUser;
    settings.wizardDbPassword = dbPassword;
  };

  /**
   * Form Validation
   *
   * Validates against SQL Server rules:
   * - Host: Standard hostname/IP format (with optional port :1433 or instance \\SQLEXPRESS)
   * - Port: Standard TCP port range (0-65535)
   * - Database: Must start with letter/underscore, can contain letters/numbers/@#$
   * - User: Supports both standard SQL users and Windows auth (DOMAIN\user format)
   * - Password: Required, 1-128 characters
   */
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
      dbPassword.length >= 1 &&
      dbPassword.length <= 128
    );
  });

  // Sync validation state to bindable prop
  $effect(() => {
    isValid = isFormValid;
  });
</script>

<fieldset>
  <legend>Database Connection</legend>
  <div class="form-group">
    <label for="dbHost">Host</label>
    <input
      type="text"
      id="dbHost"
      name="dbHost"
      bind:value={dbHost}
      oninput={saveToSettings}
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
      oninput={saveToSettings}
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
      oninput={saveToSettings}
      minlength={1}
      maxlength={128}
      placeholder="mydb"
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
      oninput={saveToSettings}
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
      bind:value={dbPassword}
      oninput={saveToSettings}
      minlength={1}
      maxlength={128}
      placeholder="Password123"
      required={true}
    />
  </div>
</fieldset>

<style>
  .form-group {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    min-width: 8ch;
  }

  input {
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

  input[type="number"]::-webkit-inner-spin-button,
  input[type="number"]::-webkit-outer-spin-button {
    opacity: 1;
  }
</style>
