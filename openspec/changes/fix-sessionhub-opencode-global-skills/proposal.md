## Why

從 SessionHub 開啟的終端執行 `opencode` 時，只載入到 7 個專案內 skills，缺少 `~/.agents\skills` 與 `~/.claude\skills` 的 14 個全域 skills；同一台機器、同一個專案目錄、同一個 `opencode.exe`（1.18.4），使用者自行開啟的終端卻能正常載入 21 個。使用者在 SessionHub 啟動的終端中因此失去大部分工作流程能力，且此問題曾被修正過又復發，代表既有 spec 未涵蓋真正的失效條件，需要以可重現的診斷證據建立防護。

## What Changes

- 建立可重複執行的診斷程序，用以判定 SessionHub 啟動的子程序與使用者手動終端之間，導致 OpenCode 全域 skill 掃描失效的最小差異
- 修正 SessionHub 建立終端／CLI 子程序的環境或啟動方式，使 OpenCode 在 SessionHub 終端中掃描到的 skill 根目錄集合與手動終端一致
- 針對 `~/.agents`、`~/.claude` 由 SessionHub 以 symlink/junction 管理的情境，確保 SessionHub 啟動的子程序仍能完整遍歷這些連結
- 新增回歸防護：SessionHub 對子程序的環境注入 SHALL NOT 移除或覆寫使用者既有的 OpenCode 相關環境變數（含使用者明確設定的停用旗標）
- 不硬編任何 skills 路徑，不無條件注入 `HOME`，不清除使用者環境變數

## Capabilities

### New Capabilities

- `terminal-child-env-parity`: 定義 SessionHub 啟動子程序（終端、AI coding CLI、session resume）時，其環境區塊與檔案系統可視性相對於使用者手動終端所必須維持的一致性契約，以及禁止破壞使用者既有設定的界線

### Modified Capabilities

- `terminal-launcher`: 既有「Windows 終端與 CLI 啟動環境一致性」需求目前僅涵蓋 MSYS stackdump 緩解；需擴充為所有啟動入口共用同一套完整的環境傳遞規則，並明確要求不得遺失使用者環境變數

## Impact

- **程式碼**
  - `src-tauri/src/sessions/mod.rs` — `configure_msys_stackdump_suppression`、`merge_msys_options`
  - `src-tauri/src/sessions/copilot.rs` — `open_terminal_internal`
  - `src-tauri/src/commands/tools.rs` — `open_in_tool_internal`、`resume_session_in_terminal_internal`
  - `src-tauri/src/agents_config.rs` — `~/.agents` 與 `~/.claude` 的 skills 連結與同步處理
- **外部相依**：OpenCode CLI 1.18.4 的 skill discovery 行為（global 與 project 掃描共用同一條 code path，scoped scan 失敗會被靜默吞掉並回傳空集合）
- **平台**：僅影響 Windows；symlink/junction 遍歷權限為關鍵變因
- **不影響**：SessionHub 的 session 掃描、quota、hook 整合等既有功能；不變更任何啟動指令、參數、工作目錄或 console creation flags
