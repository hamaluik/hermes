import { load, type Store } from "@tauri-apps/plugin-store";
import { error as logError } from "@tauri-apps/plugin-log";

export class Settings {
  store: Store | null = null;

  // message editor
  private _tabsFollowCursor: boolean = true;

  // send/receive
  private _lastHostname: string = "127.0.0.1";
  private _lastPort: number = 2575;
  private _lastTransformControlId: boolean = true;
  private _lastTransformTimestamp: boolean = true;
  private _lastWaitTimeoutSeconds: number = 5;

  constructor() {
    load("settings.json", {
      autoSave: true,
    })
      .then((store) => {
        this.store = store;

        return Promise.all([
          store.get<boolean>("tabsFollowCursor"),
          store.get<string>("lastHostname"),
          store.get<number>("lastPort"),
          store.get<boolean>("lastTransformControlId"),
          store.get<boolean>("lastTransformTimestamp"),
          store.get<number>("lastWaitTimeoutSeconds"),
        ]);
      })
      .then(
        ([
          tabsFollowCursor,
          lastHostname,
          lastPort,
          lastTransformControlId,
          lastTransformTimestamp,
          lastWaitTimeoutSeconds,
        ]) => {
          this._tabsFollowCursor = tabsFollowCursor ?? true;
          this._lastHostname = lastHostname ?? "127.0.0.1";
          this._lastPort = lastPort ?? 2575;
          this._lastTransformControlId = lastTransformControlId ?? true;
          this._lastTransformTimestamp = lastTransformTimestamp ?? true;
          this._lastWaitTimeoutSeconds = lastWaitTimeoutSeconds ?? 5;
        },
      )
      .catch((error) => {
        console.error("Error loading settings store:", error);
        logError("Failed to load settings store");
      });
  }

  public async save(): Promise<void> {
    try {
      await this.store?.save();
    } catch (error) {
      console.error("Error saving settings:", error);
      logError("Failed to save settings");
    }
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

  get lastHostname(): string {
    return this._lastHostname;
  }
  set lastHostname(value: string) {
    console.debug("Setting lastHostname to:", value);
    this._lastHostname = value;
    if (this.store) {
      this.store.set("lastHostname", value).catch((error) => {
        console.error("Error saving lastHostname setting:", error);
        logError("Failed to save lastHostname setting");
      });
    }
  }

  get lastPort(): number {
    return this._lastPort;
  }
  set lastPort(value: number) {
    console.debug("Setting lastPort to:", value);
    this._lastPort = value;
    if (this.store) {
      this.store.set("lastPort", value).catch((error) => {
        console.error("Error saving lastPort setting:", error);
        logError("Failed to save lastPort setting");
      });
    }
  }

  get lastTransformControlId(): boolean {
    return this._lastTransformControlId;
  }
  set lastTransformControlId(value: boolean) {
    console.debug("Setting lastTransformControlId to:", value);
    this._lastTransformControlId = value;
    if (this.store) {
      this.store.set("lastTransformControlId", value).catch((error) => {
        console.error("Error saving lastTransformControlId setting:", error);
        logError("Failed to save lastTransformControlId setting");
      });
    }
  }

  get lastTransformTimestamp(): boolean {
    return this._lastTransformTimestamp;
  }
  set lastTransformTimestamp(value: boolean) {
    console.debug("Setting lastTransformTimestamp to:", value);
    this._lastTransformTimestamp = value;
    if (this.store) {
      this.store.set("lastTransformTimestamp", value).catch((error) => {
        console.error("Error saving lastTransformTimestamp setting:", error);
        logError("Failed to save lastTransformTimestamp setting");
      });
    }
  }

  get lastWaitTimeoutSeconds(): number {
    return this._lastWaitTimeoutSeconds;
  }
  set lastWaitTimeoutSeconds(value: number) {
    console.debug("Setting lastWaitTimeoutSeconds to:", value);
    this._lastWaitTimeoutSeconds = value;
    if (this.store) {
      this.store.set("lastWaitTimeoutSeconds", value).catch((error) => {
        console.error("Error saving lastWaitTimeoutSeconds setting:", error);
        logError("Failed to save lastWaitTimeoutSeconds setting");
      });
    }
  }
}
