## Why

SessionCard 展開的統計詳情面板(SessionStatsPanel)目前有兩層框限疊在卡片內(panel border + 左右欄各自的 border/background),違反 minimal-ui 去卡片原則、視覺雜亂;同時後端已為各 provider 算好 per-model 的 token 與成本資料(`modelMetrics`),前端卻只顯示逗號串接的模型名稱、且僅 copilot 顯示成本,使用者無法按模型檢視 token 用量。

## What Changes

- 移除 SessionStatsPanel 的雙層框限:panel 與左右欄的 border/background 改為透明區塊 + hairline 分隔,符合 sessionhub-minimal-ui 去卡片原則
- 摘要統計改為橫向 stat row(數值大字在上、label 小字在下),取代現行左欄逐列 label-value 版面
- 「使用模型」升級為 per-model 明細表:每模型一行,顯示輸入/輸出 tokens;有成本資料時同時顯示成本與合計(不再限定 copilot)
- 工具調用只保留一處分區:顯示 top N,其餘收合為「+N 更多」展開,取消固定高度捲動框
- 時長顯示格式化為 `Xh Ym`(與 SessionStatsBadge 一致)
- 相關 i18n label 調整(所有語系檔)

## Capabilities

### New Capabilities

(無)

### Modified Capabilities

- `session-stats-display`: SessionStatsPanel 的顯示需求變更——版面由雙欄卡片改為去卡片分區、新增 per-model token/成本明細表(all providers)、工具調用改 top N + 展開、時長格式化

## Impact

- 前端:`src/components/SessionStatsPanel.tsx`(重寫版面)、`src/App.css` 的 `.stats-panel*` 區塊、`src/locales/*.ts`(label 增修)
- 後端:不變動(`modelMetrics` 資料已存在於 `SessionStats`)
- 型別:不變動(`ModelMetricsEntry` 已含 inputTokens/outputTokens/requestsCost)
