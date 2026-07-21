## Why

Windows 上的 Claude Code、Codex、OpenCode 等工具會使用 Git for Windows / MSYS2 Bash；當 Bash 子程序在結束流程中原生崩潰時，`msys-2.0.dll` 會在當時的工作目錄留下 `bash.exe.stackdump`。這是已回報的第三方執行階段問題，但 SessionHub 可在它啟動的工具程序樹注入 MSYS 的錯誤處理選項，避免 dump 檔持續污染專案目錄。

## What Changes

- SessionHub 在 Windows 啟動 terminal、Claude、Codex、Copilot、OpenCode 與 Gemini 時，為新程序樹加入抑制 MSYS crash dump 寫檔的環境選項。
- 合併既有 `MSYS` 環境變數而非覆蓋使用者選項，且重複套用時不產生重複 token。
- 將環境處理集中在共用 helper，覆蓋新開工具與 resume session 的所有啟動路徑。
- 增加環境選項合併、Windows 程序組態及非 Windows 無作用的單元測試。
- 不自動刪除既有 `bash.exe.stackdump`，也不宣稱修復 MSYS2 / Git for Windows 或各 AI coding CLI 的原生崩潰。

## Capabilities

### New Capabilities
- `msys-stackdump-suppression`: 規範由 SessionHub 啟動的 Windows 工具程序樹抑制 MSYS stackdump 寫檔，並安全保留既有 MSYS 設定。

### Modified Capabilities
- `terminal-launcher`: 所有可啟動終端或 CLI 的 Windows 路徑必須套用相同的 MSYS stackdump 緩解環境。

## Impact

- Rust 工具啟動：`src-tauri/src/commands/tools.rs`、`src-tauri/src/sessions/copilot.rs`，以及最接近程序啟動職責的共用 helper。
- 只影響 SessionHub 所產生之 Windows 子程序的環境；CLI 參數、provider integration、hook 格式與 bridge 事件不變。
- 無新增前端 API、資料庫 schema 或第三方套件。
