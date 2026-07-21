# SessionHub Review Instructions

Review changes with the repository conventions in `AGENTS.md` and `CONTRIBUTING.md`.

Prioritize findings that can cause data loss, security exposure, incorrect quota values, broken provider parsing, failed Tauri IPC contracts, or regressions on Windows. Verify that frontend and Rust types remain aligned, new commands are registered, user-facing text uses i18n, and filesystem paths do not hardcode Windows separators.

Do not request unrelated refactors. Treat generated build output and local session data as non-source files. Never reproduce secrets, tokens, or session contents in review comments.
