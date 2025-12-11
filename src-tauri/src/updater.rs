//! Background update checker for Hermes.
//!
//! This module provides automatic update detection and installation using
//! `tauri-plugin-updater`. Updates are checked on startup (after a 5-second
//! delay to let the app initialise) and periodically every 4 hours.
//!
//! # Architecture
//!
//! The update system decouples detection from installation:
//!
//! 1. A background thread runs `check_for_update()` periodically
//! 2. When an update is found, metadata is cached in static variables
//! 3. The menu's "Check for Updates" item reads this cached state
//! 4. Users choose when to install via a dialog prompt
//!
//! This approach avoids interrupting users with update prompts while still
//! keeping them informed that updates are available.
//!
//! # Static State
//!
//! Three static variables track update state:
//!
//! - `UPDATE_AVAILABLE`: Atomic bool for quick "is update ready?" checks
//! - `UPDATE_VERSION`: Cached version string for display in dialogs
//! - `UPDATE_INFO`: The full `Update` object needed for installation
//!
//! Using statics (rather than app state) simplifies access from the menu
//! event handler, which runs in a different context than Tauri commands.

use color_eyre::eyre::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::{Update, UpdaterExt};

/// Whether an update is available. Checked by menu to show update option.
pub static UPDATE_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// Stored update info for deferred installation.
static UPDATE_INFO: OnceLock<Mutex<Option<Update>>> = OnceLock::new();

/// Version string of available update, for display in dialogs.
static UPDATE_VERSION: OnceLock<Mutex<Option<String>>> = OnceLock::new();

const CHECK_INTERVAL: Duration = Duration::from_secs(4 * 60 * 60); // 4 hours
const STARTUP_DELAY: Duration = Duration::from_secs(5);

/// Starts the background update checker thread.
///
/// Checks once after a short startup delay (showing a dialog if an update is
/// available), then periodically every 4 hours (silently).
pub fn start_update_checker(app: AppHandle) {
    std::thread::spawn(move || {
        // initial check after short delay to let app fully start
        std::thread::sleep(STARTUP_DELAY);
        check_for_update(&app, true);

        // periodic checks (silent)
        loop {
            std::thread::sleep(CHECK_INTERVAL);
            check_for_update(&app, false);
        }
    });
}

/// Performs an update check asynchronously.
///
/// If `show_dialog` is true, shows a dialog prompting the user to install.
fn check_for_update(app: &AppHandle, show_dialog: bool) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match do_check(&app).await {
            Ok(Some(version)) => {
                log::info!("update available: v{version}");
                if show_dialog {
                    show_update_dialog(&app, &version);
                }
            }
            Ok(None) => {
                log::debug!("no update available");
            }
            Err(e) => {
                log::error!("failed to check for updates: {e:#}");
            }
        }
    });
}

async fn do_check(app: &AppHandle) -> Result<Option<String>> {
    let updater = app.updater().wrap_err("can get updater")?;
    let update = updater.check().await.wrap_err("can check for updates")?;

    if let Some(update) = update {
        let version = update.version.clone();

        UPDATE_AVAILABLE.store(true, Ordering::SeqCst);

        // store version for display
        let version_lock = UPDATE_VERSION.get_or_init(|| Mutex::new(None));
        *version_lock.lock().expect("can lock update version") = Some(version.clone());

        // store update for later installation
        let update_lock = UPDATE_INFO.get_or_init(|| Mutex::new(None));
        *update_lock.lock().expect("can lock update info") = Some(update);

        Ok(Some(version))
    } else {
        Ok(None)
    }
}

/// Checks for updates on demand (triggered by user clicking menu item).
///
/// Returns the version string if an update is available, or None if up to date.
pub async fn check_now(app: &AppHandle) -> Result<Option<String>, String> {
    do_check(app).await.map_err(|e| format!("{e:#}"))
}

/// Downloads and installs the pending update, then restarts the app.
pub async fn install_update(app: &AppHandle) -> Result<(), String> {
    let update = {
        let lock = UPDATE_INFO.get_or_init(|| Mutex::new(None));
        lock.lock().expect("can lock update info").take()
    };

    if let Some(update) = update {
        log::info!("installing update v{}", update.version);

        update
            .download_and_install(|_chunk, _total| {}, || {})
            .await
            .map_err(|e| format!("failed to download and install update: {e}"))?;

        UPDATE_AVAILABLE.store(false, Ordering::SeqCst);

        // clear stored version
        if let Some(lock) = UPDATE_VERSION.get() {
            *lock.lock().expect("can lock update version") = None;
        }

        // restart never returns
        app.restart();
    }

    Err("no update available to install".to_string())
}

/// Shows a dialog offering to install an available update.
pub fn show_update_dialog(app: &AppHandle, version: &str) {
    let app_clone = app.clone();
    let msg = format!(
        "Version {version} is available.\n\n\
         Would you like to download and install it now?\n\n\
         The app will restart after the update is installed."
    );
    app.dialog()
        .message(msg)
        .title("Update Available")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install Update".to_string(),
            "Later".to_string(),
        ))
        .show(move |accepted| {
            if accepted {
                let app = app_clone.clone();
                std::thread::spawn(move || {
                    tauri::async_runtime::block_on(async {
                        if let Err(e) = install_update(&app).await {
                            log::error!("failed to install update: {e}");
                            app.dialog()
                                .message(format!("Failed to install update: {e}"))
                                .title("Update Failed")
                                .kind(MessageDialogKind::Error)
                                .show(|_| {});
                        }
                    });
                });
            }
        });
}

/// Handles the "Check for Updates" menu item click.
///
/// Shows a dialog with the result (either an update is available or we're up to
/// date).
pub fn handle_check_updates(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        tauri::async_runtime::block_on(async {
            match check_now(&app).await {
                Ok(Some(version)) => {
                    show_update_dialog(&app, &version);
                }
                Ok(None) => {
                    app.dialog()
                        .message("You're running the latest version.")
                        .title("No Updates Available")
                        .kind(MessageDialogKind::Info)
                        .show(|_| {});
                }
                Err(e) => {
                    log::error!("failed to check for updates: {e}");
                    app.dialog()
                        .message(format!("Failed to check for updates: {e}"))
                        .title("Update Check Failed")
                        .kind(MessageDialogKind::Error)
                        .show(|_| {});
                }
            }
        });
    });
}
