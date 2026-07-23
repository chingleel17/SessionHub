## Context

SessionHub currently emits Windows Toast notifications through its Tauri process and through provider hook scripts that execute `snoretoast.exe`. The Tauri path is associated with the installed SessionHub application, but the hook path supplies the display name `SessionHub` without a registered Windows Application User Model ID (AUMID) and matching Start-menu shortcut. Windows therefore falls back to the invoking terminal or a generic notification entry, which creates multiple notification groups and icons for the same product.

The change must preserve the hook path because it is the only notification path available while SessionHub is not running. It must work for the installed Windows application, and must not introduce a dependency on a running Tauri process.

## Goals / Non-Goals

**Goals:**

- Define one stable Windows AUMID for SessionHub notifications.
- Associate the installed SessionHub shortcut, executable, icon, Tauri notifications, and `snoretoast` hook notifications with that AUMID.
- Ensure all supported provider hooks produce notifications grouped under SessionHub with the SessionHub icon.
- Retain existing notification switches, per-session replacement, and hook failure isolation.

**Non-Goals:**

- Replacing `snoretoast.exe` with a long-running notification service or requiring SessionHub to be running.
- Changing notification triggers, text, interaction routing, or user-configurable notification preferences.
- Supporting non-Windows system notifications.

## Decisions

### Use the Tauri bundle identifier as the canonical AUMID

The implementation SHALL define the canonical Windows notification identity from the Tauri bundle identifier, `com.ching.sessionhub`, and use the same value from a single source where hook assets are generated. This identity is reverse-DNS shaped, stable across releases, and already represents the installed application rather than an individual provider or terminal process.

An arbitrary display-name AUMID such as `SessionHub` was rejected because it is not guaranteed to resolve to the installed application shortcut and can collide with unrelated software. Per-provider AUMIDs were rejected because they would intentionally recreate the fragmented notification grouping this change fixes.

### Register an AUMID-bearing Start-menu shortcut during Windows installation

The Windows installer SHALL create or update SessionHub's Start-menu shortcut with the canonical AUMID, target executable, and application icon. Windows uses this shortcut registration to resolve externally sent Toast notifications to the owning application and its icon.

Relying on `snoretoast`'s `-appID` argument alone was rejected because it does not register the app identity or associate its icon. Relying on the Tauri process was rejected because provider hooks must notify while SessionHub is closed.

### Pass the canonical AUMID to every hook notification

The common `notify.cjs` implementation for each installed provider SHALL invoke `snoretoast.exe` with the canonical AUMID. The source hook assets are updated together so Claude, Codex, and Copilot receive identical behavior on reinstallation or update.

Provider-specific notification logic remains unchanged. A shared Node module is retained rather than adding cross-process IPC, keeping hooks self-contained and failure-tolerant.

### Verify packaged behavior, not only hook arguments

Automated tests SHALL verify the canonical identity in generated hook assets and installer configuration. Manual or packaged integration verification SHALL confirm both the in-app Tauri path and a hook-only path are grouped as SessionHub with the bundled app icon in Windows Notification Center.

## Risks / Trade-offs

- [The development build may not have an installed AUMID-bearing shortcut] → Document and test the final installer artifact; retain the existing best-effort hook behavior for development.
- [Existing hook installations retain stale scripts until refreshed] → Bump the relevant hook asset version so integration update/reinstall writes the new `notify.cjs` files.
- [Windows installer tooling supports AUMID registration differently by target] → Implement and test the configuration for each supported Windows bundle target, and fail packaging validation if the installed shortcut lacks the canonical AUMID.
- [Tauri's notification plugin does not expose a direct AUMID option] → Use the installed executable and matching shortcut identity that Windows resolves for the Tauri process; do not add a second notification implementation.

## Migration Plan

1. Add the canonical identity and Windows installer shortcut registration to the packaged application.
2. Update shared hook notification assets to pass the canonical identity, then bump their asset versions.
3. Ship the update; existing users refresh provider integrations once through the existing update or reinstall action.
4. Roll back by restoring the previous hook assets and installer settings; notifications continue to send but may again be grouped by the fallback sender.

## Open Questions

- Confirm the exact Tauri/Windows bundler setting or installer customization required to attach the canonical AUMID to the Start-menu shortcut for both NSIS and MSI targets before implementation.
