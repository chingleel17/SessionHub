## ADDED Requirements

### Requirement: Cols 模式點擊 change 自動選取 tasks artifact
在 Plan 介面 Cols 模式中，當使用者點擊左側 master 面板的 change 項目時，系統 SHALL 自動選取並顯示該 change 的 tasks artifact 內容（若該 change 已有 tasks.md）。

#### Scenario: 點擊有 tasks artifact 的 change
- **WHEN** 使用者在 Cols 模式點擊一個已有 tasks.md 的 change 項目
- **THEN** 系統 SHALL 設定 `columnsChangeId` 為該 change 的 id，並自動呼叫 `handleSelect` 選取 tasks 子節點，使右側 detail 面板顯示 tasks.md 的內容

#### Scenario: 點擊沒有 tasks artifact 的 change
- **WHEN** 使用者在 Cols 模式點擊一個尚未建立 tasks.md 的 change 項目
- **THEN** 系統 SHALL 只設定 `columnsChangeId`，不自動選取任何子節點，右側 detail 面板顯示 change 的其他可用 artifact 或空狀態

#### Scenario: 切換到另一個 change
- **WHEN** 使用者在 Cols 模式從 change A 點擊切換到 change B（change B 有 tasks artifact）
- **THEN** 系統 SHALL 同時更新 `columnsChangeId` 為 change B 並自動選取 change B 的 tasks 節點，內容區域顯示 change B 的 tasks.md
