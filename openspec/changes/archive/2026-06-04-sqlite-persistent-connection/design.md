## Context

SessionHub 的 Rust 後端目前使用**每次請求建立新連線**模式：每個 Tauri command 都呼叫 `open_db_connection()` 開啟 SQLite 檔案，許多地方還額外呼叫 `init_db()` 執行 `CREATE TABLE IF NOT EXISTS`，command 結束後連線自動 drop。雖然 `open_db_connection_and_init()` 已有 `DB_SCHEMA_INITIALIZED` AtomicBool 防止重複初始化，但大多數 command 並未使用它，每次仍需承擔 1~4ms 的連線開銷，且 SQLite 頁面快取隨連線關閉失效。

現有 Tauri managed state 範例：`ScanCache` 以 `Mutex<Option<ProviderCache>>` 存放在記憶體掃描快取，已確立模式可直接沿用。

## Goals / Non-Goals

**Goals:**
- 應用程式啟動時建立一個 SQLite 連線，持續至程式結束
- 所有 Tauri command 改從 managed state 取得連線，消除重複 file open 與 schema check
- 維持現有 `_internal` helper 接受 `&Connection` 的設計，保留測試可注入性
- 啟用 WAL journal mode，提升並發讀取效能

**Non-Goals:**
- 連線池（connection pool）：桌面 app 單一寫入者，不需要多連線
- 跨 process 共享連線
- 前端可見的行為變更

## Decisions

### 1. 新增 `DbState` 而非併入 `ScanCache`

`ScanCache` 職責是掃描快取；DB 連線是獨立的基礎設施。分開存放符合單一職責原則，且日後若需擴充（如統計、設定分離的 DB）不會污染 ScanCache。

`DbState` 定義：
```rust
pub(crate) struct DbState {
    pub(crate) conn: Mutex<Connection>,
}
```

在 `run()` 中 `.manage(DbState::new()?)` 於啟動時初始化。

**替代方案考慮**：`Mutex<Option<Connection>>` → 無必要的 Option，連線在整個 app 生命週期內必定有效，使用 direct `Mutex<Connection>` 更清晰。

### 2. WAL Mode 於初始化時啟用

`init_db` 在 `PRAGMA journal_mode=WAL` 後再建立 schema，讓並發讀取不阻塞寫入。這對 `get_session_stats`（大量讀取）與 `upsert_session_meta`（寫入）同時發生時有益。

### 3. 測試保留 `open_db_connection()` / in-memory connection

Tauri command 函式是薄包裝層（lock state → call `_internal`），測試直接呼叫 `_internal` 並注入 `Connection::open_in_memory()` 或測試專用路徑的連線，不依賴 Tauri state。`open_db_connection()` 保留但限測試使用。

### 4. Lock Poisoning 處理

`Mutex::lock()` 在前一個持有者 panic 時回傳 `PoisonError`。統一用 `map_err(|e| format!("db lock poisoned: {e}"))` 轉換為 `Result<_, String>`，與現有錯誤回報慣例一致。

## Risks / Trade-offs

- **Mutex 排隊**：高頻 command 並發時需排隊等鎖。→ 桌面 app 場景下並發極少，且 SQLite 操作通常在 1~5ms 內完成，實際影響可忽略。
- **持有鎖期間 panic**：若 `_internal` 函式 panic，Mutex 毒化後所有後續 DB 操作失敗。→ 現有 `_internal` 函式使用 `?` 傳遞錯誤而非 panic，風險極低。
- **遷移範圍大**：30+ 個呼叫點需修改。→ 以模組為單位逐步替換，每步可單獨建置驗證。

## Migration Plan

1. `types.rs`：新增 `DbState` struct
2. `db.rs`：在 `init_db` 加入 `PRAGMA journal_mode=WAL`；`open_db_connection()` 加 `#[cfg(test)]` 或保留為 pub(crate)（限測試）
3. `lib.rs`（`run()`）：建立 `DbState` 並 `.manage()`
4. 逐模組替換：`commands/sessions.rs` → `commands/analytics.rs` → `sessions/copilot.rs` → `sessions/mod.rs` → `lib.rs` 中的 command
5. 移除 `DB_SCHEMA_INITIALIZED` AtomicBool
6. `cargo test` 驗證所有 50 個測試通過

**Rollback**：遷移以 git branch 進行，任何步驟失敗均可 revert 到上一個通過測試的 commit。

## Open Questions

- `open_db_connection()` 是否完全設為 `#[cfg(test)]`，或保留 `pub(crate)` 以備未來特殊用途？→ 建議保留 `pub(crate)` 但加文件說明「僅供測試/啟動使用」。
