## Why

目前 SessionHub 主要依賴 filesystem watcher 判斷 session 是否更新，這會把 OpenCode 資料庫檔案波動、WAL checkpoint、以及非 session 核心檔案變更都誤判成「最近 session 變更」，導致使用者在所有 session 都已結束後仍持續看到即時更新。既然 Copilot CLI 與 OpenCode 都已有可掛接的 hook / plugin 能力，現在適合將 provider 原生事件升級為主要資料來源，並把檔案監看降級為 fallback。

## What Changes

- 新增 provider integration 機制：以 Copilot hook 與 OpenCode plugin 作為 session 更新事件的主要來源。
- 新增 provider bridge 設定與狀態管理：在設定頁顯示整合安裝狀態、設定檔位置，並提供快速開啟、編輯與安裝/更新入口。
- 修改現有即時更新流程：前端僅在收到 bridge 事件或 fallback 驗證成功時刷新 session 清單，減少誤報與無效重掃。
- 修改 filesystem watcher 行為：watcher 改為 fallback 機制，只監看必要路徑、過濾事件種類與目標檔案，並在 emit 前做 cheap verify。
- 為 OpenCode / Copilot 建立標準化事件格式與本地 bridge 儲存，讓 SessionHub 可以統一處理不同 provider 的更新訊號。

## Capabilities

### New Capabilities
- `provider-integration`: 管理 Copilot hook 與 OpenCode plugin 的安裝、橋接事件格式、狀態檢查與更新通知來源切換。

### Modified Capabilities
- `app-settings`: 設定頁需求擴充為可管理 provider 設定檔位置、整合狀態，並提供快速開啟、編輯與安裝操作。
- `file-watcher`: 即時更新需求由「直接依賴 filesystem watch」調整為「provider bridge 優先、filesystem watcher fallback」。

## Impact

- `src-tauri/src/lib.rs`：新增/調整 provider integration 狀態偵測、bridge 事件讀寫、watcher 過濾與 fallback 驗證流程。
- `src/App.tsx` 與設定相關元件：新增 provider integration 設定 UI、安裝狀態與快速開啟/編輯入口。
- provider 設定檔案：可能涉及 Copilot hook 設定與 OpenCode plugin 檔案的建立、更新與開啟。
- `openspec/specs/app-settings/spec.md`、`openspec/specs/file-watcher/spec.md`：需求變更。
- 新增 `openspec/specs/provider-integration/spec.md`：定義 bridge 事件與 provider 整合契約。
