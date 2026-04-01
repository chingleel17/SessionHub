## ADDED Requirements

### Requirement: Session 對話時長自動換算單位
系統 SHALL 在 SessionStatsBadge 顯示對話時長時，自動換算為適合閱讀的單位：不足 60 分鐘顯示為分鐘（`Xm`），達 60 分鐘以上換算為小時（`X.Xh` 或 `Xh`）。

#### Scenario: 時長不足 60 分鐘
- **WHEN** session 的 `durationMinutes` 小於 60
- **THEN** 顯示為 `{durationMinutes}m`（例如 `45m`）

#### Scenario: 時長剛好 60 分鐘
- **WHEN** session 的 `durationMinutes` 等於 60
- **THEN** 顯示為 `1h`

#### Scenario: 時長超過 60 分鐘且可整除
- **WHEN** session 的 `durationMinutes` 為 60 的整數倍（例如 120）
- **THEN** 顯示為整數小時（例如 `2h`），不顯示小數點

#### Scenario: 時長超過 60 分鐘且不可整除
- **WHEN** session 的 `durationMinutes` 超過 60 且非整除（例如 90）
- **THEN** 顯示為一位小數小時（例如 `1.5h`）

## ADDED Requirements

### Requirement: Sidebar 收合時顯示版本號
系統 SHALL 在 sidebar 收合狀態下於 footer 顯示縮短版本號，不得完全隱藏。

#### Scenario: Sidebar 展開時
- **WHEN** sidebar 處於展開狀態
- **THEN** footer 顯示完整版本號（例如 `v0.1.0`）

#### Scenario: Sidebar 收合時
- **WHEN** sidebar 處於收合狀態
- **THEN** footer 顯示縮短版本號（例如 `v0.1`）
- **AND** 滑鼠懸停時 tooltip 顯示完整版本號（例如 `v0.1.0`）
