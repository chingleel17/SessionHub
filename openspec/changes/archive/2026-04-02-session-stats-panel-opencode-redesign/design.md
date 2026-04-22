## Context

### OpenCode session_dir 問題根因

後端 `scan_opencode_sessions_internal`（全量掃描）與 `scan_opencode_incremental_internal`（增量掃描）在建立 `SessionInfo` 時，`session_dir` 欄位設為 `String::new()`（空字串）。

`get_session_stats_internal` 靠 `is_opencode_session_dir` 判斷是走 OpenCode 路徑還是 Copilot 路徑：

```rust
fn is_opencode_session_dir(session_dir: &Path) -> bool {
    session_dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("ses_"))
        .unwrap_or(false)
}
```

空字串 → file_name() 為 None → `is_opencode_session_dir` 回傳 `false` → 走 Copilot 路徑 → `events.jsonl` 不存在 → 統計全零。

**修正方式：** `session_dir` 改為填入 `<opencode_root>/message/<session_id>/`，這是 `calculate_opencode_session_stats` 的 `message_dir` 參數所需路徑，且目錄名 `ses_xxx` 能觸發 `is_opencode_session_dir` 的判斷。

### SessionStatsPanel 現況

現有 SessionStatsPanel（約 150 行）以垂直清單顯示統計項目，無明顯分區：

```
outputTokens: 12.3K
inputTokens: (隱藏，即使有值)
interactionCount: 5 turns
toolCallCount: 23
durationMinutes: 45m
reasoningCount: 2
modelsUsed: [claude-3-7...]
--- tool breakdown table ---
Tool         Count
bash         15
glob          5
...
```

問題：

- inputTokens 在 UI 中完全缺失（後端有計算，前端 Panel 未顯示）
- tool breakdown 在工具多時撐開整個 panel，無 scroll
- live indicator 只有靜態文字

## Goals / Non-Goals

**Goals:**

- 修正 OpenCode `session_dir` 空字串問題，使 `get_session_stats` 正確走 OpenCode 解析路徑。
- SessionStatsPanel 改為兩欄雙卡式佈局，提升可讀性。
- 加入 inputTokens 條件顯示（> 0 時顯示）。
- Tool breakdown 加最大高度 + scroll。
- Live session 加入動態指示（pulse 動畫）。
- SessionStatsBadge LIVE badge 補充 UI 樣式。

**Non-Goals:**

- 修改 SessionStats 的 Rust struct 欄位（新增/刪除欄位）。
- 實作 OpenCode session 的 `has_plan`（留待後續 change）。
- 調整整體 SessionCard 佈局。

## UI Design

### SessionStatsPanel 雙欄佈局

```
┌─────────────────────────────────────────────────────────┐
│  stats-panel-grid (display: grid; grid-template-columns: 1fr 1fr)  │
│  ┌──────────────────────┐  ┌──────────────────────────┐ │
│  │ 左欄（核心數字）        │  │ 右欄（模型與工具）          │ │
│  │                      │  │                          │ │
│  │ 🔢 輸出 Token  12.3K  │  │ 🤖 模型                  │ │
│  │ 📥 輸入 Token   4.1K  │  │   claude-3-7-sonnet      │ │
│  │ 💬 互動次數    5 次   │  │                          │ │
│  │ 🔧 工具呼叫   23 次   │  │ 🔧 工具明細              │ │
│  │ 🧠 推理        2 次   │  │ ┌──────────┬───────┐     │ │
│  │ ⏱ 時長        45m    │  │ │ bash     │  15   │     │ │
│  │                      │  │ │ glob     │   5   │     │ │
│  │ [LIVE 跳動指示器]     │  │ │ read     │   3   │     │ │
│  │                      │  │ └──────────┴───────┘     │ │
│  │                      │  │  (max-height: 160px;      │ │
│  │                      │  │   overflow-y: auto)       │ │
│  └──────────────────────┘  └──────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Live 動畫

```css
@keyframes stats-live-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.4;
  }
}

.stats-live-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--color-accent);
  animation: stats-live-pulse 1.5s ease-in-out infinite;
}
```

## Decisions

### D1：session_dir 填入 message 目錄路徑

**決定**：`<opencode_root>/message/<session_id>/` 作為 OpenCode session 的 `session_dir` 值。

**理由**：

1. 目錄名為 `ses_xxx`，能觸發現有 `is_opencode_session_dir` 判斷。
2. 這正是 `get_opencode_session_stats_internal` 的 `message_dir` 參數。
3. 不需要修改 `get_session_stats_internal` 的邏輯。

**替代方案考量**：修改 `is_opencode_session_dir` 改用 `provider` 欄位判斷 → 需要同時修改 `get_session_stats_internal` 的簽章，改動範圍較大。

### D2：Panel 改為兩欄 Grid，不使用新 UI library

**決定**：純 CSS Grid，沿用現有 CSS class 命名慣例（BEM-like）。

**理由**：專案禁止引入 CSS framework，純 CSS 維護成本低。
