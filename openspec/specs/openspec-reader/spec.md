## ADDED Requirements

### Requirement: 掃描 openspec 目錄

系統 SHALL 掃描專案工作目錄下的 `openspec/` 子目錄，讀取 specs 與 changes 結構，並在 ProjectView 的 Plans & Specs 子分頁中顯示。

#### Scenario: 偵測 openspec 存在

- **WHEN** 系統掃描 session 的 cwd
- **THEN** 若 `<cwd>/openspec/` 存在，SessionInfo 的相關標記設為 true
- **AND** ProjectView 顯示 Plans & Specs 子分頁入口

#### Scenario: 讀取 specs 清單

- **WHEN** 使用者開啟 Plans & Specs 子分頁
- **THEN** 系統列出 `<cwd>/openspec/specs/` 下所有子目錄名稱
- **AND** 每個子目錄代表一個 spec 項目，點擊可查看 spec.md 內容

#### Scenario: 讀取 active changes 清單

- **WHEN** 使用者開啟 Plans & Specs 子分頁
- **THEN** 系統列出 `<cwd>/openspec/changes/`（不含 archive）下的 change 資料夾
- **AND** 顯示每個 change 的 tasks.md 完成進度（已勾 / 總數）

### Requirement: 讀取 OpenSpec 文件內容

系統 SHALL 提供 `read_openspec_file(project_cwd, relative_path)` Tauri command，讀取指定 openspec 目錄下的 md 文件內容。

#### Scenario: 成功讀取文件內容

- **WHEN** 前端呼叫 `read_openspec_file` 並傳入有效的 project_cwd 與 relative_path
- **AND** relative_path 對應的檔案存在且在 openspec 目錄下
- **THEN** 系統回傳檔案的完整 UTF-8 文字內容

#### Scenario: 路徑安全驗證失敗

- **WHEN** relative_path 包含 `..` 或正規化後逃逸出 `<project_cwd>/openspec/` 目錄
- **THEN** 系統回傳 `Err("path traversal not allowed")`，不讀取任何檔案

#### Scenario: 檔案不存在

- **WHEN** 指定路徑的檔案不存在
- **THEN** 系統回傳 `Err("file not found")`
