# Design: Claude scoped-model 每週額度視窗

## Context

實測 Anthropic OAuth usage API（`GET https://api.anthropic.com/api/oauth/usage`）回應中，Fable 額度不在頂層 key（頂層 `seven_day_*` codename 皆為 null），而在 `limits` 陣列：

```json
"limits": [
  { "kind": "session",       "group": "session", "percent": 20,  "resets_at": "...18:10...", "scope": null },
  { "kind": "weekly_all",    "group": "weekly",  "percent": 93,  "resets_at": "...16:00...", "scope": null },
  { "kind": "weekly_scoped", "group": "weekly",  "percent": 100, "resets_at": "...16:00...",
    "scope": { "model": { "display_name": "Fable" } }, "is_active": true }
]
```

`claude.rs` 目前只解析頂層 `five_hour` / `seven_day` / `seven_day_sonnet` / `seven_day_opus`，因此 Fable 未被顯示。

## Goals / Non-Goals

**Goals:**

- 從 `limits[]` 取出 scoped-model 每週視窗（Fable 及未來任何帶 `scope.model.display_name` 的模型），加入 snapshot 的 `windows`。
- 各視窗讀自己的 `resets_at` 與 `percent`。

**Non-Goals:**

- 不重寫既有頂層視窗解析（維持 5h/7d/Sonnet/Opus 現行行為）。
- 不硬編 `"Fable"` 字串；資料驅動，對新模型自動適用。
- 不處理 `group == "session"` 或非 scoped 的 `limits` 項目（那些頂層已有對應視窗）。

## Decisions

### D1. Additive 解析，不重構

在既有 `window_defs` 迴圈之後追加一段：遍歷 `body["limits"]`，篩 `group == "weekly"` 且 `scope.model.display_name` 非空者，各建一個 `QuotaWindow`。理由：既有頂層解析已驗證可用，重寫會無謂風險化 5h/7d 顯示；additive 同樣能自動涵蓋新 scoped 模型。

### D2. window_key 與 label 命名

`window_key = "seven_day_" + display_name.to_lowercase()`（如 `seven_day_fable`），與既有 `seven_day_sonnet` / `seven_day_opus` 一致，前端可走既有本地化路徑。`label` 用 API 的 `display_name`（如 `"Fable"`）作為 fallback 顯示字串。前端 `WINDOW_KEY_MAP` 新增 `seven_day_fable → quota.window.sevenDayFable`；未知模型 key 未對映時 `localizedWindowLabel` 自動退回 raw label。

### D3. resets_at 與 utilization

- `utilization = percent / 100.0`（API `percent` 為 0–100，實測值 20/93/100）。
- `resets_at` 直接沿用該項目的 ISO 字串（與頂層視窗同格式），**不**假設等於 `weekly_all` 的 reset——今日相同僅為巧合。

### D4. 去重保護

若某 scoped model 解析出的 `window_key` 已存在於先前頂層迴圈推入的 windows，則略過，避免重複計入。今日不會觸發（Fable 頂層為 null），為防禦性保險。

## Risks / Trade-offs

- [API 結構變動] `limits[]` 為較新的結構化欄位，欄位名或巢狀可能改變 → 解析採寬鬆存取（缺欄位即略過該項），不影響其他視窗；以實際回應為 fixture 的單元測試守住回歸。
- [模型名含空白或特殊字元] `display_name` 轉小寫作 key 可能產生非預期 key（如含空白）→ 可接受：key 僅供本地化對映與去重，未對映時退回 raw label 仍可顯示。
