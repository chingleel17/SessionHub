## Why

Dashboard 目前的狀態卡片區域佔用大量版面，token/互動統計缺乏時間範圍限制導致效能浪費，tab 標頭高度在首頁與專案頁切換時產生抖動，sidebar 收合後版本號消失，以及 session card 的對話時長永遠以分鐘顯示而沒有自動換算成小時，這些問題累積起來使 UI 顯得冗餘且不精緻。

## What Changes

- **Dashboard 統計改為時間範圍**：token 用量與互動次數新增「本周 / 本月」切換器，只統計對應時間範圍內更新的 session，減少不必要的全量查詢
- **統計卡片精簡化**：將 7 張 `stat-card` 合併為單列緊湊 stat bar，每項搭配 icon，縮減垂直高度
- **Tab 高度統一**：專案 tab header 的路徑縮小字體顯示；首頁 header 加入一行副標題，使兩者行高一致，消除切換時的抖動
- **Sidebar 收合版本號**：收合狀態下在 footer 顯示縮短版本號（`v0.1`）tooltip 顯示完整版本
- **對話時長自動換算**：`durationMinutes >= 60` 時換算並顯示為小時（`1.5h`），不足 60 分鐘則保持分鐘（`45m`）

## Capabilities

### New Capabilities

- `dashboard-stats-period`: Dashboard token/互動統計支援本周 / 本月時間範圍篩選與切換

### Modified Capabilities

- `dashboard`: 統計卡片改為 stat bar 樣式，並加入時間範圍篩選
- `tabbed-ui`: tab header 高度統一（首頁副標題 + 路徑縮小字體）
- `session-stats-display`: `durationMinutes` 格式化邏輯加入小時自動換算

## Impact

- `src/components/DashboardView.tsx`：新增 period 切換 UI，重構 stats 區塊為 stat bar
- `src/components/SessionStatsBadge.tsx`：`formatDuration` 函式換算小時
- `src/components/Sidebar.tsx`：收合時 footer 顯示縮短版本號
- `src/App.tsx`：傳入 dashboard period 狀態與 filtered totals；workspace-header 加入首頁副標題
- `src/styles/`：新增 stat bar、tab header 路徑縮小字體相關 CSS
- `src/locales/`：新增翻譯 key（period 切換、時長格式）
