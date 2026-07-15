# Proposal: 新增 Claude 依模型範圍的每週額度視窗（Fable 等）

## Why

Anthropic OAuth usage API 除了既有的 5 小時、每週（seven_day）等視窗外，還在回應的 `limits` 陣列中提供**依模型範圍（scoped）的每週額度**，其中包含獨立的 Fable 額度。SessionHub 目前只解析頂層固定 key（`five_hour` / `seven_day` / `seven_day_sonnet` / `seven_day_opus`），這些 scoped 額度（實測 Fable 只出現在 `limits` 陣列、頂層對應 key 為 null）完全沒被顯示，使用者無法在 SessionHub 看到 Fable 的用量與重置時間。

## What Changes

- Claude quota adapter 在既有視窗解析之後，額外掃描回應的 `limits[]`，取出 `scope.model.display_name` 非空的每週 scoped 項目，各產生一個 `QuotaWindow`：
  - `utilization` 來自該項目的 `percent`（0–100 → 0.0–1.0）
  - `resets_at` 讀該項目自己的 `resets_at`（不假設等於週視窗；目前巧合相同，但以 API 為準）
  - `window_key` 為 `seven_day_<model_lower>`（如 `seven_day_fable`），與既有 `seven_day_sonnet` / `seven_day_opus` 命名一致
- 前端視窗標籤本地化新增 Fable 對映（`quota.window.sevenDayFable`）；未知模型退回顯示 API 的 `display_name`。
- 對既有頂層視窗已產生的相同 `window_key` 做去重保護，避免重複計入。

## Capabilities

### New Capabilities

（無新 capability；此為既有 quota 能力的擴充）

### Modified Capabilities

- `provider-quota-monitoring`: Claude adapter 資料來源規格新增「解析 `limits[]` 中的 scoped-model 每週視窗」；`QuotaWindow.window_key` 列舉新增 `seven_day_fable`（及一般化的 `seven_day_<model>`）。

## Impact

- **Rust 後端**：`src-tauri/src/quota/claude.rs` — 於 `fetch_snapshot()` 視窗解析後追加 `limits[]` scoped 視窗解析；新增對應單元測試（以實際 API 回應為 fixture）。
- **前端**：
  - `src/utils/quotaWindowLabel.ts` — `WINDOW_KEY_MAP` 新增 `seven_day_fable`。
  - `src/locales/zh-TW.ts` / `en-US.ts` — 新增 `quota.window.sevenDayFable` 文案。
- **相依性**：無新增套件；沿用既有 `parse_window` 慣例與 `QuotaWindow` 結構。
- **相容性**：純新增視窗，不改動既有 5h/7d/Sonnet/Opus 視窗行為；無 API/DB migration。
