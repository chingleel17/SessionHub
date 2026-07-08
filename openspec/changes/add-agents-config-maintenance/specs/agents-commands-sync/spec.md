## ADDED Requirements

### Requirement: Commands 來源掃描與 per-agent 目錄對映

系統 SHALL 以 `.agents/skills/command/**/*.md`（專案範圍）或 `~/.agents/skills/command/**/*.md`（全域範圍）為 commands 來源，對每個 agent 的 command 目錄計算同步狀態並以矩陣呈現。各 agent 的目標目錄對映 SHALL 為：claude → `.claude/commands/`（保留來源子路徑，如 `opsx/apply.md`）、codex → `.codex/prompts/`、opencode → `.opencode/command/`、copilot → 專案範圍優先 `.github/prompts/`，舊慣例 `.copilot/prompts/` 亦須相容；全域範圍以各 agent 全域根目錄為基準（opencode 為 `~/.config/opencode/command`）。

#### Scenario: 顯示 commands 狀態矩陣

- **WHEN** 使用者開啟 Agents 分頁的 Commands 子分頁
- **THEN** 系統列出來源目錄與各 target 目錄的 command 聯集（名稱含子路徑，如 `opsx/apply`），每欄一個目標 agent
- **AND** 每格顯示一致（in-sync）/ 缺少目標（target-missing）/ 內容不同（differs）/ 僅有目標（source-missing）/ 目標較新（target-newer）狀態

#### Scenario: 檢視 command 內容

- **WHEN** 使用者點擊矩陣中的 command 名稱
- **THEN** 右側面板以 markdown 渲染該 command 檔內容（含 frontmatter）

#### Scenario: target 端既有 command 仍應顯示於矩陣

- **WHEN** `.agents/skills/command/` 尚未建立或缺少某 command，但某 target 端（如 `.claude/commands/`、`.opencode/command/` 或 `.github/prompts/`）已存在同名 command
- **THEN** 系統仍 SHALL 在矩陣中顯示該 command 列，讓使用者可檢視現況並透過衝突流程決定是否回補來源
- **AND** 該 target 欄狀態標示為「僅有目標」（source-missing）
- **AND** 實際同步時仍以預期來源路徑 `.agents/skills/command/...` 作為 canonical source path

#### Scenario: 因其他 target 反查而產生的列，本 target 亦缺少該 command

- **WHEN** 某 command 列是因「另一個 target 端存在同名檔案」而被反查列出（來源尚未建立），而**目前這一欄**的 target 也不存在該檔案
- **THEN** 系統 SHALL 將該欄狀態標示為「缺少目標」（target-missing），與一般「來源存在但此 target 尚未同步」的情況一致對待
- **AND** 系統 SHALL NOT 將此情況標示為「錯誤」（error）狀態——error 狀態僅保留給檔案讀寫/連結解析等真正的例外錯誤，不可用於「來源與目標單純皆不存在」的正常情境
- **AND** 此為 2026-07 實地驗證發現的既有缺陷修正：修正前 `classify_file_status` 對「來源與目標皆不存在」回傳 `SyncStatus::Error`，導致 UI 顯示中性的「錯誤」標籤，使用者誤以為該 command 未被正確識別，實際上只是尚未同步到該 target

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
