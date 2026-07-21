## ADDED Requirements

### Requirement: 收折後 header 不顯示底線

當 Explorer 面板處於收折狀態時，面板 header 的底部邊框 SHALL 被隱藏，且高度 SHALL 自動縮小至剛好容納收折按鈕，不留多餘空白區域。

#### Scenario: 收折狀態 header 無底線

- **WHEN** 使用者點擊收折按鈕，Explorer 面板進入 `explorer-panel--collapsed` 狀態
- **THEN** `.explorer-panel-header` 的 `border-bottom` SHALL 不顯示
- **AND** header 區域高度 SHALL 為 auto（依按鈕大小決定）
- **AND** 面板內容區域 SHALL 完全隱藏

#### Scenario: 展開狀態 header 正常顯示底線

- **WHEN** Explorer 面板處於展開狀態
- **THEN** `.explorer-panel-header` 的 `border-bottom` SHALL 正常顯示
- **AND** header 高度 SHALL 維持 `min-height: 50px`
