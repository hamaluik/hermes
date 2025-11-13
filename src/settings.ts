import { load, type Store } from "@tauri-apps/plugin-store";
import { error as logError } from "@tauri-apps/plugin-log";

export class Settings {
  store: Store | null = null;

  // message editor
  private _tabsFollowCursor: boolean = true;
  private _editorHeight: number = 200;

  // send/receive
  private _sendHostname: string = "127.0.0.1";
  private _sendPort: number = 2575;
  private _sendTransformControlId: boolean = true;
  private _sendTransformTimestamp: boolean = true;
  private _sendWaitTimeoutSeconds: number = 5;

  // wizard database connection
  private _wizardDbHost: string = "";
  private _wizardDbPort: number = 1433;
  private _wizardDbDatabase: string = "";
  private _wizardDbUser: string = "";
  private _wizardDbPassword: string = "";

  constructor() {
    load("settings.json", {
      autoSave: true,
    })
      .then((store) => {
        this.store = store;

        return Promise.all([
          store.get<boolean>("tabsFollowCursor"),
          store.get<number>("editorHeight"),
          store.get<string>("sendHostname"),
          store.get<number>("sendPort"),
          store.get<boolean>("sendTransformControlId"),
          store.get<boolean>("sendTransformTimestamp"),
          store.get<number>("sendWaitTimeoutSeconds"),
          store.get<string>("wizardDbHost"),
          store.get<number>("wizardDbPort"),
          store.get<string>("wizardDbDatabase"),
          store.get<string>("wizardDbUser"),
          store.get<string>("wizardDbPassword"),
        ]);
      })
      .then(
        ([
          tabsFollowCursor,
          editorHeight,
          sendHostname,
          sendPort,
          sendTransformControlId,
          sendTransformTimestamp,
          sendWaitTimeoutSeconds,
          wizardDbHost,
          wizardDbPort,
          wizardDbDatabase,
          wizardDbUser,
          wizardDbPassword,
        ]) => {
          this._tabsFollowCursor = tabsFollowCursor ?? true;
          this._editorHeight = editorHeight ?? 200;
          this._sendHostname = sendHostname ?? "127.0.0.1";
          this._sendPort = sendPort ?? 2575;
          this._sendTransformControlId = sendTransformControlId ?? true;
          this._sendTransformTimestamp = sendTransformTimestamp ?? true;
          this._sendWaitTimeoutSeconds = sendWaitTimeoutSeconds ?? 5;
          this._wizardDbHost = wizardDbHost ?? "";
          this._wizardDbPort = wizardDbPort ?? 1433;
          this._wizardDbDatabase = wizardDbDatabase ?? "";
          this._wizardDbUser = wizardDbUser ?? "";
          this._wizardDbPassword = wizardDbPassword ?? "";
        },
      )
      .catch((error) => {
        console.error("Error loading settings store:", error);
        logError("Failed to load settings store");
      });
  }

  get tabsFollowCursor(): boolean {
    return this._tabsFollowCursor;
  }
  set tabsFollowCursor(value: boolean) {
    console.debug("Setting tabsFollowCursor to:", value);
    this._tabsFollowCursor = value;
    if (this.store) {
      this.store.set("tabsFollowCursor", value).catch((error) => {
        console.error("Error saving tabsFollowCursor setting:", error);
        logError("Failed to save tabsFollowCursor setting");
      });
    }
  }

  get editorHeight(): number {
    return this._editorHeight;
  }
  set editorHeight(value: number) {
    console.debug("Setting editorHeight to:", value);
    this._editorHeight = value;
    if (this.store) {
      this.store.set("editorHeight", value).catch((error) => {
        console.error("Error saving editorHeight setting:", error);
        logError("Failed to save editorHeight setting");
      });
    }
  }

  get sendHostname(): string {
    return this._sendHostname;
  }
  set sendHostname(value: string) {
    console.debug("Setting sendHostname to:", value);
    this._sendHostname = value;
    if (this.store) {
      this.store.set("sendHostname", value).catch((error) => {
        console.error("Error saving sendHostname setting:", error);
        logError("Failed to save sendHostname setting");
      });
    }
  }

  get sendPort(): number {
    return this._sendPort;
  }
  set sendPort(value: number) {
    console.debug("Setting sendPort to:", value);
    this._sendPort = value;
    if (this.store) {
      this.store.set("sendPort", value).catch((error) => {
        console.error("Error saving sendPort setting:", error);
        logError("Failed to save sendPort setting");
      });
    }
  }

  get sendTransformControlId(): boolean {
    return this._sendTransformControlId;
  }
  set sendTransformControlId(value: boolean) {
    console.debug("Setting sendTransformControlId to:", value);
    this._sendTransformControlId = value;
    if (this.store) {
      this.store.set("sendTransformControlId", value).catch((error) => {
        console.error("Error saving sendTransformControlId setting:", error);
        logError("Failed to save sendTransformControlId setting");
      });
    }
  }

  get sendTransformTimestamp(): boolean {
    return this._sendTransformTimestamp;
  }
  set sendTransformTimestamp(value: boolean) {
    console.debug("Setting sendTransformTimestamp to:", value);
    this._sendTransformTimestamp = value;
    if (this.store) {
      this.store.set("sendTransformTimestamp", value).catch((error) => {
        console.error("Error saving sendTransformTimestamp setting:", error);
        logError("Failed to save sendTransformTimestamp setting");
      });
    }
  }

  get sendWaitTimeoutSeconds(): number {
    return this._sendWaitTimeoutSeconds;
  }
  set sendWaitTimeoutSeconds(value: number) {
    console.debug("Setting sendWaitTimeoutSeconds to:", value);
    this._sendWaitTimeoutSeconds = value;
    if (this.store) {
      this.store.set("sendWaitTimeoutSeconds", value).catch((error) => {
        console.error("Error saving sendWaitTimeoutSeconds setting:", error);
        logError("Failed to save sendWaitTimeoutSeconds setting");
      });
    }
  }

  get wizardDbHost(): string {
    return this._wizardDbHost;
  }
  set wizardDbHost(value: string) {
    console.debug("Setting wizardDbHost to:", value);
    this._wizardDbHost = value;
    if (this.store) {
      this.store.set("wizardDbHost", value).catch((error) => {
        console.error("Error saving wizardDbHost setting:", error);
        logError("Failed to save wizardDbHost setting");
      });
    }
  }

  get wizardDbPort(): number {
    return this._wizardDbPort;
  }
  set wizardDbPort(value: number) {
    console.debug("Setting wizardDbPort to:", value);
    this._wizardDbPort = value;
    if (this.store) {
      this.store.set("wizardDbPort", value).catch((error) => {
        console.error("Error saving wizardDbPort setting:", error);
        logError("Failed to save wizardDbPort setting");
      });
    }
  }

  get wizardDbDatabase(): string {
    return this._wizardDbDatabase;
  }
  set wizardDbDatabase(value: string) {
    console.debug("Setting wizardDbDatabase to:", value);
    this._wizardDbDatabase = value;
    if (this.store) {
      this.store.set("wizardDbDatabase", value).catch((error) => {
        console.error("Error saving wizardDbDatabase setting:", error);
        logError("Failed to save wizardDbDatabase setting");
      });
    }
  }

  get wizardDbUser(): string {
    return this._wizardDbUser;
  }
  set wizardDbUser(value: string) {
    console.debug("Setting wizardDbUser to:", value);
    this._wizardDbUser = value;
    if (this.store) {
      this.store.set("wizardDbUser", value).catch((error) => {
        console.error("Error saving wizardDbUser setting:", error);
        logError("Failed to save wizardDbUser setting");
      });
    }
  }

  get wizardDbPassword(): string {
    return this._wizardDbPassword;
  }
  set wizardDbPassword(value: string) {
    console.debug("Setting wizardDbPassword to:", value);
    this._wizardDbPassword = value;
    if (this.store) {
      this.store.set("wizardDbPassword", value).catch((error) => {
        console.error("Error saving wizardDbPassword setting:", error);
        logError("Failed to save wizardDbPassword setting");
      });
    }
  }
}
