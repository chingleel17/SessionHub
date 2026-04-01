## ADDED Requirements

### Requirement: .sisyphus 目錄偵測
系統 SHALL 掃描指定專案目錄，判斷是否存在 `.sisyphus/` 子目錄。

#### Scenario: .sisyphus 存在
- **WHEN** 專案目錄下存在 `.sisyphus/` 子目錄
- **THEN** 系統回傳該目錄下的完整 `SisyphusData` 結構

#### Scenario: .sisyphus 不存在
- **WHEN** 專案目錄下不存在 `.sisyphus/` 子目錄
- **THEN** 系統回傳空結構（`active_plan: None`、空 `plans`/`notepads`/`evidence_files`/`draft_files`），不產生錯誤

### Requirement: boulder.json 解析
系統 SHALL 讀取並解析 `.sisyphus/boulder.json`，取得目前 active plan 狀態與 session 關聯。

#### Scenario: boulder.json 存在且格式正確
- **WHEN** `.sisyphus/boulder.json` 存在
- **THEN** 系統解析回傳 `SisyphusBoulder` 結構，包含：`active_plan`（plan 路徑）、`plan_name`、`agent`、`session_ids`（關聯的 OpenCode session ID 清單）、`started_at`

#### Scenario: boulder.json 不存在或格式異常
- **WHEN** `.sisyphus/boulder.json` 不存在或 JSON 解析失敗
- **THEN** 系統將 `active_plan` 設為 `None`，不影響其餘資料讀取

### Requirement: Plans 清單讀取
系統 SHALL 掃描 `.sisyphus/plans/` 目錄下所有 `.md` 檔案，建立 plan 清單。

#### Scenario: plans 目錄含有 Markdown 檔案
- **WHEN** `.sisyphus/plans/` 下存在 `*.md` 檔案
- **THEN** 系統回傳 `Vec<SisyphusPlan>`，每筆包含：`name`（檔名去除副檔名）、`path`（完整路徑）、`title`（從 `# heading` 取得）、`tldr`（從 `## TL;DR` section 取得前幾行）、`is_active`（比對 `boulder.json` 的 `active_plan`）

#### Scenario: plans 目錄不存在或為空
- **WHEN** `.sisyphus/plans/` 不存在或無 `.md` 檔案
- **THEN** 系統回傳空 `Vec<SisyphusPlan>`

### Requirement: Notepads 清單讀取
系統 SHALL 掃描 `.sisyphus/notepads/` 目錄下所有子目錄，建立 notepad 清單。

#### Scenario: notepads 目錄含有子目錄
- **WHEN** `.sisyphus/notepads/` 下存在子目錄（如 `api-design/`）
- **THEN** 系統回傳 `Vec<SisyphusNotepad>`，每筆包含：`name`（子目錄名稱）、`has_issues`（是否存在 `issues.md`）、`has_learnings`（是否存在 `learnings.md`）

#### Scenario: notepads 目錄不存在
- **WHEN** `.sisyphus/notepads/` 不存在
- **THEN** 系統回傳空 `Vec<SisyphusNotepad>`

### Requirement: Evidence 與 Drafts 清單讀取
系統 SHALL 掃描 `.sisyphus/evidence/` 與 `.sisyphus/drafts/` 目錄，回傳檔案清單。

#### Scenario: evidence 目錄含有檔案
- **WHEN** `.sisyphus/evidence/` 下存在 `*.txt` 檔案
- **THEN** 系統回傳 `evidence_files: Vec<String>`（檔名清單）

#### Scenario: drafts 目錄含有檔案
- **WHEN** `.sisyphus/drafts/` 下存在 `*.md` 檔案
- **THEN** 系統回傳 `draft_files: Vec<String>`（檔名清單）

### Requirement: Markdown 內容讀取
系統 SHALL 提供依路徑讀取 plan/notepad/evidence/draft 檔案原始內容的能力。

#### Scenario: 讀取指定 plan 內容
- **WHEN** 使用者點擊某個 plan 條目
- **THEN** 系統讀取該 `.md` 檔案完整內容回傳，供前端 Markdown 預覽顯示

#### Scenario: 檔案不存在
- **WHEN** 指定路徑的檔案不存在
- **THEN** 系統回傳錯誤訊息
