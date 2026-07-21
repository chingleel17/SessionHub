## ADDED Requirements

### Requirement: 提供時序聚合統計查詢

系統 SHALL 提供 `get_analytics_data` Tauri command，接受時間範圍、分組粒度（day/week/month）與可選的專案路徑（cwd）參數，從 `session_stats` SQLite 快取聚合統計資料並回傳時序陣列。

#### Scenario: 依日分組查詢單一專案

- **WHEN** 前端以 `{ cwd: "/path/to/project", startDate: "2025-04-01", endDate: "2025-04-30", groupBy: "day" }` 呼叫 `get_analytics_data`
- **THEN** 系統回傳 `Vec<AnalyticsDataPoint>`，每個元素代表該日的統計加總
- **AND** `label` 格式為 `"YYYY-MM-DD"`
- **AND** 只包含 `cwd` 符合指定專案路徑的 session

#### Scenario: 依周分組查詢所有專案

- **WHEN** 前端以 `{ cwd: null, startDate: "2025-01-01", endDate: "2025-04-30", groupBy: "week" }` 呼叫 `get_analytics_data`
- **THEN** 系統回傳每周一筆的聚合資料
- **AND** `label` 格式為 `"YYYY-WNN"`
- **AND** 聚合涵蓋所有專案的 session

#### Scenario: 依月分組查詢

- **WHEN** `groupBy` 為 `"month"`
- **THEN** 每個 `AnalyticsDataPoint.label` 格式為 `"YYYY-MM"`

#### Scenario: 區間內無資料

- **WHEN** 指定時間範圍內 `session_stats` 快取無符合 session
- **THEN** 系統回傳空陣列 `[]`，不回傳錯誤

#### Scenario: session_stats 快取未完整覆蓋

- **WHEN** 部分 session 尚未計算統計（快取不存在）
- **THEN** 系統只回傳已有快取的 session 聚合結果
- **AND** 回傳的 `AnalyticsDataPoint` 中 `missingCount` 欄位標示該分組未快取的 session 數量

### Requirement: AnalyticsDataPoint 型別定義

`AnalyticsDataPoint` Rust struct SHALL 包含以下欄位，並以 `#[serde(rename_all = "camelCase")]` 序列化。

#### Scenario: 前端收到完整欄位

- **WHEN** 前端收到 `get_analytics_data` 回傳值
- **THEN** 每個元素 SHALL 包含：`label: string`、`outputTokens: number`、`inputTokens: number`、`interactionCount: number`、`costPoints: number`、`sessionCount: number`、`missingCount: number`

### Requirement: 查詢參數驗證

系統 SHALL 驗證 `get_analytics_data` 的輸入參數。

#### Scenario: 無效日期格式

- **WHEN** `startDate` 或 `endDate` 不符合 `YYYY-MM-DD` 格式
- **THEN** 回傳 `Err("invalid date format")` 錯誤字串

#### Scenario: startDate 晚於 endDate

- **WHEN** `startDate` > `endDate`
- **THEN** 回傳 `Err("startDate must be before endDate")` 錯誤字串

#### Scenario: 無效 groupBy 值

- **WHEN** `groupBy` 不在 `["day", "week", "month"]` 中
- **THEN** 回傳 `Err("invalid groupBy value")` 錯誤字串
