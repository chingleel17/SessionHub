## Context

Copilot CLI 近期更新將 agent 任務追蹤從 `plan.md` 改至每個 session 的 `session.db`（SQLite）。SessionHub 目前不讀取此檔，造成使用者無法從 UI 追蹤 agent 進度。同時，SessionHub 的 stats 採懶讀取設計，只在使用者點開 session 時才解析 `events.jsonl`，造成 dashboard 的統計數字嚴重不完整（分析顯示目前有 80 個 session 缺少統計）。

## Goals / Non-Goals

**Goals:**
- 在 session 詳細頁讀取並顯示 `session.db` 的 todos 表（id、title、status、description）
- 將 Plan 與任務入口整合到 session 卡片同一個 badge 區域，避免入口分散
- 讓使用者可從任務摘要 badge 開啟獨立任務分頁，而非只能展開卡片檢視
- app 啟動及 file watcher 觸發時，自動對已完成但缺少 `session_stats` 的 session 補算統計

**Non-Goals:**
- 修改或寫入 `session.db`（完全唯讀）
- 顯示 todos 的歷史版本或 diff
- 對 OpenCode sessions 補算（OpenCode 無 `session.db`）
- 即時 poll `session.db`（只在使用者切換 session 時讀取）

## Decisions

### 1. Todos：on-demand 讀取，不快取

**決定**：點開 session 時呼叫一次 `read_session_todos`，結果不寫入 `metadata.db`。

**理由**：`session.db` 只在 live session 期間頻繁變更；completed session 的 todos 是靜態的。快取會增加 schema 複雜度，收益低。若 session 是 live，前端可依 `isLive` 每 N 秒 refetch（與現有 stats refetch 同步）。

**替代方案**：快取至 `metadata.db` → 需要 todos schema migration，複雜度不值得。

### 2. Stats backfill：app 啟動時批次補算，上限 50 個

**決定**：在 `get_sessions` 掃描完成後，於背景非同步對最多 50 個缺 `session_stats` 的已完成 session（有 `events.jsonl` + 無 lock 檔）進行補算，不阻塞 UI 回應。

**理由**：補算是 CPU-bound 操作，限制上限防止冷啟動卡頓。已有 lock 偵測邏輯（`is_live_session`），可直接重用。

**替代方案**：file watcher 觸發補算 → 複雜，且 watcher 是監聽 live session；補算更適合做成一次性批次。

### 3. `session.db` 讀取：直接開 SQLite，不透過 Copilot CLI API

**決定**：用 `rusqlite` 直接讀取 `<session_dir>/session.db`，查 `todos` 表。

**理由**：Copilot CLI 無公開 API；`session.db` 是標準 SQLite 格式；`rusqlite` 已是現有相依。

**風險**：Copilot CLI 可能在未來版本更改 `session.db` schema → Mitigation：用 `SELECT` + graceful error handling，schema 不符時回傳空清單。

### 4. Session 卡片摘要 badge：統一顯示 stats / plan / todos 入口

**決定**：將 `hasPlan` 與任務摘要都移到 session 卡片底部的摘要 badge 區，與互動數、token、時長、LIVE 同列顯示。

**理由**：Plan 與任務都是「這個 session 的工作入口」，放在同一區塊可讓使用者在同一視線範圍內完成辨識與點擊，不必同時在 header chip 與 action icon 之間來回找入口。

**顯示規則**：
- Plan badge 僅在 session 有 plan 時顯示，點擊後直接開啟既有 Plan 編輯分頁
- 任務 badge 僅在 `todos.length > 0` 時顯示，至少包含總任務數
- 各任務狀態 badge（done / pending / in_progress / blocked 或其他狀態）僅在對應 count > 0 時顯示

### 5. 任務查看：使用 ProjectView 子分頁，而不是只放在展開區塊

**決定**：任務查看沿用 ProjectView 既有 sub-tab 模式，新增以 session 為單位的 todos 分頁；點擊任務總數或狀態 badge 都導向同一個該 session 的 todos 分頁。

**理由**：使用者已熟悉 Plan 分頁行為；todos 與 Plan 同樣屬於 session 細節內容，使用同一種「可切換、可保留多個分頁」的互動模式一致性較高，也能避免卡片展開內容過長。

**狀態管理**：ProjectView 的 sub-tab state 需能同時記錄 plan 與 todos 類型，避免同一 session 的 plan / todos 分頁彼此衝突。

## Risks / Trade-offs

- **`session.db` schema 變更**：Copilot CLI 為第三方，不保證 schema 穩定 → 讀取時使用 `try_get` 處理缺欄位，回傳 `Ok(vec![])` 而非 error
- **backfill 影響啟動效能**：50 個 session × 平均 3MB events.jsonl 需要一定 I/O → 在 Tauri `async_runtime` 的 spawn_blocking 執行，不阻塞主執行緒
- **`todos` 表不存在的舊 session**：`session.db` 可能只有 metadata 沒有 todos → 查不到表時安靜回傳空陣列
- **badge 過多造成摘要列擁擠**：當任務狀態很多時，badge 數量可能壓縮既有 stats 顯示 → 僅顯示 count > 0 的狀態，並維持單列可掃讀的簡潔順序

## Migration Plan

1. 部署後首次啟動自動觸發 backfill（無需使用者操作）
2. 無資料遷移，只新增讀取路徑
3. Rollback：移除兩個新 command 即可，無 schema 變更需 rollback
