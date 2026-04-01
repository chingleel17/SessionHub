## Context

SessionHub 的後端掃描採用「每次全掃」策略：無論是 FS watcher 觸發或前端 `get_sessions` 呼叫，都會讀取所有 Copilot session 目錄的 `workspace.yaml` 並對 OpenCode SQLite 執行無限制 `SELECT`。兩個資料來源共用同一個 `sessions-updated` Tauri event，任意一方的變動都會讓前端 invalidate 整個 query 並觸發完整後端重掃。

目前規模下延遲可接受，但 session 數量成長後（100+ sessions）每次掃描的磁碟 I/O 與 CPU 開銷都會線性增加。此外 Copilot CLI 在工作時會產生連續寫入爆發，每次寫入都各自觸發完整掃描，造成短時間內的大量重複工作。

**現有關鍵限制：**

- `lib.rs` 是單一大檔（1630+ 行），無模組邊界，狀態靠 `tauri::State<Mutex<T>>` 管理。
- 無 tokio async runtime（Tauri 2 內建 tokio，但現有 commands 全為同步 fn）。
- 所有 IPC 只能在 `App.tsx`，子元件不得呼叫 `invoke()`。
- Windows only：路徑操作用 `PathBuf`，不 hardcode `\`。

## Goals / Non-Goals

**Goals:**

- Copilot watcher 觸發後 800ms 防抖再執行掃描，OpenCode watcher 500ms 防抖。
- 拆分事件：`copilot-sessions-updated` / `opencode-sessions-updated`。
- runtime 記憶體快取（`ScanCache`）儲存上次全掃結果，watcher 觸發後只重新讀取有變化的部分。
- Copilot 增量邏輯：比對目錄 mtime，只重新解析 `workspace.yaml` 有更新的 session。
- OpenCode 增量邏輯：`SELECT ... WHERE time_updated > last_cursor`，只查詢新行。
- 超過 30 分鐘未全掃時，下一次觸發自動執行全掃並重置快取。
- `get_sessions` 加入 `force_full: Option<bool>` 參數供封存/刪除等操作強制全掃。

**Non-Goals:**

- 快取持久化至磁碟或 SQLite（重啟後重新全掃一次即可）。
- 跨 session 的差異偵測（只偵測「是否有更動」，不 diff 內容）。
- 前端 UI 變更（使用者可見行為不變）。
- macOS / Linux 支援（架構不排除，但此 change 不驗證）。
- 後台排程全掃（idle 觸發）—— 留待後續 change，此次只做閾值觸發。

## Decisions

### D1：快取位置 — AppState (Tauri managed state)，非 SQLite

**決定**：在 `AppState` 加入 `ScanCache`，以 `Mutex<Option<ProviderCache>>` 儲存兩個 provider 各自的快取。

**理由**：快取是 runtime 狀態，不需持久化；AppState 已是 Tauri 狀態管理慣例；SQLite 寫入會增加 I/O 開銷，違背此 change 的目標。

**替代方案**：全域靜態變數 — 拒絕，因為難測試且違反現有 DI 慣例。

---

### D2：Debounce 實作 — Arc<Mutex<Instant>> + spawn thread

**決定**：每個 watcher callback 中更新一個 `Arc<Mutex<Instant>>` 的「最後事件時間」，同時 spawn 一個 thread 等待 debounce 時間後比對 Instant，若穩定才執行掃描並發送 event。

```rust
// 每個 watcher 持有：
last_event: Arc<Mutex<Instant>>

// callback 中：
*last_event.lock().unwrap() = Instant::now();
let le = Arc::clone(&last_event);
let window = window.clone();
thread::spawn(move || {
    thread::sleep(Duration::from_millis(DEBOUNCE_MS));
    let elapsed = le.lock().unwrap().elapsed();
    if elapsed >= Duration::from_millis(DEBOUNCE_MS) {
        // 執行掃描 + emit event
    }
});
```

**理由**：不引入新相依（無需 `debounce` crate）；與現有同步 command 風格一致；邏輯透明可測試。

**替代方案**：tokio `sleep` + async task — 可行，但改動面大（需將 command fn 改為 async），風險高。

---

### D3：快取結構

```rust
struct ProviderCache {
    sessions: Vec<SessionInfo>,
    // Copilot: session_id → 目錄最後修改時間 (Unix timestamp secs)
    // OpenCode: 不使用此欄位，用 last_cursor
    session_mtimes: HashMap<String, i64>,
    last_full_scan_at: Instant,
    // OpenCode only: 上次全掃時見到的最大 time_updated
    last_cursor: i64,
}

struct ScanCache {
    copilot: Mutex<Option<ProviderCache>>,
    opencode: Mutex<Option<ProviderCache>>,
}
```

**理由**：兩個 provider 各自獨立，避免 Copilot 掃描影響 OpenCode 快取失效；`Option<ProviderCache>` 表示「尚未進行過全掃」，此時回退到全掃路徑。

---

### D4：增量掃描觸發條件

全掃條件（任一滿足即全掃）：

1. `ProviderCache` 為 `None`（首次或重啟後）
2. `last_full_scan_at.elapsed() > 30 minutes`
3. `force_full == Some(true)`（封存/刪除操作傳入）

增量條件：上述都不滿足，使用快取差異掃描。

---

### D5：Copilot 增量邏輯

```
1. read_dir(session_state_dir) → [(session_id, dir_mtime)]
2. 比對 session_mtimes cache：
   - mtime 變化 or 新增 → 重新讀取 workspace.yaml
   - mtime 未變 → 直接取 cache 中的 SessionInfo
   - cache 有但 dir 消失 → 從結果移除
3. 更新 cache.session_mtimes + cache.sessions
```

---

### D6：OpenCode 增量邏輯

```sql
SELECT * FROM sessions WHERE time_updated > {last_cursor}
ORDER BY time_updated ASC
```

將結果 merge 進 `cache.sessions`（upsert by session_id），更新 `last_cursor` 為新結果中最大 `time_updated`。若查詢結果為空，快取不變。

---

### D7：事件命名與前端處理

| 舊事件 | 新事件 | watcher |
|--------|--------|---------|
| `sessions-updated` | `copilot-sessions-updated` | Copilot FS watcher |
| （無） | `opencode-sessions-updated` | OpenCode WAL watcher |

前端 `App.tsx` 兩個 listener 都呼叫 `queryClient.invalidateQueries({ queryKey: ["sessions"] })`，後端 `get_sessions` 負責合併兩個 provider 快取後回傳統一的 `Vec<SessionInfo>`。前端不感知 provider 分離。

## Risks / Trade-offs

- **[Race condition] Copilot watcher 在增量掃描進行中又觸發** → 因為每次觸發都重新計時，debounce 期間的額外事件只會延後而不會並發執行；Mutex 保護快取讀寫，不會出現 dirty read。

- **[Stale cache] 30 分鐘閾值造成短暫不一致** → 可接受：FS watcher 的 incremental path 會即時反映目錄變化；閾值全掃只是防止累積漂移的保底機制。若使用者做封存/刪除操作，`force_full=true` 確保立即全掃。

- **[Memory] session 數量極大時 `Vec<SessionInfo>` 記憶體佔用** → 現有架構已在每次全掃時建立整個 Vec，此 change 只是保留一份，無額外記憶體開銷。

- **[Thread 爆炸] 頻繁 FS 事件 spawn 大量 thread** → debounce thread 極短暫（sleep 後立即結束），不持有重資源；OS thread pool 可承受；若有顧慮可改用 channel-based debounce（後續優化）。

- **[OpenCode cursor 回滾] SQLite WAL checkpoint 造成 time_updated 亂序** → OpenCode 的 `time_updated` 是 INSERT/UPDATE 時的 Unix timestamp，checkpoint 不影響欄位值；cursor 只前進不後退，不受 checkpoint 影響。

## Migration Plan

1. 部署新版本後，app 首次啟動執行一次全掃並填充快取（`ProviderCache` 從 None → Some）。
2. 舊 `sessions-updated` event 移除，前端改為監聽兩個新事件。
3. 無資料庫 schema 變更，無需 migration script。
4. **回滾**：回退至上一版本即可，快取為記憶體狀態，無持久化資料需要清理。

## Open Questions

- 無（設計決策已與使用者確認）。
