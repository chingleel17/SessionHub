## MODIFIED Requirements

### Requirement: 固定 Dashboard 分頁
系統 SHALL 在分頁列最左側顯示固定的 Dashboard 分頁，不可關閉。

#### Scenario: 應用程式啟動
- **WHEN** 應用程式啟動
- **THEN** Dashboard 分頁自動開啟且為當前作用分頁

## ADDED Requirements

### Requirement: Tab header 高度一致
系統 SHALL 確保 Dashboard 分頁與專案分頁的 workspace-header 高度保持一致，切換時不產生高度跳動。

#### Scenario: 從 Dashboard 切換至專案分頁
- **WHEN** 使用者點擊任一專案 tab
- **THEN** workspace-header 高度不改變
- **AND** 專案路徑以縮小字體（不超過 0.75rem）單行截斷顯示於 title 下方

#### Scenario: Dashboard header 顯示副標題
- **WHEN** 使用者切換至 Dashboard 頁面
- **THEN** workspace-header 顯示「首頁」title 與一行固定副標題文字
- **AND** header 高度與專案分頁相同
