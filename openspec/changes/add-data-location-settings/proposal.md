## Why

各 AI CLI 工具（Claude Code、Codex、Copilot CLI、opencode）的 session 與設定資料預設全部存放在 C 槽使用者目錄下，使用者換電腦或 C 槽空間不足時，難以掌握資料位置與搬遷方式。各工具其實都提供官方環境變數可自訂資料目錄（`CLAUDE_CONFIG_DIR`、`CODEX_HOME`、`COPILOT_HOME`、`XDG_DATA_HOME`），但散落在各家文件中，手動設定容易遺漏「搬移既有資料」或「同步更新 SessionHub 讀取路徑」等步驟。SessionHub 作為 session 管理中心，適合提供統一的資料位置檢視與搬遷引導。

## What Changes

- 設定頁新增「資料位置」區塊：顯示各已啟用 provider 的資料目錄現況 — 目前解析後的實際路徑、是否為預設位置（或已透過環境變數/自訂設定改位置）、目錄佔用大小。
- 同時顯示 SessionHub 自身資料目錄（`%APPDATA%\SessionHub`，可由 `COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 覆寫）現況。
- 新增「搬遷引導」流程：使用者為某 provider 選擇新目錄後，SessionHub 依序執行 —
  1. 複製既有資料至新目錄（複製成功後才進行下一步，來源目錄保留由使用者自行刪除）
  2. 寫入使用者層級（User scope）環境變數（`CLAUDE_CONFIG_DIR` / `CODEX_HOME` / `COPILOT_HOME` / `XDG_DATA_HOME`）
  3. 同步更新 SessionHub 對應的 `claudeRoot` / `codexRoot` / `copilotRoot` / `opencodeRoot` 設定，使 session 掃描立即改讀新位置
- 引導完成後提示使用者：已開啟的終端機需重開才會讀到新環境變數；確認新位置運作正常後可自行刪除舊目錄。
- 新增後端指令：解析各 provider 實際資料路徑與環境變數狀態、計算目錄大小、執行資料複製、讀寫使用者層級環境變數。

## Capabilities

### New Capabilities
- `data-location-settings`: 設定頁的資料位置檢視與搬遷引導 — 各 provider 與 SessionHub 自身資料目錄的現況顯示（路徑、來源、大小）、搬遷流程（複製資料、設定環境變數、同步 root 設定）與完成後的提示。

### Modified Capabilities
- `app-settings`: 設定頁新增「資料位置」區塊入口；provider root 設定值在搬遷完成後由系統自動更新（原本僅供使用者手動填寫）。

## Impact

- **前端**：`src/App.tsx` 新增 IPC 呼叫；設定頁（Settings view）新增「資料位置」區塊與搬遷引導 UI；新增翻譯 key。
- **後端**：新增 commands（查詢資料位置現況、計算目錄大小、執行搬遷）；`settings.rs` 的 `default_*_root` 解析邏輯需納入環境變數（`CLAUDE_CONFIG_DIR`、`CODEX_HOME`、`COPILOT_HOME`、`XDG_DATA_HOME`）判斷。
- **系統層**：透過 Windows 使用者層級環境變數寫入（registry `HKCU\Environment`），不需管理員權限；不影響其他使用者。
- **風險**：搬移中若複製失敗需保持原狀（來源不動、不寫環境變數）；大目錄複製需有進度回報與取消機制。
