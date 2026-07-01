## Context

SessionHub 目前的通知是「應用內路徑」：前端 `src/App.tsx` 的 `useEffect`（約 1204 行）監聽 `activityStatusQuery` 結果，當某 session 由非 `waiting` 轉 `waiting` 或進入 `done` 時，呼叫 `send_intervention_notification` Tauri command（`src-tauri/src/commands/notifications.rs`）發 Toast。

兩個限制使其不符使用者需求：

- **Codex 永不進入 `waiting`**：`src-tauri/src/activity.rs` 的 `get_codex_activity_status` 只回 `active`（30 分內有活動）或 `idle`，沒有 `waiting` 分支，因此前端永遠收不到 Codex 的 `waiting` 轉換。
- **依賴應用程式運行**：整條鏈路在 SessionHub 視窗關閉時失效。

hook 腳本（`hooks/<provider>/*.cjs`）是各 CLI 工具在生命週期事件時由 CLI 本身執行的短命 Node 程序，與 SessionHub 是否開啟無關。現行 hook 只透過 `record-event.cjs` 寫 bridge record。OpenCode 原生通知在 Windows 是呼叫隨附的 `snoretoast.exe`，這正是「離線可用」的關鍵：通知由 hook 程序自己發，不需要任何長駐應用。

## Goals / Non-Goals

**Goals:**

- hook 腳本在 SessionHub 未開啟時，仍能於「完成」「等待回應/需決策」「需授權」三類事件發送 Windows 系統通知。
- 統一三個 provider（Copilot/Codex/Claude）的觸發點，不再依賴前端 activity 狀態判定，解決「只有 Copilot 觸發」。
- 通知可被既有設定開關控制，且同一 session 不重複疊加。
- SessionHub 開著時不產生重複通知。

**Non-Goals:**

- 不支援 macOS/Linux（`.sh` hook 維持現狀，不發通知）。
- 不重寫應用內前端通知路徑；本變更只新增獨立的 hook 路徑。
- 不為 Codex 補 `activity.rs` 的 `waiting` 判定（改由 hook 事件直接觸發，見 Decisions）。
- 不實作通知點擊聚焦（hook 路徑下 SessionHub 可能未開，無 callback 可執行；點擊聚焦僅應用內路徑提供）。

## Decisions

### D1：以 hook 事件直接觸發，取代 activity 狀態推導

通知改由各 provider 的 hook 在對應事件點直接發送，而非靠應用端從 events 推導 `waiting`。對映：

| 語意 | Copilot | Codex | Claude |
| --- | --- | --- | --- |
| 完成 | `on-session-end` | `on-stop` | `Stop` |
| 等待回應/需決策 | 回合結束（`on-user-prompt-submitted` 後的 turn end） | — | `Notification` matcher `idle_prompt` |
| 需授權 | `on-pre-tool-use` | `PermissionRequest` | `Notification` matcher `permission_prompt` |

理由：hook 事件是 CLI 原生且離線可用的信號源，語意明確，且天然涵蓋 Codex（不必改 Rust 判定邏輯）。

### D2：隨附 `snoretoast.exe` + 共用 `notify.cjs` 模組

- 在 hook 安裝時複製一支 `snoretoast.exe`（Windows-only，約 0.5MB）至落地目錄；`notify.cjs` 以 `child_process` 呼叫它。
- snoretoast 參數約定：`-t <title> -m <body> -appID <AppID> -tag sessionhub-<sessionId> -group intervention -silent`（依實際 snoretoast 版本旗標調整）。`-tag`/`-group` 達成同一 session 取代而非疊加。
- 選 snoretoast 而非 PowerShell WinRT：啟動快、可設定 AppID 與圖示、與 OpenCode 行為一致；代價是需納入 binary。
- `notify.cjs` 永不拋出（對齊 `record-event.cjs` 的靜默失敗原則），失敗只寫 `hook-errors.log`。

### D3：設定快照供離線讀取

`enable_intervention_notification` / `enable_session_end_notification` 目前存於 `%APPDATA%\SessionHub\settings.json`。hook 啟動時直接讀取該檔的快照決定是否發送：

- 完成類事件 → 受 `enable_session_end_notification` 控制。
- 等待/授權類事件 → 受 `enable_intervention_notification` 控制。
- 讀檔失敗或欄位缺失 → 採安全預設（`enable_intervention_notification` 預設開、`enable_session_end_notification` 預設關，對齊前端現有預設）。

無需新增 IPC；hook 直接讀已落地的 settings.json。

### D4：雙路徑去重

應用內路徑與 hook 路徑都以 `tag = sessionhub-{session_id}`、`group = intervention` 發送。同 tag 的通知在 Windows 通知中心會取代而非累積，因此即使兩條路徑在 SessionHub 開啟時同時觸發，使用者最多看到一則最新通知，不會疊加。

## Risks / Trade-offs

- **重複通知**：SessionHub 開著時，hook 與前端可能對同一語意各發一次。`tag` 去重可避免疊加，但仍可能各觸發一次（時間差）。可接受；必要時後續可讓 hook 偵測 SessionHub 是否運行再決定是否發。
- **snoretoast AppID 與圖示**：未註冊 AppID 時 Toast 會掛在 snoretoast 預設 AppID 下，標題列顯示可能非「SessionHub」。需在安裝時確認 AppID 設定；此為已知微調項，列入 tasks。
- **binary 納入版本庫**：`snoretoast.exe` 進 repo 會增加體積並需信任來源。採用官方釋出版並記錄來源與版本。
- **設定快照時效**：使用者改設定後 hook 讀到的是落地檔，與前端記憶體狀態間有寫檔延遲；影響極小。
- **參數差異**：不同 snoretoast 版本旗標略有差異，實作時需以實際版本驗證一次。
