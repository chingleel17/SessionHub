## MODIFIED Requirements

### Requirement: 讀取 .sisyphus 目錄結構

系統 SHALL 支援讀取專案目錄下的 `.sisyphus/` 資料夾，作為另一種 AI task 管理工具的 plan 資料來源，且所有可瀏覽的檔案項目 SHALL 提供可直接開啟的絕對路徑。

#### Scenario: 偵測 .sisyphus 存在

- **WHEN** 系統掃描 session 的 cwd
- **THEN** 若 `<cwd>/.sisyphus/` 存在，SessionInfo 標記 has_sisyphus 為 true

#### Scenario: 讀取 .sisyphus task 檔案

- **WHEN** 使用者在 Plans & Specs 子分頁查看 .sisyphus 內容
- **THEN** 系統列出 `.sisyphus/` 下的 plan 與 draft 檔案（`.md` 格式）
- **AND** 每個檔案項目帶有可直接讀取的絕對路徑

#### Scenario: notepad 提供可讀取的檔案路徑

- **WHEN** `.sisyphus/notepads/<name>/` 下存在 `issues.md` 或 `learnings.md`
- **THEN** 系統為該 notepad 回傳對應檔案的絕對路徑
- **AND** 使用者可在 Explorer 中展開該 notepad 並點擊開啟其內容

#### Scenario: 不存在的 notepad 檔案不產生節點

- **WHEN** 某 notepad 僅具備 `issues.md` 而無 `learnings.md`
- **THEN** 系統僅提供 `issues.md` 的路徑
- **AND** Explorer 不顯示無法開啟的 `learnings.md` 節點

### Requirement: .sisyphus task 顯示

系統 SHALL 在 Plans & Specs 子分頁提供 .sisyphus plan、notepad 與 draft 的內容預覽，並 SHALL NOT 將 evidence 作為可瀏覽節點呈現。

#### Scenario: 查看 task 內容

- **WHEN** 使用者點擊 .sisyphus 的 plan、notepad 檔案或 draft 項目
- **THEN** 系統顯示該檔案的完整 Markdown 內容（唯讀）

#### Scenario: evidence 不出現在 Explorer

- **WHEN** `.sisyphus/evidence/` 下存在任意 `.txt` 檔案
- **THEN** Explorer 不顯示 Evidence 群組或任何 evidence 檔案節點

#### Scenario: 僅有 evidence 的專案視為無 Sisyphus 可顯示資料

- **WHEN** 專案的 `.sisyphus/` 僅含 evidence 檔案，無 plans、notepads、drafts 與 active plan
- **THEN** Explorer 不顯示 Sisyphus 來源層級
- **AND** 若該專案亦無 OpenSpec 資料，則顯示無資料的空狀態提示
