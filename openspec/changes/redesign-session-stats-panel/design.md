## Context

SessionCard 展開統計時渲染 `SessionStatsPanel`(`src/components/SessionStatsPanel.tsx`),現行版面為兩欄 grid,`.stats-panel` 與 `.stats-panel-col` 各帶一層 border + 背景,疊在 SessionCard 本身的卡片邊界內形成三層框限。左欄為逐列 label-value 摘要,右欄為模型名稱純文字、工具調用捲動清單、以及僅限 copilot 的模型成本表。

資料面:`SessionStats.modelMetrics: Record<string, ModelMetricsEntry>` 已由後端對 copilot 與 claude 填入 per-model 的 `inputTokens` / `outputTokens` / `requestsCost`(見 `src-tauri/src/stats.rs`),前端未使用 token 部分。opencode / codex / antigravity 目前 `modelMetrics` 為空物件,僅有 `modelsUsed` 名單。

視覺規範:sessionhub-minimal-ui skill——去卡片、hairline 分界、設計 token、雙主題。

## Goals / Non-Goals

**Goals:**

- 去除 panel 內兩層框限,視覺融入 SessionCard,符合去卡片原則
- 呈現 per-model 的 token(輸入/輸出)與成本明細,不限 provider
- 摘要區改橫向 stat row,提高資訊密度
- 工具調用單一分區、top N + 展開
- 時長格式化 `Xh Ym`

**Non-Goals:**

- 不改後端解析或 `SessionStats` / `ModelMetricsEntry` 型別
- 不為 opencode / codex / antigravity 補 per-model 資料(modelMetrics 為空時退回顯示 `modelsUsed` 名單)
- 不動 SessionStatsBadge 與 ProjectStatsBanner / Analytics 相關元件

## Decisions

1. **版面:垂直三分區(摘要 / 模型 / 工具),分區間 hairline**,取代左右兩欄。
   - 理由:兩欄在窄卡片下已需 media query 摺疊;垂直分區天然自適應,且與去卡片原則的「標題 + hairline + 內容」範式一致。
   - 替代方案:保留兩欄僅去框——資訊分組仍割裂(總數在左、明細在右),不採用。
2. **摘要 stat row:數值(`font-weight: 600`,較大字)在上、label(`--color-text-secondary`,小字)在下,`flex` 橫排 + wrap**。
   - 理由:對標 Vercel/Linear 的 metric 呈現;比逐列 label-value 節省一半以上垂直空間。
3. **模型分區:有 `modelMetrics` 時渲染表格(模型 / 輸入 / 輸出 / 成本),多於一個模型時附合計列;`modelMetrics` 為空時退回逗號串 `modelsUsed`**。
   - 成本欄:該模型 `requestsCost > 0` 才顯示數值,全部為 0 時整欄隱藏(claude 可能無 pricing 資料)。
   - 理由:資料已存在,單一表格同時解決「token 沒有 by 模型」與「成本僅 copilot」兩個問題;移除 `provider === "copilot"` 分支。
4. **工具調用:預設顯示呼叫次數前 5 名,其餘以「+N」文字按鈕展開/收合(local `useState`),移除 `max-height` 捲動框**。
   - 理由:卡片內出現內嵌 scrollbar 是視覺噪音;top 5 覆蓋絕大多數場景。
5. **時長格式化:沿用 SessionStatsBadge 既有的換算規則(`<60` → `Xm`;`>=60` → `XhYm`/`Xh`),抽成共用 util 或複用現有函式**,避免兩處實作分歧。
6. **CSS:重寫 `.stats-panel*` 區塊**——`.stats-panel` 保留上方 hairline(`border-top: 1px solid var(--color-border-subtle)`)與留白以區隔卡片主體,其餘 border/background/radius 移除;所有顏色一律 token,雙主題實測。

## Risks / Trade-offs

- [top 5 截斷讓少用工具不可見] → 「+N」展開即可看到全部;預設狀態資訊密度優先。
- [claude 的成本為估算值,與 copilot 計費點數語意不同] → 沿用現有 `stats.detail.modelCost` label,不在本次引入單位換算;若使用者反映混淆再另開 change。
- [垂直三分區比兩欄更長] → 摘要改橫向 stat row 與工具 top 5 截斷抵銷高度增加;實測展開高度不高於現狀。
- [深色主題露白] → 只用 token、不硬編顏色,完成後 light/dark 雙主題實測。
