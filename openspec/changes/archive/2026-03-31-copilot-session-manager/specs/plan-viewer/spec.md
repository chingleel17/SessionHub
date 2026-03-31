## ADDED Requirements

### Requirement: 讀取 plan.md
系統 SHALL 偵測 session 目錄下是否存在 `plan.md`，並在 session 卡片上顯示「有 plan」的視覺標示。

#### Scenario: Session 有 plan.md
- **WHEN** 掃描 session 時發現 `<sessionID>/plan.md` 存在
- **THEN** session 卡片顯示 plan 圖示標記
