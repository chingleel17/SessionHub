## Context

SessionHub 目前的 OpenCode session 掃描器是建立在舊版 OpenCode JSON storage 形狀上：`storage/project/*.json`、`storage/session/<projectId>/*.json` 與 `storage/message/<sessionId>/*.json`。但實際檢查最新的 OpenCode 安裝後，新的 session 已寫入 `opencode.db` 的 `session`、`project`、`message` 等資料表，bridge event 也能持續通知 SessionHub。

這導致目前系統出現「bridge 有通知，但列表沒更新」的錯覺。實際上 refresh 有發生，只是 refresh 後仍去掃舊 JSON 路徑，因此掃不到最新 session。你本機的 `metadata.db` 也驗證了這一點：OpenCode cache 停在很舊的時間，而 `opencode.db` 裡已經有新的 session row。

這次不是純 UI 問題，而是 OpenCode provider 的主資料來源已經改版。因此修正不只涉及 `sessions/opencode.rs`，還會連動到 fallback watcher、stats 來源與測試模型。

## Goals / Non-Goals

**Goals:**

- 將 SessionHub 的 OpenCode session 掃描主來源改為 `opencode.db`。
- 讓 bridge refresh 後，DB-only 的最新 OpenCode session 能正確出現在 SessionHub 列表。
- 更新 OpenCode fallback watcher，改監看 `opencode.db` / WAL 變化。
- 檢查 OpenCode stats / message 解析是否仍依賴過時 JSON 路徑，至少修正已知會導致最新 session 缺資料的部分。
- 保留必要的舊 JSON 相容性，避免舊版 OpenCode 使用者完全失效。

**Non-Goals:**

- 本次不全面重寫所有 OpenCode stats 與 analytics 邏輯到新 DB schema，除非它直接影響 session 列表或基礎 stats。
- 本次不改動 provider bridge 事件格式。
- 本次不導入新的外部套件；優先使用既有 `rusqlite` 讀取 OpenCode DB。

## Decisions

### 1. OpenCode session 掃描改為 DB 優先、JSON fallback

SessionHub 讀取 OpenCode session 時，優先查詢 `opencode.db` 的 `session` 與 `project` 表；若 DB 不存在、表結構不可用，才退回舊 JSON storage 掃描。

原因：這能解決最新版 OpenCode session 不落在 JSON storage 的問題，同時保留對舊資料布局的最低相容性。

替代方案：

- 直接完全移除 JSON 掃描。缺點是舊版 OpenCode 或異常環境可能失效。
- 維持 JSON 為主、DB 為輔。缺點是會繼續漏掉 DB-only session。

### 2. fallback watcher 改監看 `opencode.db` 與 `opencode.db-wal`

OpenCode fallback watcher 不再以 `storage/session` / `storage/message` 目錄作為主判斷，而改以資料庫檔與 WAL 檔變化為主。

原因：既然 session 主資料已進入 DB，fallback watcher 也必須與資料來源一致，否則即使 bridge 不可用也無法可靠刷新。

替代方案：

- 繼續監看舊目錄。缺點是新 session 可能完全不觸發有效 refresh。

### 3. OpenCode stats / message 解析至少要支援新 session 的基本資料鏈

對於最新 session，只要 `session_dir` 或對應 message source 已從 DB 可解析，就應確保後續 stats 邏輯至少不會因為找不到舊路徑而直接失效。

原因：session 列表修好後，如果點進去 stats 全部失敗，使用者仍會覺得 OpenCode 支援不穩。

替代方案：

- 這次只修列表。缺點是功能體驗會斷裂。

### 4. 測試要覆蓋「bridge 有通知 + JSON 沒有檔案 + DB 有 row」的情境

新的測試案例應明確驗證：即使舊 JSON session 檔不存在，只要 `opencode.db` 有 session row，SessionHub 就能在 refresh 後列出該 session。

原因：這正是本次 bug 的核心重現條件。

## Risks / Trade-offs

- [OpenCode DB schema 後續仍可能演進] -> 將 DB 查詢集中在單一模組，減少日後修正面積。
- [同時支援 DB 與 JSON 會增加複雜度] -> 以 DB 優先、JSON fallback 的單向策略降低分支數。
- [stats 邏輯可能仍有部分依賴舊 message/part 目錄] -> 先補列表與已知關聯路徑，將剩餘 stats 差異留待後續追蹤。
- [WAL / DB watcher 在 Windows 上事件較頻繁] -> 沿用既有 debounce 與 cheap verify。

## Migration Plan

1. 先在 `sessions/opencode.rs` 加入 DB 讀取路徑。
2. 保留舊 JSON 掃描作 fallback，避免一次切斷相容性。
3. 調整 OpenCode watcher 與 snapshot 驗證來源。
4. 補 DB-only session 測試與快取驗證。
5. 若 stats 邏輯仍有缺口，再補最小相容修正。

## Open Questions

- 最新 OpenCode session message / part 是否仍完整落地於 `storage/message` / `storage/part`，或未來也會轉進 DB？
- OpenCode DB schema 是否有穩定欄位可直接取代目前部分 JSON-based stats 推導？
