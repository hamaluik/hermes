# Application Updates

Hermes includes automatic update detection and installation using Tauri's
updater plugin. Updates are checked periodically in the background and users
are prompted when new versions are available.

## Update Detection

A background thread checks for updates on two occasions: once at startup after
a short delay, and periodically every four hours while the application runs.
The startup delay allows the interface to render before potentially showing an
update dialog, avoiding a jarring experience.

Periodic checks run silently. If an update is found during a periodic check,
the application caches the information but doesn't interrupt the user. The
cached state allows the "Check for Updates" menu item to show the available
version immediately without making a network request.

## Update Flow

When an update is detected, the application shows a dialog explaining that a
new version is available. Users can choose to install immediately or defer
until later. Deferring simply dismisses the dialog; the update remains cached
and the user can install it manually from the Help menu.

Choosing to install triggers a download of the update package. Once downloaded,
the update installs and the application restarts automatically. The restart
happens immediately after installation completes, so users should save any work
before accepting an update.

## Signature Verification

All updates are cryptographically signed. The application contains a public key
that verifies update signatures before installation. This prevents tampering
with update packages, whether by malicious actors intercepting the download or
compromised distribution infrastructure.

The signing key pair is generated locally and the private key stored securely.
GitHub Actions uses the private key during release builds to sign artifacts.
The public key embedded in the application cannot be changed without releasing
a new version, ensuring users always verify against a trusted key.

## Release Distribution

Updates are distributed through GitHub Releases. When a new version is tagged,
GitHub Actions builds platform-specific packages and uploads them along with a
JSON manifest describing the release. The application fetches this manifest to
check for updates, comparing the available version against the running version.

The release workflow generates updater artifacts automatically when building.
These artifacts include the update package and its signature, ready for the
updater to download and verify.

## Platform Considerations

On Windows, the updater uses a basic UI mode that shows installation progress.
This provides feedback during the update without requiring administrator
privileges until the actual installation begins.

On macOS, updates install as application bundles. The application must not be
code-signed with a hardened runtime that prevents self-modification, or the
updater will fail to replace the running binary.

## Related Documentation

- [Backend Architecture](backend.md) — Plugin initialisation and managed state
- [Communication Patterns](communication.md) — Dialog interactions during
  updates
