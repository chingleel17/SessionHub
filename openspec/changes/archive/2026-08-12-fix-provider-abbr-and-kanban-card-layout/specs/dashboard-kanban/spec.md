## MODIFIED Requirements

### Requirement: Kanban 視圖跨專案顯示

系統 SHALL 在 Kanban 看板中展示所有專案的 sessions，以**專案為單位**分組顯示。ProjectCard 標頭在容器寬度不足以完整顯示所有內容時，SHALL 依固定優先序犧牲次要資訊，確保專案名稱恆可辨識，且標頭內容不得超出卡片容器寬度。

#### Scenario: 跨專案 session 顯示（ProjectCard 分組）

- **WHEN** 使用者瀏覽 Kanban 視圖
- **THEN** 同一欄中的 sessions 依所屬專案分組，每個專案顯示為一張 `ProjectCard`
- **AND** 每張 ProjectCard 標頭顯示：專案名稱、分支名稱（如有）、該欄中的 session 數量、provider 標籤、最後更新時間
- **AND** ProjectCard 預設為展開狀態

#### Scenario: ProjectCard 收折

- **WHEN** 使用者點擊 ProjectCard 的收折按鈕
- **THEN** 隱藏 session 列表，僅顯示標頭摘要（session 數量保留可見）
- **AND** 再次點擊時展開，恢復顯示

#### Scenario: ProjectCard 展開後 session 列表

- **WHEN** ProjectCard 處於展開狀態
- **THEN** 以輕量列表行顯示每個 session：summary（截至 60 字元）、activity badge、啟動按鈕
- **AND** Active 狀態的 session 額外顯示活動細節（Thinking / Tool Call / File Op / Sub-Agent / Working）

#### Scenario: 專案名稱保有最小可讀寬度

- **WHEN** ProjectCard 標頭因分支名稱過長或 provider 標籤數量過多導致空間不足
- **THEN** 專案名稱維持最小可讀寬度，不得被壓縮至不可見或消失
- **AND** 專案名稱、分支名稱皆以 `title` 屬性提供完整文字，供使用者 hover 查看

#### Scenario: 分支名稱優先截斷

- **WHEN** 標頭空間不足以同時完整顯示專案名稱與分支名稱
- **THEN** 分支名稱優先被壓縮並以省略號（ellipsis）截斷，讓出空間給專案名稱
- **AND** session 數量、最後更新時間、展開箭頭等固定元素維持原有寬度不縮放

#### Scenario: Provider 標籤數量過多時以摘要呈現

- **WHEN** 同一專案的 sessions 使用超過顯示上限（2 個）的不同 provider
- **THEN** 標頭僅顯示前 2 個 provider 標籤，其餘以 `+N` 摘要呈現
- **AND** `+N` 摘要以 `title` 屬性列出其餘 provider 的完整名稱
- **AND** provider 標籤不得因空間不足而被無聲裁切（隱藏但無提示）

#### Scenario: 標頭內容不得溢出容器

- **WHEN** ProjectCard 標頭在任意容器寬度下渲染
- **THEN** 標頭所有子元素的總寬度不得超出卡片容器寬度（不產生水平捲動或視覺溢出）
