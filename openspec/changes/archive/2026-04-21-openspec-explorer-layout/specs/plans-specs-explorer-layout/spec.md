## ADDED Requirements

### Requirement: 雙面板 Explorer 佈局

系統 SHALL 將 PlansSpecsView 渲染為左右雙面板佈局：左側為 Explorer 樹狀導覽面板，右側為內容檢視面板。

#### Scenario: 雙面板同時顯示

- **WHEN** PlansSpecsView 有可顯示的資料（Sisyphus 或 OpenSpec 資料存在）
- **THEN** 畫面分為左側 Explorer 面板與右側內容面板
- **AND** 兩個面板均可獨立捲動

#### Scenario: 右側面板初始空白狀態

- **WHEN** 使用者尚未選取任何節點
- **THEN** 右側面板顯示提示文字（如「請從左側選取文件」）

### Requirement: 左側 Explorer 樹狀導覽

系統 SHALL 在左側面板以多層樹狀結構顯示所有可瀏覽的文件節點，支援展開與折疊。

#### Scenario: 根節點顯示

- **WHEN** Sisyphus 資料與 OpenSpec 資料均存在
- **THEN** 左側顯示「Sisyphus」與「OpenSpec」兩個根節點
- **AND** 每個根節點可獨立展開或折疊

#### Scenario: 群組節點展開

- **WHEN** 使用者點擊群組節點（如「Active Changes」或「Specs」）
- **THEN** 系統切換該節點的展開/折疊狀態
- **AND** 展開時顯示子節點列表

#### Scenario: 葉節點選取

- **WHEN** 使用者點擊葉節點（如 `proposal.md`、`spec.md`）
- **THEN** 系統在右側面板載入並顯示對應文件內容
- **AND** 左側高亮顯示已選取的節點

### Requirement: 左側面板寬度可調整

系統 SHALL 允許使用者透過拖曳分隔線調整左側 Explorer 面板的寬度。

#### Scenario: 拖曳調整寬度

- **WHEN** 使用者按住分隔線並拖曳
- **THEN** 左側面板寬度隨滑鼠移動即時調整
- **AND** 左側最小寬度為 160px，右側最小寬度為 200px

#### Scenario: 折疊左側面板

- **WHEN** 使用者點擊折疊切換按鈕
- **THEN** 左側面板縮小至最小寬度（≤40px）
- **AND** 右側面板佔用釋放的空間

### Requirement: 右側內容檢視面板

系統 SHALL 在右側面板以純文字方式顯示選取文件的完整內容。

#### Scenario: 顯示選取文件內容

- **WHEN** 使用者在左側選取葉節點
- **THEN** 右側面板顯示該文件的完整文字內容
- **AND** 文件路徑顯示於面板頂部作為標題列

#### Scenario: 文件載入中狀態

- **WHEN** 文件內容正在讀取
- **THEN** 右側面板顯示載入指示器

#### Scenario: 文件讀取錯誤顯示

- **WHEN** 文件讀取失敗
- **THEN** 右側面板以醒目的錯誤樣式（紅色 banner）顯示錯誤訊息
- **AND** 錯誤訊息不出現在左側樹狀清單中

### Requirement: Explorer 佈局高度與標題列對齊

系統 SHALL 確保 Explorer 佈局的高度受限於視窗可視區域，且兩側面板的標題列高度完全一致。

#### Scenario: Explorer 整體高度限制

- **WHEN** PlansSpecsView 在任意視窗大小下渲染
- **THEN** `.explorer-layout` 的高度使用 `calc(100vh - 215px)` 而非 `height: 100%`
- **AND** 最小高度為 420px（防止視窗過小時版面破裂）
- **AND** 左右兩側面板各自獨立捲動，不引發頁面級捲動

#### Scenario: 兩側標題列等高對齊

- **WHEN** Explorer 面板與內容面板同時顯示
- **THEN** `.explorer-panel-header` 與 `.explorer-content-header` 均為固定高度 36px（`box-sizing: border-box`）
- **AND** 兩者使用 `display: flex; align-items: center` 確保內容垂直置中
- **AND** 視覺上形成跨面板的連續水平標題列
