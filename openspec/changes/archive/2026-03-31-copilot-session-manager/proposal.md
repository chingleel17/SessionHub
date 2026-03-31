## Why

開發者在使用 GitHub Copilot CLI 時，累積了大量的 session 記錄，卻沒有任何 GUI 工具可以快速瀏覽、管理、或重新開啟這些 session。需要一個 Windows 桌面應用程式，讓開發者能直觀地管理 Copilot session 生命週期，節省手動翻找目錄的時間。

## What Changes

- 新增一個 Windows 桌面應用程式（Tauri 2 + React + TypeScript），可讀取 `~/.copilot/session-state/` 目錄
- 應用程式提供 Chrome 風格的分頁 UI：主頁面（Dashboard/統計）+ 每個專案一個分頁
- 每個 session 顯示：ID、summary（作為 session 標題，若存在）、cwd（專案路徑）、建立時間、更新時間、summary_count
- 支援依專案路徑分組與篩選 session 列表
- 可對 session 執行：封存、刪除、開啟終端（進入該 session）、複製開啟指令
- 終端程式（pwsh / powershell）路徑可在設定中自訂，並驗證可執行檔是否存在
- 支援亮色系 UI 主題（預留多主題擴充結構）
- 支援多語系架構，初版先提供繁體中文（`zh-TW`）
- 支援自訂 Copilot 根目錄（預設 `~/.copilot`）
- 支援對 session 新增自訂備註（notes）與標籤（tags），儲存於本地 SQLite DB
- 支援查看、App 內編輯、或開啟外部編輯器（可自訂，預設 VSCode）查看 session 的 `plan.md`
- 支援 filesystem watch 即時監聽 `session-state/` 目錄變更，自動更新 UI（不需手動重新整理）

## Capabilities

### New Capabilities

- `session-list`: 讀取並解析 `~/.copilot/session-state/` 下所有 session 的 `workspace.yaml`，回傳結構化 session 資料（含 id、summary 標題、cwd、建立時間、更新時間、summary_count）
- `session-grouping`: 依 `cwd`（專案路徑）將 session 分組，提供篩選與排序功能
- `session-actions`: 對指定 session 執行封存、刪除、開啟終端、複製開啟指令等操作
- `terminal-launcher`: 以可設定的終端程式（pwsh/powershell）開啟指定目錄，並驗證執行檔路徑
- `app-settings`: 儲存並讀取應用程式設定（Copilot 根目錄、終端路徑、UI 主題偏好）
- `dashboard`: 主頁面統計視圖，顯示 session 總數、最近活動、依專案分布等彙總資訊
- `tabbed-ui`: Chrome 風格分頁導航，主頁 + 每個專案群組各一分頁，支援分頁開關
- `localization`: 提供多語系資源載入與 UI 文案切換能力，初版先實作繁體中文
- `session-meta`: 對 session 新增自訂備註（notes）與標籤（tags），儲存於本地 SQLite DB
- `plan-viewer`: 讀取、App 內編輯、或開啟外部編輯器（可自訂，預設 VSCode）查看 session 的 `plan.md`
- `file-watcher`: 以 filesystem watch 即時監聽 `session-state/` 目錄變更，自動推送更新到前端 UI

### Modified Capabilities

（無現有 spec）

## Impact

- **新增應用程式**：全新專案
- **檔案系統存取**：唯讀存取 `~/.copilot/session-state/`；封存操作移動目錄；刪除操作移除目錄
- **外部相依**：需要系統安裝 pwsh 或 powershell；Rust + Tauri（需 Node.js + Cargo）；`notify` crate（filesystem watch）；`rusqlite` crate（SQLite）
- **設定持久化**：應用程式設定儲存於本機（`%APPDATA%\CopilotSessionManager\settings.json`），metadata DB 存於 `%APPDATA%\CopilotSessionManager\metadata.db`
- **即時更新**：透過 filesystem watch 監聽 session-state 目錄，無 Copilot CLI 官方 hook 機制，以 OS 檔案事件間接感知變化
