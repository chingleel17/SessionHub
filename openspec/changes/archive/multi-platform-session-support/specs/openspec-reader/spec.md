## ADDED Requirements

### Requirement: openspec 目錄偵測
系統 SHALL 掃描指定專案目錄，判斷是否存在 `openspec/` 子目錄。

#### Scenario: openspec 存在
- **WHEN** 專案目錄下存在 `openspec/` 子目錄
- **THEN** 系統回傳該目錄下的完整 `OpenSpecData` 結構

#### Scenario: openspec 不存在
- **WHEN** 專案目錄下不存在 `openspec/` 子目錄
- **THEN** 系統回傳空結構（`schema: None`、空 `active_changes`/`archived_changes`/`specs`），不產生錯誤

### Requirement: config.yaml 解析
系統 SHALL 讀取並解析 `openspec/config.yaml`，取得 schema 設定資訊。

#### Scenario: config.yaml 存在且格式正確
- **WHEN** `openspec/config.yaml` 存在
- **THEN** 系統解析並回傳 `schema` 欄位值（如 `"spec-driven"`）

#### Scenario: config.yaml 不存在或格式異常
- **WHEN** `openspec/config.yaml` 不存在或 YAML 解析失敗
- **THEN** 系統將 `schema` 設為 `None`，不影響其餘資料讀取

### Requirement: Active Changes 清單讀取
系統 SHALL 掃描 `openspec/changes/` 目錄下的子目錄（排除 `archive/`），建立進行中 change 清單。

#### Scenario: 存在進行中的 change
- **WHEN** `openspec/changes/` 下存在子目錄（如 `multi-platform-session-support/`）
- **THEN** 系統回傳 `Vec<OpenSpecChange>`，每筆包含：`name`（目錄名稱）、`has_proposal`（是否存在 `proposal.md`）、`has_design`（是否存在 `design.md`）、`has_tasks`（是否存在 `tasks.md`）、`specs_count`（`specs/` 子目錄數量）

#### Scenario: 無進行中的 change
- **WHEN** `openspec/changes/` 不存在或無子目錄（除 `archive/` 外）
- **THEN** 系統回傳空 `Vec<OpenSpecChange>`

### Requirement: Archived Changes 清單讀取
系統 SHALL 掃描 `openspec/changes/archive/` 目錄下的子目錄，建立已封存 change 清單。

#### Scenario: 存在已封存的 change
- **WHEN** `openspec/changes/archive/` 下存在子目錄
- **THEN** 系統回傳 `archived_changes: Vec<OpenSpecChange>`，結構與 active change 相同

#### Scenario: archive 目錄不存在
- **WHEN** `openspec/changes/archive/` 不存在
- **THEN** 系統回傳空 `Vec<OpenSpecChange>`

### Requirement: Specs 清單讀取
系統 SHALL 掃描 `openspec/specs/` 目錄下的子目錄，建立累積規格清單。

#### Scenario: 存在 spec 目錄
- **WHEN** `openspec/specs/` 下存在子目錄（如 `app-settings/`）
- **THEN** 系統回傳 `Vec<OpenSpecSpec>`，每筆包含：`name`（目錄名稱）、`path`（`spec.md` 完整路徑）

#### Scenario: specs 目錄不存在
- **WHEN** `openspec/specs/` 不存在
- **THEN** 系統回傳空 `Vec<OpenSpecSpec>`

### Requirement: Change/Spec 內容讀取
系統 SHALL 提供依路徑讀取 change 文件（proposal.md、design.md、tasks.md）與 spec 文件（spec.md）原始內容的能力。

#### Scenario: 讀取指定文件內容
- **WHEN** 使用者點擊某個 change 或 spec 條目
- **THEN** 系統讀取該 `.md` 或 `.yaml` 檔案完整內容回傳，供前端 Markdown 預覽顯示

#### Scenario: 檔案不存在
- **WHEN** 指定路徑的檔案不存在
- **THEN** 系統回傳錯誤訊息
