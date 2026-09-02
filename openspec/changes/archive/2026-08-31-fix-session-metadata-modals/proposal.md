## Why

編輯標籤與備註的對話框使用半透明面板，底下內容會穿透而降低閱讀性，且與 Skills 檢視對話框的呈現不一致。更嚴重的是，中繼資料雖已寫入 SQLite，session 增量掃描快取仍可能回傳舊資料，導致剛儲存的標籤與備註未出現在卡片或再次開啟的編輯器中。

## What Changes

- 提供可重用的實心 Modal 容器，統一遮罩、面板背景、邊框、陰影與內容版面，並沿用現有 theme token。
- 將標籤、單一標籤與備註編輯對話框改用實心 Modal，視覺與 Skills 檢視對話框一致且不透出背景內容。
- 讓標籤與備註儲存成功後立即更新所有相關 session 查詢快取，使 session 卡片及下一次編輯立即顯示新值。
- 確保後續 session 重新查詢或增量掃描不會以記憶體掃描快取中的舊 metadata 覆蓋已儲存的值。
- 從 Project 頁面的 session 卡片移除重複且非必要的 Git Repo 資訊，保留更新時間、建立時間與 Summary 數量。
- 增加共用 Modal 與 session metadata 儲存／回顯流程的自動化測試。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `ui-primitives`: 將內容密集的對話框從僅靠 CSS modifier 擴充為可重用、具一致遮罩與實心面板的 Modal 容器。
- `session-actions`: 標籤與備註儲存後必須立即顯示於正確的 session 卡片，重新開啟編輯器及後續重新查詢亦須保留最新值。
- `session-list`: Project 頁面的 session 卡片不再顯示 Git Repo 欄位，精簡重複的專案資訊。

## Impact

- 前端元件：`src/components/EditDialog.tsx`、`src/components/SessionCard.tsx`、Skills 預覽所使用的 Modal 結構，以及新增或調整的共用 UI 元件。
- 前端狀態：`src/App.tsx` 的 `saveMetaMutation` 與 React Query sessions cache 同步。
- 樣式：`src/App.css` 的 dialog backdrop、實心面板與共用 Modal 版面規則。
- 後端快取：session 掃描快取回傳 `SessionInfo` 前的 metadata 合併策略，可能涉及 `src-tauri/src/sessions/` 與 `src-tauri/src/db.rs`。
- API 與資料庫 schema 不變，不新增第三方相依套件。
