## MODIFIED Requirements

### Requirement: OpenSpec Change 文件展開閱讀

系統 SHALL 在 PlansSpecsView 的左側 Explorer 樹中，以子節點形式顯示 change 的各個 md 文件（proposal.md、design.md、tasks.md），點擊子節點後在右側面板顯示內容。

#### Scenario: Change 節點顯示文件子節點

- **WHEN** 使用者展開 Changes 群組中的某個 change 節點
- **THEN** 系統顯示該 change 擁有的文件子節點（proposal / design / tasks）
- **AND** 標示各文件是否存在（不存在的文件節點顯示為停用狀態）

#### Scenario: 讀取並顯示文件內容

- **WHEN** 使用者點擊展開的文件子節點（如 proposal.md）
- **THEN** 系統呼叫 `read_openspec_file` command 讀取文件內容
- **AND** 以純文字方式顯示文件內容於右側面板中
- **AND** 左側節點高亮顯示為已選取狀態

#### Scenario: 文件讀取中顯示載入狀態

- **WHEN** 文件內容正在讀取
- **THEN** 右側面板顯示載入指示器

### Requirement: OpenSpec Spec 文件展開閱讀

系統 SHALL 在 PlansSpecsView 的左側 Explorer 樹中，以葉節點形式顯示各 spec 項目，點擊後在右側面板顯示 spec.md 內容。

#### Scenario: 點擊 Spec 節點查看內容

- **WHEN** 使用者點擊左側 Specs 群組中的某個 spec 節點
- **THEN** 系統在右側面板顯示該 spec 的 spec.md 內容（純文字）
- **AND** 左側節點高亮為已選取狀態

#### Scenario: 展開/折疊切換

- **WHEN** 使用者點擊群組節點（非葉節點）
- **THEN** 系統切換該群組的展開/折疊狀態
- **AND** 切換狀態不清除右側面板的當前文件內容

### Requirement: 文件內容路徑安全驗證

系統 SHALL 在 Rust backend 驗證 `read_openspec_file` 的路徑，防止讀取 openspec 目錄以外的檔案。

#### Scenario: 有效路徑讀取

- **WHEN** 呼叫 `read_openspec_file(path)` 且 path 在專案 openspec 目錄下
- **THEN** 系統回傳檔案的 UTF-8 文字內容

#### Scenario: 路徑穿越攻擊防護

- **WHEN** 呼叫 `read_openspec_file(path)` 且 path 包含 `..` 或嘗試存取 openspec 目錄以外的位置
- **THEN** 系統回傳錯誤，不讀取任何檔案內容
