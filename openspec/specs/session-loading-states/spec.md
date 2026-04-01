## ADDED Requirements

### Requirement: Session 卡片載入骨架狀態

系統 SHALL 在 session 列表載入期間顯示骨架（skeleton）卡片，避免版面跳動。

#### Scenario: 初始載入中

- **WHEN** 應用程式初始載入尚未完成
- **THEN** session 列表顯示 3–5 個骨架卡片（灰色矩形佔位，不含真實資料）
- **AND** 骨架卡片尺寸與真實 session 卡片相同

#### Scenario: 重新掃描中

- **WHEN** 背景重新掃描 session 目錄
- **THEN** 現有 session 卡片保持顯示，不替換為骨架
- **AND** 可用更新指示器（如頂部進度條）表示正在更新

#### Scenario: 載入完成

- **WHEN** session 資料載入完成
- **THEN** 骨架卡片以動畫過渡方式替換為真實 session 卡片
