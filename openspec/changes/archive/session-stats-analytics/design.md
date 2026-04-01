## Context

SessionHub 目前從 `workspace.yaml` 取得基本 session 元資料（summary、summaryCount、cwd），但每個 session 的 `events.jsonl` 包含豐富的使用事件（assistant.message、tool.execution_start/complete、session.start/model_change 等），這些數據從未被解析利用。

`events.jsonl` 是 append-only 的 JSONL 檔案，每個事件含 `type`、`data`、`timestamp`、`id`、`parentId`。一個活躍 session 的檔案大小可達數 MB（3-5 MB 常見），解析成本不可忽略，需要快取機制。

現有 SQLite（`metadata.db`）已儲存備註與標籤，可延伸儲存統計快取。

## Goals / Non-Goals

**Goals:**
- 解析 `events.jsonl` 提取可靠的統計指標：輸出 token 總量、使用者互動次數（頂層 user.message）、工具調用次數、對話時長、使用模型清單
- 將解析結果快取至 SQLite，透過 `parsed_at` vs 檔案 mtime 判斷是否需要重新解析
- Session 卡片顯示精簡徽章（時長、token、互動次數）
- Session 詳情面板顯示完整統計（工具分組、模型佔比、reasoning 次數）
- 專案頁面頂部彙總 banner（session 總數、總 token、總互動）

**Non-Goals:**
- 不解析 `inputTokens`（events.jsonl 不含此欄位）
- 不計算 Copilot premium request 確切數量（無可靠資料來源）
- 不跨 session 做時序分析或趨勢圖表（超出本次範疇）
- 不即時更新統計（進行中的 session 不需要即時反映）

## Decisions

### D1: 快取策略 — SQLite mtime 比對

**選擇**: 在 `session_stats` 表儲存 `events_mtime` (Unix timestamp)，每次取得統計前比對當前 `events.jsonl` 的 mtime。若一致則直接回傳快取；若不同則重新解析並更新。

**理由**: `events.jsonl` 是 append-only，mtime 變更等同內容更新。避免解析大型 JSONL 文件的重複 I/O 成本。

**替代方案考慮**: 記錄 file size（更快，但理論上可能誤判）；永遠不快取（最簡單，但效能差）；hash 比對（最準確但成本最高）。選 mtime 是可靠性與效能的最佳平衡。

### D2: 解析位置 — Rust backend command

**選擇**: 新增 `get_session_stats(session_dir: String) -> Result<SessionStats, String>` Tauri command，在 Rust 端以行讀取 JSONL 並直接 deserialize 需要的欄位。

**理由**: JSONL 解析屬 I/O 密集任務，在 Rust 執行比在 JS/TS 端更適合。且符合既有架構（所有 IPC 集中在 App.tsx invoke）。

**替代方案考慮**: 前端直接呼叫 `readTextFile` 解析 → 違反架構分層，且大型文件會阻塞 UI 執行緒。

### D3: 統計指標定義

- **outputTokens**: 所有頂層（非 subagent）`assistant.message.data.outputTokens` 之和
- **interactionCount**: 頂層 `user.message`（`data.parentToolCallId` 不存在）的數量
- **toolCallCount**: 頂層 `tool.execution_start`（`data.parentToolCallId` 不存在）的數量  
- **durationMinutes**: `session.start.data.startTime` 到最後一個事件 `timestamp` 的分鐘數
- **modelsUsed**: `session.start.data.selectedModel` + 所有 `session.model_change.data.newModel`（去重）
- **reasoningCount**: `assistant.message.data.reasoningOpaque` 不為 null 的頂層訊息數（作為進階請求估算）
- **toolBreakdown**: 各工具名稱的調用次數 map（`tool.execution_start.data.toolName`）

### D4: UI 呈現 — 卡片徽章 + 側滑/展開詳情

**選擇**: SessionCard 卡片底部新增一行精簡徽章（3-4 個數字），點擊卡片標題區域或新增「詳情」icon-button 展開 `SessionStatsPanel`（inline 展開於卡片下方，非 modal）。

**理由**: Modal 會遮蓋列表情境，inline 展開保持脈絡感。徽章只顯示最重要的 3 個指標不佔太多空間。

## Risks / Trade-offs

- **[Risk] events.jsonl 格式變更** → Mitigation: 使用 `#[serde(default)]` 對所有 optional 欄位容錯，解析失敗時回傳 partial stats 而非 error
- **[Risk] 大型 JSONL 導致 UI 卡頓** → Mitigation: Tauri command 在非同步執行緒執行，卡片初始渲染不等待 stats（stats 載入中顯示 skeleton）
- **[Risk] session 仍在進行中（有 inuse.*.lock）** → Mitigation: 標記 `isLive: true` 於 stats，UI 顯示「進行中」提示且不快取結果
- **[Risk] SQLite schema 擴充影響現有資料** → Mitigation: `CREATE TABLE IF NOT EXISTS` 並在 `init_db` 中 `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` 向前相容

## Migration Plan

1. Rust `init_db` 加入 `session_stats` 表建立語句（`IF NOT EXISTS`，無需 migration script）
2. 統計資料為可再生內容，無需遷移舊資料
3. 前端 stats 為獨立 Query key（`["session_stats", sessionDir]`），不影響現有 `["sessions", ...]` 查詢

## Open Questions

- Dashboard 彙總統計要在 sessions 查詢時一起計算，還是前端從已快取的 stats 彙整？→ 建議前端彙整以避免 backend 計算瓶頸
- 「進階請求數量」是否要用 reasoningCount 還是不顯示？→ 顯示但標注為「估算（含 reasoning 次數）」
