## Context

SessionHub 目前的即時更新主要仰賴 provider 資料夾與資料庫檔案的 filesystem watcher。這種做法雖然容易落地，但對 OpenCode 會把 `opencode.db`、`opencode.db-wal`、`opencode.db-shm` 等檔案波動都視為 session 更新；對 Copilot 也可能把非關鍵檔案事件放大成整體 session refresh。結果是使用者在 session 實際已結束時，仍看到持續同步或最近活動被重排。

這次變更除了要把 provider 原生事件升級為主要來源，也要補上管理面：讓使用者在設定頁看見 Copilot / OpenCode 的整合狀態、設定檔位置，並能快速開啟、編輯或由 SessionHub 自動建立必要的 bridge 檔案。

## Goals / Non-Goals

**Goals:**

- 以 provider bridge 事件作為 session 更新的主要來源。
- 讓 SessionHub 自動安裝或更新 OpenCode plugin 與 Copilot hook / bridge 設定。
- 在設定頁顯示 provider integration 狀態、設定檔位置與快速操作入口。
- 將 filesystem watcher 降級為 fallback，並在 emit 前做路徑/事件過濾與 cheap verify。
- 定義統一的 bridge 事件格式，讓前端與後端不必理解 provider 私有事件細節。

**Non-Goals:**

- 不重寫整個 session 掃描架構或移除既有 scan cache。
- 不要求 provider 提供官方 IPC API；若沒有，仍以可管理的本地檔案 bridge 實作。
- 不在此次變更中支援所有第三方 provider，只處理 Copilot 與 OpenCode。
- 不將 provider 設定檔管理擴充成完整 IDE 或文字編輯器，只提供快速開啟/編輯入口。

## Decisions

### D1：Provider bridge 採「provider 原生 hook/plugin → SessionHub bridge 檔案」模式

**決定**：Copilot hook 與 OpenCode plugin 不直接呼叫 Tauri IPC，而是把標準化事件寫入 SessionHub 管理的本地 bridge 檔案，再由 SessionHub 讀取並刷新。

**理由**：

- 跨程序最穩定，無需暴露本地 socket 或新增常駐服務。
- hook/plugin 可以獨立於 SessionHub 啟動與結束，不需依賴 app process state。
- bridge 檔案可保留最後事件，方便診斷與回放。

**替代方案**：

- 直接呼叫本地 HTTP / IPC：耦合高，失敗恢復差。
- 完全依賴 provider DB / 目錄 watcher：誤報過多，正是本次要解的問題。

### D2：SessionHub 優先管理使用者層級整合檔案，無法自動寫入時退回引導模式

**決定**：SessionHub 在可解析且可寫入的 provider 設定路徑下，自動建立或更新整合檔案；若路徑不存在、權限不足或 provider 不支援自動管理，設定頁顯示 `manual_required` 狀態，並提供快速開啟/編輯。

**理由**：

- 符合使用者期待的「能自動處理就自動處理」。
- 避免因環境差異或 provider 行為變動，讓自動安裝變成脆弱單點。
- 讓規格同時覆蓋自動與手動兩條路徑。

**替代方案**：

- 一律強制自動寫入：風險高，遇到權限或路徑差異容易失敗。
- 一律只給說明文件：體驗不足，無法解決使用者目前痛點。

### D3：Bridge 事件格式統一為 provider-neutral record

**決定**：定義標準 bridge record，至少包含 `provider`、`eventType`、`sessionId`、`cwd`、`timestamp`、`sourcePath`、`title` 等欄位；缺值允許為 null，但欄位名稱固定。

**理由**：

- 後端 refresh 決策只依賴統一欄位，不需要知道是 hook 還是 plugin。
- 後續若加入其他 provider，可沿用相同 bridge 契約。
- 可直接用於 debug 與狀態檢查。

### D4：Filesystem watcher 保留，但只作 fallback + verify

**決定**：保留現有 watcher 作為 fallback，僅監看關鍵路徑／檔案，並在收到 event 後先做 cheap verify（例如比對 session 目錄 mtime、cursor、或 bridge 狀態），只有確認 session 清單可能變更時才 emit UI refresh。

**理由**：

- provider integration 尚未安裝完成時，仍需基本即時更新能力。
- provider hook/plugin 失效時，有保底方案。
- 能顯著降低目前「任何 FS event 都刷新」的誤報。

### D5：設定頁新增 provider integration 區塊

**決定**：在 Settings 中增加 provider integration 管理區，顯示每個 provider 的：

- integration 狀態（installed / outdated / missing / manual_required / error）
- 設定檔或插件路徑
- 最後事件時間 / 最後錯誤
- 操作按鈕：安裝、更新、重新檢查、快速開啟、直接編輯

**理由**：

- 讓使用者理解目前是 bridge 模式還是 fallback 模式。
- 對除錯與支援非常重要，避免黑箱感。

## Risks / Trade-offs

- **[Provider 行為變動]** → 用版本化 bridge 檔案與狀態檢查緩解；若解析失敗，自動退回 fallback。
- **[權限或路徑不可寫]** → 顯示 `manual_required`，提供快速開啟與編輯，不阻塞 app 主要功能。
- **[Bridge 檔案累積或損毀]** → 採 append-only + 截斷策略，讀取失敗時保留錯誤訊息並退回 fallback。
- **[雙重事件來源造成重複 refresh]** → bridge 事件與 watcher verify 都需做短時間去重。

## Migration Plan

1. 新版本啟動後，先讀取 provider integration 狀態。
2. 若使用者啟用 provider 且環境可管理，SessionHub 可一鍵安裝 bridge。
3. 安裝成功後，bridge 成為主要更新來源；watcher 保留為 fallback。
4. 若 bridge 初始化失敗，UI 明確顯示 fallback 模式與錯誤原因。
5. 回滾時只需移除或停用 managed integration，既有 watcher 路徑仍可運作。

## Open Questions

- Copilot 使用者層級 hook 路徑在不同 CLI 版本上的相容性，實作時需以偵測結果為準。
- 是否要在設定頁提供「僅使用 fallback watcher」的顯式切換開關，可在實作時視 UI 複雜度決定。
