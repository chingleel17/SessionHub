## Context

目前 `EditDialog` 與 Skills 預覽都直接組合 `dialog-backdrop`、`dialog-card` class，但只有 Skills 預覽額外使用 `dialog-card--solid`，造成內容密集的標籤／備註表單仍使用半透明玻璃背景。這種重複標記也使各 Modal 容易產生不同的遮罩、語意與視覺設定。

Session metadata 由 `upsert_session_meta` 寫入 SQLite 的 `session_meta` 表。前端成功後只 invalidate `sessions` 查詢；後端增量掃描則可能直接複製記憶體 `ProviderCache.sessions`，而該快取中的 `notes`、`tags` 尚未更新，因此重新查詢仍可收到舊值。`sessions_cached` 查詢雖會從 SQLite 合併 metadata，但目前具無限 stale time，亦不會因這次 mutation 自動更新。

## Goals / Non-Goals

**Goals:**

- 建立一個純顯示、可重用的 Modal 容器，集中 backdrop、dialog semantics 與實心面板樣式。
- 讓 `EditDialog` 及 Skills 預覽採用同一容器，同時保留各自的尺寸與內容版面 class。
- 在 metadata 儲存成功時，以伺服器已確認的輸入原子更新所有前端 session query cache。
- 讓任何後端掃描路徑在回傳 session 前，以 SQLite `session_meta` 作為 notes 與 tags 的最終來源。
- 從 `SessionCard` 的中繼資訊格移除 Git Repo 項目，並讓剩餘三項資訊自然填滿可用寬度。
- 以測試涵蓋成功儲存、清除、重新查詢及 Modal 實心背景契約。

**Non-Goals:**

- 不變更 SQLite schema、Tauri command 名稱或 `SessionInfo` 型別。
- 不重做所有既有確認對話框；簡短 confirm 可繼續使用現有玻璃樣式。
- 不新增焦點鎖定、portal 或動畫套件，也不全面重構 Dialog 狀態管理。
- 不改變標籤正規化、搜尋或篩選規則。
- 不移除 `SessionInfo.repoName` 或其他畫面、分析與分組功能對 repository 資訊的使用。

## Decisions

### 1. 建立共用 Modal 外殼元件

在 `src/components/ui/` 提供 Modal 容器，接收 children 與可選的面板 class，輸出共用 backdrop 及具 `role="dialog"`、`aria-modal="true"` 的實心 dialog 面板。`EditDialog` 與 Skills 預覽只負責內容及功能專屬尺寸，不再自行重複 backdrop／card 結構。

選擇預設為實心面板，是因本次共用對象皆為表單或長文字，且需求明確要求不可穿透。簡短確認視窗仍可保留現行結構，避免把本次 bug fix 擴大成全站 Dialog 遷移。

替代方案是只在 `EditDialog` 加上 `dialog-card--solid`。雖能修正當下透明問題，但無法達成共用元件要求，重複的 Modal 語意與結構仍會持續分歧，因此不採用。

### 2. 儲存成功後直接修補所有 session query cache

`saveMetaMutation` 保留完整 mutation variables，於 `onSuccess` 針對 query key 前綴為 `sessions` 與 `sessions_cached` 的 `SessionInfo[]`，依穩定的 session id 替換 `notes` 與 `tags`。只有 IPC 成功後才更新 cache，避免 optimistic update 在失敗時需要回滾；之後仍 invalidate `sessions` 以與後端重新同步。

這讓目前卡片、編輯器閉合後的下一次開啟，以及以 cached placeholder 顯示的畫面立即一致。更新所有相符 query 而非只寫入目前完整 key，可涵蓋不同 provider/root/filter 組合已存在的快取。

替代方案是 mutation 後強制 full scan。它會增加所有 provider 的檔案掃描成本，且 UI 必須等待掃描完成才更新；因此僅作為錯誤修正並不合適。

### 3. 後端在組合完成後統一重新套用 metadata

`get_sessions_internal` 完成各 provider 的 full/incremental/cache sessions 合併後，逐筆從同一 SQLite connection 讀取 `session_meta`，覆寫 `SessionInfo.notes` 與 `SessionInfo.tags`，再排序並回傳。Provider cache 持續只負責來源 session 掃描資料；使用者可編輯的 metadata 則由 SQLite 表作為最終來源。

集中在共用出口可避免逐一修改 Copilot、OpenCode、Codex、Claude、Antigravity 等 provider 的增量快取邏輯，也確保新增 provider 經過共同流程時自動符合契約。資料量為每個 session 一次簡單主鍵查詢；目前已有 cached loader 採相同策略，風險可接受。

替代方案是讓 `upsert_session_meta` 同步更新 `ScanCache`。這會要求 command 注入並鎖定所有 provider cache，且必須搜尋 metadata 所屬 provider，增加鎖順序與耦合；因此不採用。

### 4. 測試分層驗證狀態與視覺契約

Rust 測試驗證 stale `ProviderCache` 經 metadata 更新後，再次取得 sessions 仍回傳 SQLite 最新值。前端測試若現有基礎允許，驗證 mutation 成功會更新正確 id 的 `sessions` 與 `sessions_cached` cache，失敗則不更新；元件測試驗證 EditDialog 與 Skills 預覽使用共用 Modal 的實心面板及 dialog semantics。若專案目前沒有 DOM 測試基礎，至少以 lint/build 加上可抽離的 cache updater 單元測試覆蓋資料邏輯，不為本次變更新增大型測試框架。

### 5. 僅移除 Session Card 的 Git Repo 呈現

從 `SessionCard` 的 `session-meta-grid` 移除 Git Repo label 與 `repoName` 值，但保留後端與 TypeScript 型別中的 `repoName`。Repository 資訊仍可供專案分組、搜尋或其他功能使用，本次只精簡 Project 頁面中已可由所在專案得知的重複欄位。

替代方案是從 `SessionInfo` 完全移除 `repoName`。這會不必要地擴大前後端型別與其他功能的影響範圍，不符合本次純顯示調整，因此不採用。

## Risks / Trade-offs

- [每次查詢產生 N 次 metadata 主鍵查詢] → 保持查詢為 SQLite 同連線與主鍵索引；若日後量測顯示瓶頸，再改為單次批次載入，不在本次提前最佳化。
- [前端 cache 修補與後端 refetch 競爭] → 後端出口同樣以 SQLite metadata 為最終來源，因此後續 refetch 應回傳相同值，不會把成功結果復原。
- [共用 Modal class 變更影響 Skills 預覽尺寸或捲動] → 容器只擁有共通視覺與語意，`agents-preview-modal` 繼續控制寬高、overflow 與內容捲動。
- [只遷移兩種 Modal，短期仍存在舊結構] → 刻意限制本次修正範圍；共用元件可供後續逐步遷移，不要求一次改完所有 Dialog。
- [移除第四個欄位後中繼資訊格可能留下不均衡空間] → 同步調整 grid 欄數或自適應規則，並在桌面與窄視窗確認三項資訊排列。

## Migration Plan

1. 新增共用 Modal 元件與樣式，遷移 `EditDialog`、Skills 預覽並驗證深色／淺色主題。
2. 從 `SessionCard` 移除 Git Repo 呈現並調整三欄版面。
3. 加入前端成功後 cache 更新，再保留 sessions invalidation 作背景同步。
4. 在後端 session 清單共同出口重新套用 SQLite metadata，並加入 regression tests。
5. 執行前端 lint/build、Rust tests 與 OpenSpec strict validation。

本變更無資料遷移。若需回復，可移除共用 Modal 使用及 cache 同步邏輯；`session_meta` 中既有資料不受影響。
