# Proposal: Hook 原生系統通知（不依賴 SessionHub 運行）

## Why

目前的「AI 介入通知」與「Session 結束通知」完全依賴 SessionHub 前端輪詢：前端 `useEffect` 偵測 `activity status` 由非 `waiting` 轉為 `waiting`（或 `done`）後才呼叫 Tauri command 發 Toast。這造成兩個實際痛點：

1. **只有 Copilot 會觸發** — `waiting` 狀態只有 Copilot（`assistant.turn_end`）與 OpenCode 會產生；Codex 的 activity 判定（`activity.rs`）永遠只回 `active`/`idle`，沒有 `waiting` 分支，所以 Codex 從不通知。
2. **不開應用就完全沒通知** — 整條通知鏈路（前端輪詢 → command → Toast）只在 SessionHub 視窗開著時運作。使用者多工時若沒開 SessionHub，AI 完成、需授權、需決策都不會有任何提示，必須手動切回終端查看。

`intervention-notification` spec 雖已定義「Hook 腳本通知（獨立路徑）」需求，但實際 hook 腳本（`record-event.cjs`）只寫 bridge record，從未實作任何 Toast 發送邏輯。OpenCode 之所以能在無 SessionHub 時跳通知，是因為它原生內建跨平台通知（Windows 透過隨附的 `snoretoast.exe`）。本提案即補上這條獨立於應用程式的通知路徑。

## What Changes

- 在 hooks 隨附一支 `snoretoast.exe`（Windows-only），由 hook 腳本直接呼叫發送系統 Toast，不經過 SessionHub 行程。
- 新增共用通知模組 `hooks/<provider>/modules/notify.cjs`，封裝「組標題/內容 → 呼叫 snoretoast → 以 `session_id` 為 tag 去重」邏輯，供各 provider hook 共用。
- 各 provider hook 在「完成（Stop/Session 結束）」與「等待回應/需決策」「需授權」事件點直接呼叫 `notify.cjs` 發通知：
  - **Copilot**：`on-session-end`（完成）、`on-user-prompt-submitted` 之後的回合結束、`on-pre-tool-use`（需授權）。
  - **Codex**：`on-stop`（完成）、`PermissionRequest`（需授權）。
  - **Claude**：對齊官方 `Notification` hook 的 `permission_prompt`（需授權）、`idle_prompt`（等待回應）與 `Stop`（完成）。
- hook 通知受開關控制：沿用 `enable_intervention_notification` / `enable_session_end_notification` 設定，hook 啟動時讀取已落地的設定快照決定是否發送（避免應用未開時無法讀設定）。
- SessionHub 開著時維持現有前端 Toast 路徑；以 `tag = sessionhub-{session_id}` 確保兩條路徑不重複疊加。

## Capabilities

### New Capabilities

- `hook-native-notification` — 定義 hook 腳本獨立發送系統通知的行為：觸發事件、`snoretoast.exe` 呼叫方式、去重 tag、開關讀取、與應用內通知路徑的關係。

### Modified Capabilities

- `intervention-notification` — 既有的「Hook 腳本通知（獨立路徑）」需求由「PowerShell WinRT 片段」改為「隨附 snoretoast.exe + notify.cjs 模組」實作；並補上 Codex/Claude 觸發點與「不依賴應用內 activity 狀態判定」的描述。

## Impact

- **新增檔案**：`hooks/<provider>/modules/notify.cjs`、隨附 `snoretoast.exe`（每個 provider 目錄或共用 bin 目錄）。
- **修改檔案**：`hooks/copilot/*.cjs`、`hooks/codex/*.cjs`、Claude hook 設定／腳本（觸發點加掛通知呼叫）。
- **安裝流程**：provider hook 安裝程序需一併複製 `snoretoast.exe` 與 `notify.cjs` 至落地目錄（`src-tauri` 的 `install_hook_scripts` 相關邏輯）。
- **設定**：`AppSettings` 的 `enable_intervention_notification` / `enable_session_end_notification` 需以快照形式供 hook 讀取（落地一份設定檔或環境變數）。
- **平台**：僅 Windows；`.sh` hook 路徑維持現狀（不發通知）。
