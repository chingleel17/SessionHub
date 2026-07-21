# Tasks: Hook 原生系統通知

## 1. 通知資源與共用模組

- [x] 1.1 取得官方 `snoretoast.exe`（Windows x64），記錄來源 URL 與版本，放入 `hooks/_bin/snoretoast.exe`（或各 provider 共用位置）
- [x] 1.2 以 `snoretoast.exe -?` 確認實際旗標（`-t`/`-m`/`-appID`/`-tag`/`-group`/`-silent`），更新 design 中的參數約定
- [x] 1.3 新增 `hooks/copilot/modules/notify.cjs`：匯出 `sendNotification({ sessionId, title, body, kind })`，以 `child_process.spawnSync` 呼叫 snoretoast，`tag = sessionhub-${sessionId}`、`group = intervention`，失敗時寫 `hook-errors.log` 且不拋出
- [x] 1.4 新增 `notify.cjs` 的設定讀取：解析 `%APPDATA%\SessionHub\settings.json`，依 `kind`（完成 → `enableSessionEndNotification`、等待/授權 → `enableInterventionNotification`）決定是否發送；讀取失敗採安全預設（介入開、結束關）
- [x] 1.5 將 `notify.cjs` 同步複製到 `hooks/codex/modules/`（與 record-event.cjs 同模式）

## 2. Copilot hook 觸發點

- [x] 2.1 `hooks/copilot/on-session-end.cjs`：寫 bridge record 後呼叫 `sendNotification({ kind: "done", ... })`，標題「SessionHub — Session 已完成」，內容取 cwd 末段專案名
- [x] 2.2 `hooks/copilot/on-pre-tool-use.cjs`：呼叫 `sendNotification({ kind: "intervention", ... })`，標題「SessionHub — 需要您授權」
- [x] 2.3 確認「等待回應」對映的回合結束事件（Copilot `assistant.turn_end` 對應的 hook），加掛 `kind: "intervention"` 通知

## 3. Codex hook 觸發點

- [x] 3.1 `hooks/codex/on-stop.cjs`：加掛 `kind: "done"` 通知
- [x] 3.2 確認 Codex `PermissionRequest` 對應的 hook 腳本（若尚未存在則新增 `on-permission-request.cjs` 並註冊），加掛 `kind: "intervention"` 通知

## 4. Claude hook 觸發點

- [x] 4.1 對齊官方 `Notification` hook：matcher `permission_prompt` → `kind: "intervention"`（需授權）
- [x] 4.2 matcher `idle_prompt` → `kind: "intervention"`（等待回應）
- [x] 4.3 `Stop` hook → `kind: "done"`（完成）
- [x] 4.4 確認 Claude hook 安裝設定（settings.json 片段）正確帶入 snoretoast 呼叫或 notify.cjs 路徑

## 5. 安裝流程

- [x] 5.1 修改 `src-tauri` 的 `install_hook_scripts` 相關邏輯：複製 `snoretoast.exe` 與 `notify.cjs` 至各 provider 落地目錄
- [x] 5.2 確認 AppID 設定：若標題列需顯示「SessionHub」，於安裝時註冊 AppID（捷徑或登錄）或在 snoretoast 參數指定 `-appID`
- [x] 5.3 解除安裝時一併移除複製的 `snoretoast.exe` 與 `notify.cjs`

## 6. 去重與雙路徑驗證

- [x] 6.1 確認應用內路徑（`notifications.rs` / 前端 useEffect）與 hook 路徑使用相同 `tag = sessionhub-{session_id}`、`group = intervention`
- [x] 6.2 驗證 SessionHub 開啟時同一語意不疊加（最多顯示一則最新通知）

## 7. 測試與驗證

- [x] 7.1 SessionHub 關閉狀態下，分別以 Copilot/Codex/Claude 觸發「完成」「需授權」「等待回應」，確認 Toast 正常彈出
- [x] 7.2 關閉 `enable_intervention_notification` / `enable_session_end_notification`，確認對應通知不發送
- [x] 7.3 同一 session 連續觸發，確認通知取代而非疊加
- [x] 7.4 設定檔不存在 / 損毀時，確認採安全預設且不報錯（檢查 `hook-errors.log`）
- [x] 7.5 `.sh` hook 在非 Windows 路徑維持不發通知、不報錯
