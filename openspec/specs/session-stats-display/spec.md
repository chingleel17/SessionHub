## Purpose

定義 Session 統計摘要、詳細統計面板、模型明細與工具調用清單的顯示行為，確保不同來源的 session 都能以一致且可讀的方式呈現統計資料。

## Requirements

### Requirement: Session 對話時長自動換算單位

系統 SHALL 在 SessionStatsBadge 顯示對話時長時，自動換算為適合閱讀的單位：不足 60 分鐘顯示為分鐘（`Xm`），達 60 分鐘以上換算為小時加分鐘（`XhXm` 或 `Xh`）。

#### Scenario: 時長不足 60 分鐘

- **WHEN** session 的 `durationMinutes` 小於 60
- **THEN** 顯示為 `{durationMinutes}m`（例如 `45m`）

#### Scenario: 時長超過 60 分鐘

- **WHEN** session 的 `durationMinutes` 超過 60
- **THEN** 顯示為 `XhYm` 格式，若可整除則省略分鐘（例如 `2h`）

### Requirement: Sidebar 收合時顯示版本號

系統 SHALL 在 sidebar 收合狀態下於 footer 顯示縮短版本號，不得完全隱藏。

#### Scenario: Sidebar 收合時

- **WHEN** sidebar 處於收合狀態
- **THEN** footer 顯示縮短版本號（例如 `v0.1`），hover 時 tooltip 顯示完整版本號

### Requirement: SessionStatsBadge 顯示欄位

SessionStatsBadge SHALL 以緊湊 badge 格式顯示 session 統計摘要。

#### Scenario: Badge 欄位

- **WHEN** session stats 已載入
- **THEN** badge 顯示：互動次數、output tokens（K 格式）、時長（Xm / XhYm）
- **AND** 若 isLive 為 true，顯示「LIVE」標記

### Requirement: SessionStatsPanel 顯示完整統計

SessionStatsPanel SHALL 以去卡片版面顯示 session 的完整詳細統計:panel 本身與內部分區不得有 border/background 框限(僅允許與卡片主體之間的 hairline 分隔),分區之間以 hairline(`1px var(--color-border-subtle)`)分界,由上而下依序為摘要 stat row、模型明細、工具調用。

#### Scenario: 摘要 stat row

- **WHEN** 使用者開啟 stats 詳情 panel
- **THEN** panel 頂部以橫向 stat row 顯示:output tokens、input tokens(大於 0 時)、互動次數、工具呼叫次數、reasoning count(大於 0 時)、時長
- **AND** 每個 stat 為數值在上(較大字重)、label 在下(次要文字色)的直式排列,整排橫向排開並可換行

#### Scenario: inputTokens 為 0 時不顯示

- **WHEN** session 的 `inputTokens` 等於 0
- **THEN** SessionStatsPanel 不顯示 inputTokens 欄位

#### Scenario: 時長格式化

- **WHEN** panel 顯示時長
- **THEN** 依 SessionStatsBadge 相同規則換算:不足 60 分鐘顯示 `Xm`,達 60 分鐘以上顯示 `XhYm`(可整除時 `Xh`)

#### Scenario: Live session 說明

- **WHEN** session 的 `isLive` 為 true
- **THEN** panel 顯示「Session 進行中」提示，統計標示為當前快照

### Requirement: SessionStatsPanel per-model 明細表

SessionStatsPanel SHALL 在 `modelMetrics` 非空時,於模型分區以表格顯示每個模型一行的明細:模型名稱、輸入 tokens、輸出 tokens,以及成本(任一模型 `requestsCost > 0` 時才顯示成本欄);此顯示不限定 provider。

#### Scenario: modelMetrics 有資料

- **WHEN** session 的 `modelMetrics` 含至少一個模型
- **THEN** 模型分區以表格顯示各模型的輸入/輸出 tokens(K/M 縮寫格式)
- **AND** 表格依 `requestsCost` 降冪排序

#### Scenario: 多模型時顯示合計

- **WHEN** `modelMetrics` 含兩個以上模型
- **THEN** 表格底部顯示合計列(輸入/輸出 tokens 與成本加總)

#### Scenario: 成本欄顯示條件

- **WHEN** 所有模型的 `requestsCost` 皆為 0
- **THEN** 成本欄與合計成本不顯示

#### Scenario: modelMetrics 為空時退回名單

- **WHEN** session 的 `modelMetrics` 為空物件
- **THEN** 模型分區顯示 `modelsUsed` 逗號串接名單;`modelsUsed` 也為空時顯示無資料文案

### Requirement: SessionStatsPanel 工具調用 top N 顯示

SessionStatsPanel SHALL 在工具調用分區預設僅顯示呼叫次數前 5 名的工具,其餘工具收合;分區不得使用固定高度捲動框。

#### Scenario: 工具超過 5 個

- **WHEN** `toolBreakdown` 含超過 5 個工具
- **THEN** 預設顯示前 5 名(依呼叫次數降冪),並提供「+N」展開控制
- **AND** 點擊展開後顯示全部工具,可再收合

#### Scenario: 工具不超過 5 個

- **WHEN** `toolBreakdown` 含 5 個以下工具
- **THEN** 全部顯示,不出現展開控制

#### Scenario: 無工具調用

- **WHEN** `toolBreakdown` 為空
- **THEN** 工具調用分區顯示無資料文案

### Requirement: OpenCode session 統計資料完整性

系統 SHALL 確保 OpenCode session 的統計資料從 JSON storage 正確讀取，顯示欄位與 Copilot session 一致。

#### Scenario: OpenCode session 統計顯示

- **WHEN** 使用者查看 OpenCode session 的統計
- **THEN** 系統顯示非零的 outputTokens（從 JSON storage 讀取）
- **AND** 顯示正確的 interactionCount、toolCallCount、modelsUsed、durationMinutes

#### Scenario: OpenCode session 無訊息資料時

- **WHEN** OpenCode session 的 message 目錄為空或不存在
- **THEN** 統計顯示全零值，不顯示錯誤訊息
