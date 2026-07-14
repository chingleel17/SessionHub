## ADDED Requirements

### Requirement: Claude adapter 解析 limits 陣列中的 scoped-model 每週視窗

Claude quota adapter SHALL 在解析頂層時間視窗後，額外掃描 usage API 回應的 `limits` 陣列，為每個 `group == "weekly"` 且 `scope.model.display_name` 非空的項目產生一個 `QuotaWindow`：`window_key` 為 `seven_day_<display_name 小寫>`、`label` 為該 `display_name`、`utilization` 為 `percent / 100`、`resets_at` 為該項目自身的 `resets_at`。此解析 SHALL 為資料驅動，不硬編特定模型名稱；若解析出的 `window_key` 與已產生的頂層視窗重複，SHALL 略過以避免重複計入。

`QuotaWindow.window_key` 的合法值 SHALL 擴充為包含 `seven_day_fable` 及一般化的 `seven_day_<model>`（依 API 提供的 scoped model 動態產生）。

#### Scenario: 解析 Fable scoped 週視窗

- **WHEN** usage API 的 `limits` 含一項 `group: "weekly"`、`percent: 100`、`scope.model.display_name: "Fable"`、`resets_at: "2026-07-14T16:00:00Z"`
- **THEN** snapshot 的 `windows` 含一個 `window_key: "seven_day_fable"`、`label: "Fable"`、`utilization: 1.0`、`resets_at` 為該項目自身值的視窗

#### Scenario: 不同 scoped 模型自動產生視窗

- **WHEN** `limits` 含某個 `scope.model.display_name` 為 SessionHub 未預先對映的模型
- **THEN** 系統仍為其產生 `seven_day_<model 小寫>` 視窗
- **AND** 前端在無本地化對映時退回顯示該模型的 `display_name`

#### Scenario: scoped reset 時間獨立於週視窗

- **WHEN** 某 scoped 週視窗的 `resets_at` 與 `weekly_all`（seven_day）不同
- **THEN** 該 scoped 視窗顯示自己的 `resets_at`，不套用週視窗的 reset 時間

#### Scenario: 與頂層視窗去重

- **WHEN** 某 scoped model 解析出的 `window_key` 已由頂層視窗解析產生
- **THEN** 系統略過該 `limits` 項目，`windows` 中不出現重複 `window_key`

#### Scenario: 無 scoped 週視窗

- **WHEN** `limits` 缺席，或其中沒有任何 `scope.model.display_name` 非空的每週項目
- **THEN** `windows` 僅含既有頂層視窗，不新增 scoped 視窗
