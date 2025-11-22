/**
 * Application settings management using Tauri's persistent store plugin.
 *
 * Provides a reactive settings system where changes are automatically saved to
 * disk and available across app restarts. Uses a getter/setter pattern to
 * intercept property changes and trigger persistence.
 *
 * ## Settings Persistence Flow
 *
 * 1. At app startup, Settings instance is created
 * 2. Constructor asynchronously loads settings.json using Tauri store plugin
 * 3. Store values override the default values defined in private fields
 * 4. Settings object is made available to components via context or props
 * 5. When user changes a setting (e.g., sets sendPort = 3000):
 *    - Setter updates the private field
 *    - Setter calls store.set() to persist to disk
 *    - Tauri store plugin auto-saves to AppData/settings.json
 * 6. On next app launch, saved values are loaded from disk
 *
 * ## Why Async Constructor?
 *
 * Tauri store loading is asynchronous because it involves IPC with the Rust
 * backend. The constructor starts the load but doesn't block. Settings are
 * available immediately with defaults, then updated when the store loads.
 * This prevents the UI from blocking during startup.
 *
 * ## Change Notification Callbacks
 *
 * Some settings need to notify external systems when they change. For example:
 * - Recent files list needs to update the native File menu
 * - Auto-save setting needs to update the File menu's checkable Auto-Save item
 * - Theme setting needs to update the document's data-theme attribute
 *
 * This is handled via optional callback properties (e.g., `onRecentFilesChanged`,
 * `onAutoSaveChanged`, `onThemeChanged`).
 *
 * The callback pattern is used instead of events because:
 * - It's simpler than setting up a full event emitter
 * - Only one listener is needed (the page component)
 * - The callback is called both on setter changes AND after initial load
 *
 * ## Default Values
 *
 * Defaults are chosen to match typical development environments:
 * - sendHostname: "127.0.0.1" (localhost testing)
 * - sendPort: 2575 (standard HL7 MLLP port)
 * - wizardDbPort: 1433 (SQL Server default port)
 * - tabsFollowCursor: true (better UX for most users)
 * - autoSaveEnabled: false (explicit opt-in, matches VS Code default)
 * - themeSetting: "auto" (follows system preference, most intuitive default)
 * - editorHeight: 200px (fits typical screen layouts)
 * - recentFiles: [] (empty list, populated as user opens files)
 */

import { load, type Store } from "@tauri-apps/plugin-store";
import { error as logError } from "@tauri-apps/plugin-log";

export class Settings {
  store: Store | null = null;

  // Message editor preferences
  private _tabsFollowCursor: boolean = true;
  private _editorHeight: number = 200;
  private _autoSaveEnabled: boolean = false;

  // Callback to notify when auto-save setting changes (for menu updates)
  onAutoSaveChanged: ((enabled: boolean) => void) | null = null;

  // Theme setting: "light", "dark", or "auto" (follows system preference)
  private _themeSetting: "light" | "dark" | "auto" = "auto";

  // Callback to notify when theme setting changes (for applying theme to document)
  onThemeChanged: ((theme: "light" | "dark" | "auto") => void) | null = null;

  // Send/Receive configuration for MLLP client
  private _sendHostname: string = "127.0.0.1";
  private _sendPort: number = 2575;
  private _sendTransformControlId: boolean = true;
  private _sendTransformTimestamp: boolean = true;
  private _sendWaitTimeoutSeconds: number = 5;

  // Wizard database connection settings for HL7 system integration
  private _wizardDbHost: string = "";
  private _wizardDbPort: number = 1433;
  private _wizardDbDatabase: string = "";
  private _wizardDbUser: string = "";
  private _wizardDbPassword: string = "";

  // Recent files list (most recent first, max 10 entries)
  private _recentFiles: string[] = [];

  // Callback to notify when recent files change (for menu updates)
  onRecentFilesChanged: ((files: string[]) => void) | null = null;

  /**
   * Initializes settings by loading from persistent store.
   *
   * Loads settings.json with autoSave enabled, meaning every store.set() call
   * automatically triggers a save to disk. Falls back to defaults if the store
   * fails to load or if individual settings are not present in the file.
   */
  constructor() {
    load("settings.json", {
      autoSave: true,
    })
      .then((store) => {
        this.store = store;

        return Promise.all([
          store.get<boolean>("tabsFollowCursor"),
          store.get<number>("editorHeight"),
          store.get<boolean>("autoSaveEnabled"),
          store.get<"light" | "dark" | "auto">("themeSetting"),
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
          store.get<string[]>("recentFiles"),
        ]);
      })
      .then(
        ([
          tabsFollowCursor,
          editorHeight,
          autoSaveEnabled,
          themeSetting,
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
          recentFiles,
        ]) => {
          this._tabsFollowCursor = tabsFollowCursor ?? true;
          this._editorHeight = editorHeight ?? 200;
          this._autoSaveEnabled = autoSaveEnabled ?? false;
          this._themeSetting = themeSetting ?? "auto";
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
          this._recentFiles = recentFiles ?? [];

          // Notify listeners that settings are loaded (for initial menu population)
          if (this.onRecentFilesChanged) {
            this.onRecentFilesChanged(this._recentFiles);
          }
          if (this.onAutoSaveChanged) {
            this.onAutoSaveChanged(this._autoSaveEnabled);
          }
          if (this.onThemeChanged) {
            this.onThemeChanged(this._themeSetting);
          }
        },
      )
      .catch((error) => {
        console.error("Error loading settings store:", error);
        logError("Failed to load settings store");
      });
  }

  // All getters/setters follow the same pattern:
  // - Getter returns the private field value (immediate, synchronous)
  // - Setter updates the private field AND persists to store
  // - Persistence is async but errors are logged, not thrown
  // - This ensures settings are immediately available to the UI even if save fails

  /** Whether segment tabs should automatically switch when cursor moves to a different segment */
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

  /** Height of the message editor in pixels (user-resizable) */
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

  /** Whether auto-save is enabled (automatically saves open files on changes) */
  get autoSaveEnabled(): boolean {
    return this._autoSaveEnabled;
  }
  set autoSaveEnabled(value: boolean) {
    console.debug("Setting autoSaveEnabled to:", value);
    this._autoSaveEnabled = value;
    if (this.store) {
      this.store.set("autoSaveEnabled", value).catch((error) => {
        console.error("Error saving autoSaveEnabled setting:", error);
        logError("Failed to save autoSaveEnabled setting");
      });
    }
    if (this.onAutoSaveChanged) {
      this.onAutoSaveChanged(value);
    }
  }

  /** Theme setting: "light" for light mode, "dark" for dark mode, "auto" to follow system */
  get themeSetting(): "light" | "dark" | "auto" {
    return this._themeSetting;
  }
  set themeSetting(value: "light" | "dark" | "auto") {
    console.debug("Setting themeSetting to:", value);
    this._themeSetting = value;
    if (this.store) {
      this.store.set("themeSetting", value).catch((error) => {
        console.error("Error saving themeSetting setting:", error);
        logError("Failed to save themeSetting setting");
      });
    }
    if (this.onThemeChanged) {
      this.onThemeChanged(value);
    }
  }

  /** Target hostname/IP for sending HL7 messages via MLLP */
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

  /** Target port for sending HL7 messages via MLLP (typically 2575) */
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

  /**
   * Whether to auto-generate MSH.10 (control ID) when sending messages.
   * When true, replaces the control ID with a UUID to ensure uniqueness.
   */
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

  /**
   * Whether to auto-update MSH.7 (timestamp) when sending messages.
   * When true, replaces the timestamp with the current date/time.
   */
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

  /**
   * Maximum seconds to wait for a response after sending an HL7 message.
   * If no response is received within this time, the send operation fails.
   */
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

  /**
   * Database hostname for wizard queries.
   * Wizards query database to populate message fields with real data.
   */
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

  /** List of recently opened file paths (most recent first, max 10) */
  get recentFiles(): string[] {
    return this._recentFiles;
  }
  set recentFiles(value: string[]) {
    console.debug("Setting recentFiles to:", value);
    this._recentFiles = value;
    if (this.store) {
      this.store.set("recentFiles", value).catch((error) => {
        console.error("Error saving recentFiles setting:", error);
        logError("Failed to save recentFiles setting");
      });
    }
    if (this.onRecentFilesChanged) {
      this.onRecentFilesChanged(value);
    }
  }

  /**
   * Adds a file path to the recent files list.
   * If the file already exists in the list, it is moved to the front.
   * The list is limited to 10 entries.
   */
  addRecentFile(filePath: string) {
    // Remove the file if it already exists (to move it to front)
    const filtered = this._recentFiles.filter((f) => f !== filePath);
    // Add to front and limit to 10
    this.recentFiles = [filePath, ...filtered].slice(0, 10);
  }

  /** Clears the recent files list */
  clearRecentFiles() {
    this.recentFiles = [];
  }
}
