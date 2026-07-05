## ADDED Requirements

### Requirement: Commands 來源掃描與 per-agent 目錄對映

系統 SHALL 以 `.agents/skills/command/**/*.md`（專案範圍）或 `~/.agents/skills/command/**/*.md`（全域範圍）為 commands 來源，對每個 agent 的 command 目錄計算同步狀態並以矩陣呈現。各 agent 的目標目錄對映 SHALL 為：claude → `.claude/commands/`（保留來源子路徑，如 `opsx/apply.md`）、codex → `.codex/prompts/`、opencode → `.opencode/command/`、copilot → `.copilot/prompts/`；全域範圍以各 agent 全域根目錄為基準（opencode 為 `~/.config/opencode/command`）。

#### Scenario: 顯示 commands 狀態矩陣

- **WHEN** 使用者開啟 Agents 分頁的 Commands 子分頁
- **THEN** 系統列出來源目錄下的所有 command md 檔（名稱含子路徑，如 `opsx/apply`），每欄一個目標 agent
- **AND** 每格顯示一致 / 缺少 / 內容不同 / 目標較新狀態

#### Scenario: 檢視 command 內容

- **WHEN** 使用者點擊矩陣中的 command 名稱
- **THEN** 右側面板以 markdown 渲染該 command 檔內容（含 frontmatter）

#### Scenario: 只列出來源存在的 command

- **WHEN** 某 target 目錄下存在一個來源已不存在的同名 command
- **THEN** 系統不為其產生矩陣列，該 command 不參與同步（與 agents-skills-sync 相同限制）

### Requirement: Commands 同步與格式轉換擴充點

系統 SHALL 支援將勾選的 commands 複製到勾選的目標 agent 目錄，沿用 dry-run 預覽、衝突詢問與記住選擇機制。v1 SHALL 為位元組複製（不轉換 frontmatter），但同步引擎 SHALL 保留可插拔的內容轉換擴充點（CommandAdapter），供日後實作各 agent 的 frontmatter 格式適配。

#### Scenario: 同步選取的 commands

- **WHEN** 使用者勾選若干 commands 與目標、預覽後套用
- **THEN** 系統將來源 md 檔複製到各目標目錄的對應路徑（保留子路徑、必要時建立目錄）
- **AND** 已一致者略過；完成後更新矩陣並顯示摘要

#### Scenario: 目標端 command 較新

- **WHEN** 同步時某目標端 command 檔 mtime 比來源新且內容不同，且無記住的衝突選擇
- **THEN** 系統依衝突流程詢問方向，支援記住選擇

#### Scenario: v1 純複製

- **WHEN** 執行 command 同步
- **THEN** 目標檔內容與來源檔位元組相同（透過 Passthrough 轉換器）
- **AND** 轉換器介面允許日後依 target_id 替換為格式適配實作，無需改動同步管線
