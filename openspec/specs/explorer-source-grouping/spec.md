## Purpose

定義 Plans & Specs Explorer 如何依資料來源（Sisyphus 與 OpenSpec）分層呈現節點，確保三種檢視模式的階層語意一致，且各來源可獨立展開或收折，避免兩套工具的文件混雜在同一層。

## Requirements

### Requirement: Explorer 以資料來源分層

系統 SHALL 在 Plans & Specs Explorer 中，將節點依其資料來源歸入 Sisyphus 或 OpenSpec 兩個來源層級，且該分層在 `Tree`、`List`、`Cols` 三種檢視模式中皆須呈現。

#### Scenario: 兩種來源同時存在

- **WHEN** 專案同時具有可顯示的 `.sisyphus/` 與 `openspec/` 資料
- **THEN** Explorer 在任一檢視模式下顯示兩個獨立的來源層級標頭
- **AND** Sisyphus 的群組（Plans、Notepads、Drafts）僅出現在 Sisyphus 來源層級下
- **AND** OpenSpec 的群組（Active Changes、Archived Changes、Specs）僅出現在 OpenSpec 來源層級下

#### Scenario: 僅存在單一來源

- **WHEN** 專案僅具有其中一種來源的資料
- **THEN** Explorer 僅顯示該來源的層級標頭與其底下的群組
- **AND** 不顯示另一來源的空層級

#### Scenario: 來源層級可獨立展開收折

- **WHEN** 使用者點擊任一來源層級標頭
- **THEN** 系統切換該來源層級的展開/折疊狀態
- **AND** 另一來源層級的展開/折疊狀態不受影響
- **AND** 該狀態在當前頁面 session 內維持

#### Scenario: 來源層級預設展開狀態

- **WHEN** Explorer 首次載入某專案的資料
- **THEN** 所有存在的來源層級預設為展開
- **AND** 各來源層級底下的群組維持其自身的預設展開規則

### Requirement: OpenSpec 專屬呈現僅套用於 OpenSpec 項目

系統 SHALL 僅對 OpenSpec change 項目套用 OpenSpec 專屬的動作徽章、可複製 slash command，以及 `proposal`/`design`/`tasks` artifact badge。

#### Scenario: Sisyphus 項目不顯示 OpenSpec 動作徽章

- **WHEN** `Cols` 模式呈現 Sisyphus 來源層級下的 plan 或 notepad 項目
- **THEN** 系統不顯示 `待 propose`、`可 apply`、`進行中 x/y`、`可封存` 任一動作徽章
- **AND** 不提供可複製的 slash command

#### Scenario: Sisyphus 項目不顯示 artifact chip

- **WHEN** `List` 模式呈現 Sisyphus 來源層級下的項目
- **THEN** 系統不顯示 `proposal`、`design`、`tasks` chip
- **AND** 不顯示 `n specs` 計數

#### Scenario: OpenSpec change 項目維持既有徽章行為

- **WHEN** `Cols` 或 `List` 模式呈現 OpenSpec 來源層級下的 change 項目
- **THEN** 系統依既有規則顯示動作徽章、進度 badge 與 artifact chip

### Requirement: 來源層級的狀態同步

系統 SHALL 在使用者於任一模式選取節點時，依該節點實際所屬的來源層級與群組展開對應層級，而非依固定的節點深度推導。

#### Scenario: 選取 Sisyphus 節點時展開其來源層級

- **WHEN** 使用者選取一個位於 Sisyphus 來源層級下的節點
- **THEN** 系統展開 Sisyphus 來源層級與該節點所屬群組
- **AND** 不誤將 OpenSpec 的群組標記為選取狀態

#### Scenario: 選取 OpenSpec artifact 時展開其來源層級

- **WHEN** 使用者選取一個位於 OpenSpec change 下的 artifact 節點
- **THEN** 系統展開 OpenSpec 來源層級、該 change 所屬的狀態群組，並選中該 change
