## Why

過去曾觀察到 SessionHub 終端只列出 7 個專案 skills、手動終端列出 21 個（OpenCode 1.18.4）；這是歷史紀錄，並非目前已重現的問題。2026-11-04 從本專案目錄執行 OpenCode 1.19.5 的 `debug skill` 可列出 38 個，但尚未在同一時間比較 SessionHub 終端與手動終端。因此先確認問題是否仍存在，再決定是否需要修正啟動流程；避免在原因未明或問題已消失時改動子程序環境。

## What Changes

- 以官方 `opencode debug skill` 列出結果、計數並比較名稱與來源路徑；同時記錄版本、工作目錄、啟動模式及有效設定，避免將總數相同誤判為來源一致
- 在相同工作目錄、相近時間分別比較手動終端與 SessionHub shell 終端；若使用 herdr，另與 herdr pane 比較，辨識既有 server／client 環境影響
- 僅在可重現差異且證實由 SessionHub 啟動流程造成時，修正該路徑並建立回歸防護；若結果一致，記錄結論並停止實作
- 保留使用者現有環境設定與既有 MSYS 合併語意，不硬編 skills 路徑或注入 home 相關變數

## Capabilities

### New Capabilities

- `terminal-child-env-parity`: 定義 shell 子程序環境繼承、herdr 既有 pane 的環境邊界，以及可重複執行的 OpenCode skill 來源比對流程

### Modified Capabilities

- `terminal-launcher`: 保留既有 shell／herdr 分派與 MSYS stackdump 緩解情境，補充 shell 環境繼承與 herdr 不可視為新 spawn 之語意

## Impact

- **程式碼**
  - 診斷：`src-tauri/src/sessions/terminal.rs`、`sessions/herdr.rs`、`sessions/mod.rs`、`commands/tools.rs`、`sessions/copilot.rs` 及 `resource_discovery.rs` 的既有行為
  - 僅於重現且確定根因後修改相關啟動路徑；目前不預設需更動 `agents_config.rs` 的連結部署
- **外部相依**：OpenCode CLI `debug skill` 的目前 JSON 輸出；依實測版本與官方說明驗證，不綁定 1.18.4 的反組譯實作
- **平台**：Windows 的 SessionHub shell 與 herdr 啟動模式；連結遍歷是待查假說而非已確立根因
- **不影響**：session 掃描、quota、hook 整合等既有功能；保留原有命令、參數、工作目錄與 console creation flags
