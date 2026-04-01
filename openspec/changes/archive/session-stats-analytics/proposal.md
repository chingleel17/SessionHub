## Why

目前 ProjectView 只顯示 session 卡片清單，缺乏數量摘要與使用統計。每個 session 的 `events.jsonl` 中已記錄完整的互動事件，可從中提取豐富的使用指標（token 用量、工具調用、互動次數、時長、使用模型），但目前完全未被利用。增加統計分析功能可幫助使用者評估 Copilot 使用效率，並了解各專案的工作負荷。

## What Changes

- **專案頁面**新增 session 數量顯示（含空/封存計數）
- **Session 卡片**新增摘要統計徽章（互動次數、輸出 token、對話時長）
- **Session 詳情頁面**（新）：點擊 session 卡片可展開/開啟詳細統計面板，包含：
  - 對話時長
  - 使用者互動次數
  - 工具調用次數（依工具名稱分組）
  - 輸出 token 總量（及每次互動平均）
  - 使用的 AI 模型清單（及各模型佔比）
  - Reasoning 使用次數估算（进階請求指標）
- **專案統計**：專案頁面頂部顯示該專案所有 session 的彙總統計
- **Rust 後端**新增 `parse_session_stats` command，解析 `events.jsonl` 並快取結果至 SQLite
- **Dashboard** 數據更豐富：總 token 用量、總工具調用次數

## Capabilities

### New Capabilities

- `session-stats`: 解析 `events.jsonl`，提取並快取單一 session 的統計指標（token、互動次數、工具調用、時長、模型使用）
- `session-stats-view`: Session 詳細統計 UI（卡片摘要徽章 + 詳情展開面板）
- `project-stats-summary`: 專案頁面頂部的彙總統計 banner（session 數、總 token、總互動）

### Modified Capabilities

- `session-list`: Session 卡片新增統計摘要徽章顯示
- `dashboard`: Dashboard 統計數據擴充，加入 token 與工具調用總覽

## Impact

- **新增 Rust command**: `get_session_stats(session_dir)` → `SessionStats`
- **SQLite schema 擴充**: 新增 `session_stats` 表快取解析結果（含 `parsed_at` 欄位避免重複解析）
- **新增前端元件**: `SessionStatsPanel.tsx`（詳情面板）、`SessionStatsBadge.tsx`（卡片徽章）
- **修改**: `SessionCard.tsx`（加入徽章）、`ProjectView.tsx`（加入專案統計 banner）、`DashboardView.tsx`（擴充統計）
- **效能考量**: `events.jsonl` 最大可達數 MB，解析結果必須快取至 SQLite，不可每次重新解析
