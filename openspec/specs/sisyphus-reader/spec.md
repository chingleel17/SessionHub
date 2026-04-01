## ADDED Requirements

### Requirement: 讀取 .sisyphus 目錄結構

系統 SHALL 支援讀取專案目錄下的 `.sisyphus/` 資料夾，作為另一種 AI task 管理工具的 plan 資料來源。

#### Scenario: 偵測 .sisyphus 存在

- **WHEN** 系統掃描 session 的 cwd
- **THEN** 若 `<cwd>/.sisyphus/` 存在，SessionInfo 標記 has_sisyphus 為 true

#### Scenario: 讀取 .sisyphus task 檔案

- **WHEN** 使用者在 Plans & Specs 子分頁查看 .sisyphus 內容
- **THEN** 系統列出 `.sisyphus/` 下的 task 檔案（`.md` 或 `.json` 格式）
- **AND** 依檔案修改時間降冪排列

### Requirement: .sisyphus task 顯示

系統 SHALL 在 Plans & Specs 子分頁提供 .sisyphus task 的內容預覽。

#### Scenario: 查看 task 內容

- **WHEN** 使用者點擊 .sisyphus task 項目
- **THEN** 系統顯示該 task 的完整 Markdown 或 JSON 內容（唯讀）
