## 1. Windows Application Identity

- [x] 1.1 Confirm the Tauri/Windows bundler mechanism that registers `com.ching.sessionhub` on the installed Start-menu shortcut for every supported Windows installer target.
- [x] 1.2 Configure the Windows bundle so the installed SessionHub executable, Start-menu shortcut, and bundled icon share the canonical `com.ching.sessionhub` application identity.
- [x] 1.3 Add a packaging-level verification that the installed shortcut exposes the canonical application identity and SessionHub icon.

## 2. Hook Notification Assets

- [x] 2.1 Update the shared Claude, Codex, and Copilot `notify.cjs` assets to pass `com.ching.sessionhub` to `snoretoast.exe`.
- [x] 2.2 Bump the affected provider hook asset versions so the existing integration update/reinstall flow deploys the revised notification modules.
- [x] 2.3 Add unit tests that verify every generated hook notification asset uses the canonical identity without changing notification settings, tags, groups, or failure handling.
- [x] 2.4 Bundle the SessionHub logo with hook notification assets and pass it to `snoretoast.exe` for every hook Toast.

## 3. Verification

- [x] 3.1 Run the applicable Rust unit tests and frontend build checks.
- [x] 3.2 Build and install a Windows package, then verify an application-process Toast and a hook-only Toast are both grouped as SessionHub and display the SessionHub icon while the app is closed.
