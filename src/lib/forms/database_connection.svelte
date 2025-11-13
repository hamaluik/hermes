<script lang="ts">
  import type { Settings } from "../../settings";

  let {
    settings,
    isValid = $bindable(false),
  }: {
    settings: Settings;
    isValid?: boolean;
  } = $props();

  // Local reactive state for database connection (synced with settings)
  let dbHost: string = $state(settings.wizardDbHost);
  let dbPort: number = $state(settings.wizardDbPort);
  let dbDatabase: string = $state(settings.wizardDbDatabase);
  let dbUser: string = $state(settings.wizardDbUser);
  let dbPassword: string = $state(settings.wizardDbPassword);

  // Sync changes back to settings
  $effect(() => {
    settings.wizardDbHost = dbHost;
  });
  $effect(() => {
    settings.wizardDbPort = dbPort;
  });
  $effect(() => {
    settings.wizardDbDatabase = dbDatabase;
  });
  $effect(() => {
    settings.wizardDbUser = dbUser;
  });
  $effect(() => {
    settings.wizardDbPassword = dbPassword;
  });

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
      bind:value={dbPassword}
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
