## ADDED Requirements

### Requirement: OpenSpec Change 文件展開閱讀

系統 SHALL 在 PlansSpecsView 的 Changes 清單中，允許使用者展開查看該 change 的各個 md 文件內容（proposal.md、design.md、tasks.md）。

#### Scenario: 展開 change 查看文件列表

- **WHEN** 使用者點擊 Changes 清單中的某個 change 項目
- **THEN** 系統展開顯示該 change 擁有的文件列表（proposal / design / tasks）
- **AND** 標示各文件是否存在（存在的文件可點擊讀取）

#### Scenario: 讀取並顯示文件內容

- **WHEN** 使用者點擊展開的文件項目（如 proposal.md）
- **THEN** 系統呼叫 `read_openspec_file` command 讀取文件內容
- **AND** 以 markdown 渲染方式顯示文件內容於展開面板中

#### Scenario: 文件讀取中顯示載入狀態

- **WHEN** 文件內容正在讀取
- **THEN** 系統顯示載入提示，完成後替換為文件內容

### Requirement: OpenSpec Spec 文件展開閱讀

系統 SHALL 在 PlansSpecsView 的 Specs 清單中，允許使用者展開查看 spec.md 內容。

#### Scenario: 展開 spec 查看內容

- **WHEN** 使用者點擊 Specs 清單中的某個 spec 項目
- **THEN** 系統展開顯示該 spec 的 spec.md 內容（markdown 渲染）

#### Scenario: 展開/折疊切換

- **WHEN** 使用者再次點擊已展開的 spec 或 change 項目
- **THEN** 系統折疊該項目，隱藏文件內容

### Requirement: 文件內容路徑安全驗證

系統 SHALL 在 Rust backend 驗證 `read_openspec_file` 的路徑，防止讀取 openspec 目錄以外的檔案。

#### Scenario: 有效路徑讀取

- **WHEN** 呼叫 `read_openspec_file(path)` 且 path 在專案 openspec 目錄下
- **THEN** 系統回傳檔案的 UTF-8 文字內容

#### Scenario: 路徑穿越攻擊防護

- **WHEN** 呼叫 `read_openspec_file(path)` 且 path 包含 `..` 或嘗試存取 openspec 目錄以外的位置
- **THEN** 系統回傳錯誤，不讀取任何檔案內容
