## Why

Tray mini panel 顯示的 quota 內容不會即時更新，與 overlay widget 不同步：overlay 收到後端刷新後立即改變，panel 卻停留在舊數值。

根因在 `src-tauri/src/lib.rs` 的三個事件發送函式。它們都採「AppHandle 廣播 + 定向補送」的模式，但定向補送只送往 `QUOTA_OVERLAY_LABEL`，從未送往 `TRAY_PANEL_LABEL`。`lib.rs:48` 的註解已寫明補送存在的理由——AppHandle 廣播對動態建立的 webview 不可靠。tray panel 由 `toggle_tray_panel`（`lib.rs:203`）以 `WebviewWindowBuilder` 動態建立，屬於完全相同的情況，卻是唯一沒有拿到這層保護的視窗。

overlay 之所以看起來正常，除了有定向補送，其 snapshot query 另有 `refetchInterval: 15_000` 的輪詢兜底；tray panel 兩者皆無，事件一旦遺漏即永久停在舊值，直到使用者手動按刷新。

已逐一查證所有寫入 snapshot 的路徑（`app_setup.rs:124`、`app_setup.rs:163`、`watcher.rs:443`、`commands/quota.rs:166`）皆有呼叫 `emit_quota_snapshots_updated`，因此問題不在發送時機，而在定向補送的收件對象。

## What Changes

- 將定向補送抽為共用 helper，統一送往 overlay 與 tray panel 兩個動態 webview；`emit_quota_snapshots_updated`、`emit_overlay_settings_changed`、`emit_intervention_list_changed` 三者共用，避免日後新增動態視窗再次遺漏。
- tray panel 前端在視窗顯示時主動重取 quota 快照，作為事件遺漏的兜底。不採用 overlay 的 `refetchInterval` 常駐輪詢——panel 每次只顯示數秒、其餘時間為 hidden（非 close），常駐輪詢屬無謂耗用。
- 修正 tray panel 未套用 enabled provider 過濾的問題。`get_quota_snapshots`（`commands/quota.rs:119`）直接回傳快取全量、不做過濾，而 `EmbeddedTrayPanelApp` 僅傳入 `snapshots`、未傳 `quotaEnabledProviders`，導致 panel 可能列出使用者已關閉監控的 provider，與既有 spec「顯示所有 enabled provider」不符。

非破壞性變更：不動事件名稱、不動 payload 結構、不動既有設定欄位。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `tray-quota-widget`: 「Tray 點擊彈出 Mini Panel」需求新增內容即時同步與 enabled provider 過濾的行為要求，明確規範 panel 與 overlay 顯示一致。

## Impact

- `src-tauri/src/lib.rs` — 三個 emit 函式改用共用定向補送 helper（行為擴充，簽章不變）
- `src/app/EmbeddedTrayPanelApp.tsx` — 顯示時重取快照、傳入 enabled provider
- `src/components/TrayQuotaPanel.tsx` — 新增 `enabledProviders` prop 並據以過濾
- 不影響 overlay 現有行為、不影響主視窗、不涉及資料庫或設定檔遷移
