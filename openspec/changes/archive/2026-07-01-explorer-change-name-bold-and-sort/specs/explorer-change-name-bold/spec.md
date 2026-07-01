## ADDED Requirements

### Requirement: List 與 Cols 模式 change 名稱加粗
在 Explorer 的 List 和 Cols 模式中，change 名稱 SHALL 以 `font-weight: 600` 顯示，使每個項目視覺上更清晰，與 Tree 模式群組標籤的視覺權重保持一致。

#### Scenario: List 模式 change 名稱加粗
- **WHEN** Explorer 處於 List 模式並顯示 change 清單
- **THEN** 每個 change 的名稱按鈕（`.explorer-list-row-name`）SHALL 以粗體（font-weight: 600）渲染

#### Scenario: Cols 模式 change 名稱加粗
- **WHEN** Explorer 處於 Cols 模式並在左側 master 面板顯示 change 清單
- **THEN** 每個 change 的名稱文字（`.explorer-cols-entry-name`）SHALL 以粗體（font-weight: 600）渲染
