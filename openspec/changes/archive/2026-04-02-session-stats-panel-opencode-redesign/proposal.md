## Why

SessionStatsPanel 目前有兩個問題：

1. **OpenCode 統計顯示全零**：OpenCode session 的 `get_session_stats` 呼叫接收的是 OpenCode session directory 路徑格式，後端 `get_session_stats_internal` 已能區分並呼叫 `get_opencode_session_stats_internal`，但前端傳入的 `session_dir` 欄位來自 `SessionInfo.session_dir`，而 OpenCode session 目前 `session_dir` 設為空字串（`String::new()`），導致所有統計歸零。

2. **Panel 排版設計待改善**：現有 SessionStatsPanel 將所有統計項目以垂直列表呈現，在 session 有大量工具呼叫時 tool breakdown 表格佔據過多空間，可讀性差；且與 SessionStatsBadge 的視覺風格不一致，整體 UX 需要打磨。

## What Changes

**後端（Rust）：**

- OpenCode session 的 `session_dir` 欄位改為儲存 `storage/message/<session_id>/` 路徑（OpenCode stats 所需的 message 目錄），讓後端 `get_session_stats_internal` 的 `is_opencode_session_dir` 判斷能正確觸發 OpenCode 解析路徑。

**前端（React/TypeScript）：**

- SessionStatsPanel 改為兩欄式卡片式佈局：左欄顯示核心數字（tokens、互動、時長），右欄顯示模型與工具明細。
- 加入 inputTokens 顯示（大於 0 時才顯示，條件性渲染）。
- Tool breakdown 改為緊湊表格，限制最大高度並加上 scroll。
- Live session 加入動態呼吸動畫指示器。
- SessionStatsBadge 加入 `isLive` 的 LIVE badge（已有型別，補充 UI）。

## Capabilities

### Modified Capabilities

- `session-stats-display`：Panel 改版佈局 + inputTokens 欄位 + live 動畫

### Modified Data Flow

- `opencode-json-parser`：OpenCode `session_dir` 改為 message 目錄路徑，解決統計零值問題

## Impact

- `src-tauri/src/lib.rs`：`scan_opencode_sessions_internal` 與 `scan_opencode_incremental_internal` 中，`session_dir` 欄位改為填入 `<opencode_root>/message/<session_id>/` 路徑。
- `src/components/SessionStatsPanel.tsx`：排版重構為雙欄 Grid，加入 inputTokens 條件顯示、tool breakdown scroll、live 動畫。
- `src/components/SessionStatsBadge.tsx`：補充 LIVE badge UI（`isLive: true` 時顯示）。
- `src/styles/` 或 `App.css`：新增 panel 雙欄 grid CSS class 與 live 動畫 keyframe。
- `src/locales/zh-TW.ts` 及 `en-US.ts`：新增 `inputTokens`、`sessionLive` 等 i18n key。
