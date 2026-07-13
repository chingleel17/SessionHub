## Context

`src-tauri/src/quota/codex.rs` 的 `fetch_snapshot` 目前這樣建構窗口：

```rust
if let Some(primary) = rate_limit.get("primary_window") {
    if let Some(window) = parse_window("primary", "5h", primary) { windows.push(window); }
}
if let Some(secondary) = rate_limit.get("secondary_window") {
    if let Some(window) = parse_window("secondary", "7d", secondary) { windows.push(window); }
}
```

標籤（5h / 7d）與 `window_key`（primary / secondary）是**寫死綁定 API 欄位名**的。而 `parse_window` 只讀 `used_percent` 與 `reset_at`，忽略了窗口時長資訊。

實測 usage API（free plan）回應顯示，每個 window 物件都帶有精確時長欄位：

```json
"primary_window": {
  "used_percent": 5,
  "limit_window_seconds": 2592000,   // 30 天
  "reset_after_seconds": 2592000,
  "reset_at": 1786547913
},
"secondary_window": null
```

官方暫時移除 5h 限制後，唯一的長期限制被放進 `primary_window`，於是被此邏輯錯標為「5h」，7d 窗口則空白——即使用者回報的症狀。

## Goals / Non-Goals

**Goals:**
- 窗口類型與標籤依 window 物件的真實時長（`limit_window_seconds`）決定，與 API 欄位名解耦。
- 官方只回單一窗口時，依真實時長正確顯示（含非 5h/7d 的時長，如 30d）。
- `secondary_window` 為 null / 缺席時不產生假窗口。
- 保留對缺 `limit_window_seconds` 舊回應的相容退路。

**Non-Goals:**
- 不改動前端顯示邏輯（`quotaWindowLabel.ts` 對未知 `window_key` 已 fallback 到 `label` 字串）。
- 不改動 `local_tokens` 本月掃描邏輯。
- 不改動其他 provider 的 adapter。
- 不處理 `additional_rate_limits` / `code_review_rate_limit`（目前 API 回傳 null，超出本次範圍）。

## Decisions

### 決策 1：以 `limit_window_seconds` 分類，而非欄位名或 reset 時間反推

`parse_window` 增加讀取 `limit_window_seconds`（缺席時退回 `reset_after_seconds`，再退回 `reset_at - now`）。依時長映射：

| 時長（秒） | window_key | label |
|---|---|---|
| ≈ 18000（±容差，5h） | `five_hour` | `5h` |
| ≈ 604800（±容差，7d） | `seven_day` | `7d` |
| 其他 | `dynamic`（如 `30d`） | 由時長格式化（`Nh` / `Nd`） |

用「約略比對＋容差」而非精確等值，避免 API 微幅調整秒數就 fallback。容差取窗口的合理帶寬（例如 5h 命中區間 [1h, 24h)、7d 命中區間 [2d, 14d)，其餘落入 dynamic）。

**為何不用欄位名（現況）**：欄位名 primary/secondary 只表示「第一/第二組限制」，與時間語意無關；官方調整限制數量時就會錯位。

**為何不用純 `reset_at - now` 反推**：`reset_at` 反映的是「距下次 reset 的剩餘時間」，一個 7d 窗口在快 reset 時剩餘可能只有幾小時，會被誤判成 5h。`limit_window_seconds` 是窗口本身的固定長度，無此邊界問題——這是實測欄位存在後的首選。reset 反推僅作為欄位缺席時的退路。

### 決策 2：非 5h/7d 時長動態產生標籤

`label` 直接放格式化字串（如 `30d`、`3h`）。前端 `localizedWindowLabel` 對映不到的 `window_key` 會 fallback 回傳 `label`，因此動態標籤能正確顯示，無需改前端或新增 i18n key。`window_key` 對 dynamic 窗口可用穩定字串（如 `codex_window`）避免與既有 key 衝突。

### 決策 3：分類邏輯抽成純函式，補單元測試

將「秒數 → (window_key, label)」抽成獨立函式（如 `classify_window(seconds) -> (String, String)`），對 300 秒週邊、604800 秒週邊、2592000 秒、缺欄位退路等寫 `#[cfg(test)]` 測試，鎖住行為。

## Risks / Trade-offs

- **[容差區間選得太寬或太窄]** → 以「數量級」區分而非精確值：5h 級別（小時）與 7d 級別（天）差兩個數量級，容差有很大安全邊界；30d 落入 dynamic 也合理。
- **[未來官方恢復 5h 且同時有 7d]** → 兩窗口各依自身 `limit_window_seconds` 分類，primary=5h、secondary=7d 會自然正確，無需再改。
- **[label 動態字串未走 i18n]** → 5h/7d 仍走既有 i18n key；僅罕見的非標準時長用原始字串，可接受。

## Migration Plan

單檔改動（`codex.rs`）+ spec 更新，無資料遷移。舊快照在下次 refresh 後自動被正確窗口覆蓋。回滾僅需還原該檔。
