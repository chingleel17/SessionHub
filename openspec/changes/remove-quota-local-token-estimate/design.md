## Context

Quota 面板目前混用兩種來源的數字：`windows` / `extra_credits` / `reset_credits` 來自各 provider 官方 API，`local_tokens` 則是 SessionHub 掃描本機 session 檔推估的當月 token 累計。兩者在同一張卡片內垂直排列，視覺上無從區分。

`local_tokens` 僅由 codex 與 opencode 兩個 adapter 填入，其餘 provider 一律為 `None`。Codex 的掃描函式 `count_monthly_tokens_from_jsonl` 假設每行 JSONL 頂層有 `usage` 物件，但現行 Codex rollout 檔的格式為：

```json
{"timestamp":"...","type":"event_msg","payload":{"type":"token_count",
 "info":{"total_token_usage":{...},"last_token_usage":{"input_tokens":12020,"output_tokens":62}}}}
```

token 位於 `payload.info.last_token_usage`，頂層無 `usage` 鍵，因此每行都被略過，結果恆為 `0k / 0k`。該函式另有以 `chrono::Utc` 判斷月份的問題（使用者為 UTC+8，月初 8 小時內的紀錄會被歸入上月）。

跨供應商的 token 統計已由 Analytics 提供（`commands/analytics.rs` 的 `get_analytics_data_internal`，支援 day / week / month 分組），功能上不需要 quota 面板重複承擔。

移除的關鍵約束：`source: "local_scan"` 與 `local_tokens` 是兩個獨立概念。前者是資料來源徽章，由 opencode 與 antigravity 使用（`antigravity.rs` 三處均為 `source: "local_scan"` 搭配 `local_tokens: None`），必須完整保留。

## Goals / Non-Goals

**Goals:**

- 從 Rust 與 TypeScript 兩端移除 `LocalTokenUsage` 型別與 `QuotaSnapshot.local_tokens` 欄位
- 移除 codex 與 opencode 的本機 token 掃描邏輯
- 為「provider 有 snapshot 但無任何額度內容」新增明確的 UI 呈現，避免 OpenCode 卡片變成空白區塊
- 舊 SQLite 快照維持可反序列化，不需 DB migration

**Non-Goals:**

- 不修復 Codex 的 JSONL 解析路徑（該邏輯整個移除，修復無意義）
- 不改動 Analytics 的統計邏輯，也不在本次於 quota 面板加入跨供應商總計
- 不移除 `source: "local_scan"` 標記與 `quota.monitoring.source.local_scan` 翻譯鍵
- 不改動 OpenCode 的 session 掃描（`sessions/opencode.rs`）——本次僅涉及 `quota/opencode.rs`

## Decisions

### 決策一：保留 OpenCode provider 卡片，改顯示「無額度資料」

OpenCode 的 snapshot 是 `windows: None` + `local_tokens: Some(...)`，`local_tokens` 是其唯一內容。直接移除會讓卡片只剩 provider 名稱與來源徽章，看起來像壞掉。

**選擇**：新增一個通用的「無額度資料」呈現狀態，條件為 `status === "ok"` 且無 `windows`（null 或空陣列）且無 `extraCredits`、`resetCredits`。三處 UI（QuotaOverview / TrayQuotaPanel / QuotaOverlay）共用同一判斷與同一翻譯鍵。

**替代方案（未採用）**：
- 從 quota 面板完全移除 OpenCode provider — 使用者已在設定中啟用該 provider，靜默消失比空卡片更令人困惑；且 OpenCode 未來若有額度 API 需要重新加回。
- 僅在 OpenCode 硬編此文字 — 違反通用性，antigravity 等 provider 未來落入同樣狀態時需重複處理。

判斷條件寫成通用述詞而非 `provider === "opencode"`，因為這是「snapshot 無可渲染內容」的普遍情形，不是 OpenCode 專屬。

### 決策二：serde 預設行為處理舊快照，不做 DB migration

`quota_snapshots` table 存的是整個 snapshot 的 JSON 序列化結果。移除 struct 欄位後，serde 反序列化遇到 JSON 中多餘的 `localTokens` 鍵預設忽略（未啟用 `deny_unknown_fields`），舊快照可正常載入且 `local_tokens` 相關資訊自然消失。

**替代方案（未採用）**：寫一次性 migration 清理既有 JSON — 沒有必要，欄位被忽略後下次 refresh 就會覆寫為新格式。

需在實作時確認 `QuotaSnapshot` 未標註 `#[serde(deny_unknown_fields)]`，否則舊快照載入會失敗。

### 決策三：QuotaOverlay 的 fallback 分支改寫

`QuotaOverlay.tsx:348-354` 現有結構是「有 windows 就渲染 bar 列，否則顯示 localTokens 摘要，localTokens 也沒有就顯示來源徽章文字」。移除 `localTokens` 後該三元式退化為只剩來源徽章文字（「本地估算」），語意上答非所問——使用者要看的是額度，不是資料來源。

**選擇**：該 fallback 分支直接改為「無額度資料」說明文字，與另兩處 UI 一致。

### 決策四：翻譯鍵命名

移除 `quota.monitoring.localUsage`，新增 `quota.monitoring.noQuotaData`（zh-TW：「無額度資料」；en-US：「No quota data」）。沿用既有 `quota.monitoring.*` 命名空間。

## Risks / Trade-offs

**[使用者感知功能消失]** → 該數值長期顯示 `0k / 0k`（Codex）或無法對應任何訂閱額度（OpenCode），移除反而消除誤導。跨供應商 token 統計在 Analytics 仍可查。

**[誤刪 `local_scan` 來源標記導致 antigravity / opencode 徽章消失]** → 實作時以 `local_tokens` / `localTokens` 為搜尋關鍵字，明確排除 `local_scan` 字串；`quota.monitoring.source.local_scan` 翻譯鍵不得移除。

**[編譯通過但測試 fixture 未更新]** → `tray_icon.rs:349` 有 `local_tokens: None` 的測試 snapshot、`codex.rs:593` 有 `"localTokens": null` 的預期 JSON。必須執行 `cargo test` 而非僅 `cargo build`。

**[前端型別檢查遺漏 preview mock]** → `.design-sync/previews/QuotaOverview.tsx:37,40` 設定了 `localTokens` mock，移除型別欄位後會觸發 tsc 錯誤，需一併更新。

**[移除掃描函式後殘留未使用的 import]** → `quota/codex.rs` 的 `chrono::Datelike`、`quota/opencode.rs` 的 `chrono::Datelike` 與 rusqlite 相關 import 可能變成未使用，產生編譯警告，需一併清理。

## Migration Plan

單一 commit 即可完成，無需分階段：

1. 後端移除型別與掃描邏輯 → `cargo test` 通過
2. 前端移除渲染與型別、加入無資料呈現 → typecheck / lint 通過
3. 更新 live specs（透過 OpenSpec archive 流程）

**回滾**：純刪除性變更，`git revert` 即可完整還原，無資料庫或設定檔的不可逆變更。

## Open Questions

無。OpenCode 空卡片的處理方式與 OpenSpec 流程皆已與使用者確認。
