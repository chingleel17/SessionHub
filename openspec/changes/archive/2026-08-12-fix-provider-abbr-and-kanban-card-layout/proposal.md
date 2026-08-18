## Why

Provider 縮寫代碼原本在 `ProviderIcon.tsx`、`DashboardView.tsx`、`QuotaOverlay.tsx`、`StatusBar.tsx` 四處各自定義，彼此不一致且已出現錯誤（Claude Code 在部分位置誤植為 `CL`，在 Dashboard 專案標籤的 fallback 分支誤顯示為 `CP`）。此外，Dashboard Kanban 的 ProjectCard 標題列在分支名稱過長、或同專案使用多個工具（provider 標籤數量多）時，會把「專案名稱」擠壓到不可見，使用者無法辨識卡片對應的專案。這兩項已修正並以 Playwright 於實際渲染結果驗證，現在需要把行為固化進規格，避免日後被誤改回舊行為。

## What Changes

- Provider 縮寫代碼改為單一定義來源 `getProviderAbbr()`（`src/utils/providerLabel.ts`），移除四處重複/衝突的常數定義，全 App 呼叫端一致
- Dashboard Kanban ProjectCard 標題列版面調整優先序：
  - 專案名稱保有最小可讀寬度（不得被壓縮至不可見）
  - 分支名稱可壓縮並以省略號截斷，空間不足時優先讓出給專案名稱
  - Provider 標籤超過顯示上限（2 個）時，以 `+N` 摘要呈現，不再無聲裁切或無限撐開版面

## Capabilities

### New Capabilities
（無）

### Modified Capabilities
- `provider-tag`: 新增「provider 縮寫代碼」需求 — 縮寫代碼 SHALL 有單一定義來源，各處呈現一致，且涵蓋 claude/copilot/opencode/codex/antigravity
- `dashboard-kanban`: 修改「Kanban 視圖跨專案顯示」中 ProjectCard 標頭相關 Scenario — 標頭內容在空間不足時的截斷/省略優先序需求

## Impact

- `src/utils/providerLabel.ts`：新增 `getProviderAbbr()`
- `src/components/ProviderIcon.tsx`、`src/components/DashboardView.tsx`、`src/components/QuotaOverlay.tsx`、`src/components/StatusBar.tsx`：改用共用函式，移除重複常數
- `src/components/DashboardView.tsx`：ProjectCard 標頭新增 provider 標籤數量上限與 `+N` 摘要邏輯
- `src/App.css`：`.kanban-project-name`、新增 `.kanban-project-branch`、`.kanban-project-providers-more` 的版面規則
