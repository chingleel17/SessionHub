## 1. 後端：修正 OpenCode session_dir

- [x] 1.1 在 `scan_opencode_sessions_internal` fn 中，將 `SessionInfo.session_dir` 改為填入 `<opencode_root>/message/<session_id>/` 路徑（使用 `opencode_root.join("message").join(&id).to_string_lossy().to_string()`）
- [x] 1.2 在 `scan_opencode_incremental_internal` fn 中，同樣修正 `session_dir` 欄位填值方式
- [ ] 1.3 手動驗證：以實際 OpenCode session 呼叫 `get_session_stats`，確認 outputTokens > 0

## 2. 後端：has_events 修正（OpenCode）

- [x] 2.1 在 `scan_opencode_sessions_internal` 中，`has_events` 改為：判斷 `<opencode_root>/message/<session_id>/` 目錄存在且非空（`fs::read_dir` 有回傳至少一個 entry）
- [x] 2.2 在 `scan_opencode_incremental_internal` 中，同上修正 `has_events`

## 3. 前端：SessionStatsPanel 排版重構

- [x] 3.1 在 `src/components/SessionStatsPanel.tsx` 中，將現有垂直清單改為兩欄 Grid 結構：左欄（核心數字）、右欄（模型與工具明細）
- [x] 3.2 左欄顯示：outputTokens、inputTokens（`> 0` 時才渲染）、interactionCount、toolCallCount、reasoningCount（`> 0` 時才渲染）、durationMinutes
- [x] 3.3 右欄顯示：modelsUsed 清單、toolBreakdown 表格
- [x] 3.4 toolBreakdown 表格加上 `max-height: 160px; overflow-y: auto;` 的 scroll 容器
- [x] 3.5 toolBreakdown 表格依呼叫次數降冪排列（已有邏輯確認或新增 sort）

## 4. 前端：Live Session 動畫指示器

- [x] 4.1 在 `src/App.css`（或對應 css 檔）中新增綠色 `.stats-live-dot` class，包含主點脈動與向外擴散閃爍的 ripple 動畫（`@keyframes stats-live-pulse` + `@keyframes stats-live-ripple`）
- [x] 4.2 在 SessionStatsPanel 左欄底部，當 `stats.isLive === true` 時顯示：pulse dot + i18n 文字（`t("statsLiveIndicator")`）
- [x] 4.3 在 SessionStatsBadge 中，當 `stats.isLive === true` 時顯示 `LIVE` badge（小型、accent 顏色背景）

## 5. 前端：CSS 樣式

- [x] 5.1 新增 `.stats-panel-grid` CSS class：`display: grid; grid-template-columns: 1fr 1fr; gap: 12px;`
- [x] 5.2 新增 `.stats-panel-col` CSS class 作為左右欄容器（padding、border-radius）
- [x] 5.3 新增 `.stats-tool-breakdown-scroll` CSS class：`max-height: 160px; overflow-y: auto;`
- [x] 5.4 確認 RWD：若 panel 寬度不足（`< 400px`），使用 `@media` 回退為單欄

## 6. i18n

- [x] 6.1 在 `src/locales/zh-TW.ts` 新增 key：`inputTokens`（「輸入 Token」）、`statsLiveIndicator`（「進行中」）
- [x] 6.2 在 `src/locales/en-US.ts` 新增對應 key：`inputTokens`（"Input Tokens"）、`statsLiveIndicator`（"Live"）

## 7. 測試

- [x] 7.1 新增 Rust unit test：`test_opencode_session_stats_session_dir`：建立 mock message 目錄（含 msg\_\*.json），驗證 `get_session_stats_internal` 回傳非零 outputTokens
- [ ] 7.2 手動測試（OpenCode session）：確認 SessionStatsPanel 顯示正確的 outputTokens、interactionCount、toolCallCount、durationMinutes
- [ ] 7.3 手動測試（Copilot session）：確認現有統計欄位顯示不受影響
- [ ] 7.4 手動測試（Live session）：確認 isLive=true 時顯示綠點，且有明顯向外擴散閃爍效果，LIVE badge 正確顯示
