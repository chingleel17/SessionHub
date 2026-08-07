## 1. 後端：共用定向送出 helper

- [x] 1.1 在 `src-tauri/src/lib.rs` 新增動態 webview label 清單常數（含 `QUOTA_OVERLAY_LABEL` 與 `TRAY_PANEL_LABEL`），作為定向送出的單一來源
- [x] 1.2 新增泛型 helper，對清單中每個 label 取得 webview 並送出指定事件與 payload；doc comment 註明「廣播 + 定向」會使 listener 收到重複事件，listener 必須 idempotent
- [x] 1.3 將 `emit_quota_snapshots_updated` 改為呼叫 helper，保留既有 AppHandle 廣播
- [x] 1.4 將 `emit_overlay_settings_changed` 改為呼叫 helper（原本無廣播，僅定向送 overlay，改為送往兩個動態 webview）
- [x] 1.5 將 `emit_intervention_list_changed` 改為呼叫 helper，保留既有 AppHandle 廣播
- [x] 1.6 執行 `cargo check`（於 `src-tauri/`）確認編譯通過

## 2. 前端：tray panel 即時同步兜底

- [x] 2.1 在 `src/app/EmbeddedTrayPanelApp.tsx` 以 `getCurrentWindow().onFocusChanged(...)` 監聽原生 focus 事件，於 focused=true 時 `refetchQueries` 重取 snapshot query；不使用 TanStack Query 的 `refetchOnWindowFocus`（理由見 design 決策二）
- [x] 2.2 將 snapshot query 的 `staleTime` 由 `60_000` 改為 `0`，避免 freshness 抑制 focus 觸發的重取
- [x] 2.3 於元件卸載時呼叫 `onFocusChanged` 回傳的 unlisten，與既有 listener 的清理邏輯一併處理
- [x] 2.4 確認既有 `quota-snapshots-updated` listener 的 invalidate 行為維持不變，與 focus 重取並存不衝突

## 3. 前端：enabled provider 過濾

- [x] 3.1 在 `EmbeddedTrayPanelApp` 新增 settings query 取得 `quotaEnabledProviders`，預設值比照 overlay 為全部 provider
- [x] 3.2 為 settings query 建立更新來源：監聽 `quota-overlay-settings-changed`（由任務 1.4 送達 panel）並以 payload 更新快取，同時將該 query 納入任務 2.1 的 focus 重取範圍
- [x] 3.3 在 `src/components/TrayQuotaPanel.tsx` 新增 `enabledProviders` prop，於 `sortSnapshots` 之前過濾未啟用的 provider
- [x] 3.4 由 `EmbeddedTrayPanelApp` 將 `enabledProviders` 傳入 `TrayQuotaPanel`

## 4. 驗證

- [x] 4.1 執行既有測試與型別檢查（`bun run build` 或專案既有的 tsc/test 指令），確認無回歸
- [x] 4.2 實機驗證：開啟 panel 與 overlay，觸發一次後端 quota 刷新，確認兩者數值同時更新且一致
- [x] 4.3 實機驗證：關閉 panel、等待背景刷新後重新開啟，確認 panel 顯示的是最新數值而非開啟前的舊值
- [x] 4.4 實機驗證：由 panel 開啟設定（webview 會 close 而非 hide），關閉某個 provider 的監控後重新點擊 tray 開啟 panel，確認 panel 不再列出該 provider 且快照為最新，與 overlay 一致
- [x] 4.5 執行 `openspec validate fix-tray-panel-quota-sync --strict` 確認變更文件通過驗證
