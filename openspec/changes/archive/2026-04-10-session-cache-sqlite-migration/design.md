## Context

`metadata.db` 已是 SessionHub 的持久化核心，儲存 `session_meta`（備註/標籤）與 `session_stats`（統計快取）。目前 session 清單快取以 JSON 檔案（`session_cache.json`）獨立維護，增量掃描所需的 mtime 索引（`session_mtimes`）與 OpenCode cursor（`last_cursor`）僅存於記憶體 `ScanCache`，應用程式重啟後清空。

**現有關鍵限制：**

- `lib.rs` 是單一大檔（1700+ 行），無模組邊界。
- Tauri commands 全為同步 fn。
- 所有 IPC 集中在 `App.tsx`，子元件不得呼叫 `invoke()`。
- Windows only，路徑用 `PathBuf`。

## Goals / Non-Goals

**Goals:**

- 在 `metadata.db` 新增 `sessions_cache`、`scan_state`、`session_mtimes` 三個資料表。
- 應用程式啟動時，`ScanCache` 從 SQLite 恢復上次的 mtime 索引與 cursor，並以 `sessions_cache` 表提供即時顯示資料。
- 舊 `session_cache.json` 自動一次性遷移後刪除。
- 移除 `#[tauri::command] load_session_cache` 及相關 JSON 讀寫函式。

**Non-Goals:**

- 改變前端任何 UI 樣式或使用者可見行為。
- 引入新外部套件。
- macOS / Linux 驗證（架構不排除，此 change 不驗證）。

## Database Schema

```sql
-- session 清單快取（取代 session_cache.json）
CREATE TABLE IF NOT EXISTS sessions_cache (
    session_id   TEXT    NOT NULL,
    provider     TEXT    NOT NULL DEFAULT 'copilot',
    cwd          TEXT,
    summary      TEXT,
    summary_count INTEGER,
    created_at   TEXT,
    updated_at   TEXT,
    session_dir  TEXT,
    parse_error  INTEGER NOT NULL DEFAULT 0,
    is_archived  INTEGER NOT NULL DEFAULT 0,
    has_plan     INTEGER NOT NULL DEFAULT 0,
    has_events   INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (session_id, provider)
);

-- per-provider 掃描狀態（取代記憶體 ProviderCache 的持久化部分）
CREATE TABLE IF NOT EXISTS scan_state (
    provider           TEXT    NOT NULL PRIMARY KEY,
    last_full_scan_at  INTEGER NOT NULL DEFAULT 0,  -- unix timestamp secs
    last_cursor        INTEGER NOT NULL DEFAULT 0   -- opencode: max time_updated (ms)
);

-- per-session mtime 索引（Copilot 增量掃描用）
CREATE TABLE IF NOT EXISTS session_mtimes (
    session_id  TEXT    NOT NULL PRIMARY KEY,
    provider    TEXT    NOT NULL DEFAULT 'copilot',
    mtime       INTEGER NOT NULL DEFAULT 0  -- unix timestamp secs
);
```

## Decisions

### D1：sessions_cache 主鍵用 (session_id, provider)

**決定**：composite PK，避免不同 provider 的 session_id 碰撞（OpenCode `ses_xxx` 與 Copilot UUID 格式不同，但為安全起見仍使用複合主鍵）。

### D2：scan_state 的 last_full_scan_at 儲存格式

**決定**：unix timestamp 秒數（i64）。啟動時從 SQLite 讀取後轉為 `Instant::now() - Duration::from_secs(now - stored)`，以維持現有 `should_full_scan` 的 elapsed() 邏輯。

### D3：`load_session_cache` command 移除方式

**決定**：後端在 `get_sessions_internal` 中直接從 `sessions_cache` 表讀取並合併作為初始回傳值，`load_session_cache` tauri command 整體移除，前端不再單獨呼叫它。前端 `App.tsx` 移除對應的 `invoke("load_session_cache")` 呼叫，改依賴 `get_sessions` 的回傳結果（包含快取資料或新掃結果）。

### D4：舊 JSON 遷移策略

**決定**：在 `init_db` / `init_sessions_cache_tables` 首次呼叫時，若 `session_cache.json` 存在則讀取、寫入 `sessions_cache` 表後刪除 JSON 檔案。失敗時不中斷流程，只記錄警告。

## Data Flow

```
App 啟動
  ↓
init_db (含新三表)
  ↓
遷移 session_cache.json → sessions_cache 表（若 JSON 存在）
  ↓
get_sessions 呼叫
  ↓
從 scan_state 恢復 last_cursor / last_full_scan_at → 初始化 ScanCache
從 session_mtimes 恢復 mtime 索引 → 填入 ProviderCache
    ↓ (如果距上次全掃 > 30min 或快取空)   ↓ (否則)
    全量掃描                                增量掃描（利用恢復的 mtime/cursor）
  ↓
結果寫回 sessions_cache / scan_state / session_mtimes
  ↓
回傳 Vec<SessionInfo> 給前端
```
