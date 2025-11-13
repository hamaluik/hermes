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
        ]) => {
          this._tabsFollowCursor = tabsFollowCursor ?? true;
          this._editorHeight = editorHeight ?? 200;
          this._sendHostname = sendHostname ?? "127.0.0.1";
          this._sendPort = sendPort ?? 2575;
          this._sendTransformControlId = sendTransformControlId ?? true;
          this._sendTransformTimestamp = sendTransformTimestamp ?? true;
          this._sendWaitTimeoutSeconds = sendWaitTimeoutSeconds ?? 5;
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
}
