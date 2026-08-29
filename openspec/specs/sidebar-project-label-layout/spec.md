## Purpose

讓 Sidebar 在固定寬度內穩定呈現專案名稱與 Git 分支，避免長專案名稱把分支資訊擠到不可辨識，同時保留完整標籤的存取方式。

## Requirements

### Requirement: 專案與分支使用明確的寬度優先序

Sidebar 的專案項目 SHALL 優先為分支區保留至少足以完整顯示 `master` 的寬度，剩餘空間由專案名稱使用；空間不足時 SHALL 優先截斷專案名稱，不得讓短分支被壓縮成不可辨識的片段。

#### Scenario: 長專案名稱搭配 master 分支

- **WHEN** 專案名稱與 `master` 分支無法在 Sidebar 可用寬度內完整並列
- **THEN** 專案名稱 SHALL 以省略號截斷
- **AND** `master` SHALL 完整顯示
- **AND** 專案與分支之間的分隔符 SHALL 保持可見

#### Scenario: 短專案名稱搭配短分支

- **WHEN** 專案名稱與分支名稱可在可用寬度內完整並列
- **THEN** 兩者 SHALL 完整顯示且不產生不必要截斷

#### Scenario: 專案沒有分支資訊

- **WHEN** 專案項目沒有分支名稱
- **THEN** 專案名稱 SHALL 可使用分支原本會占用的剩餘寬度
- **AND** 不顯示空白分支欄位或分隔符

### Requirement: 長分支名稱在保留區內截斷

當分支名稱長於可分配寬度時，Sidebar SHALL 將分支維持單列並以省略號截斷，同時保留至少 `master` 寬度的可辨識前綴區域。

#### Scenario: feature 分支超過可用寬度

- **WHEN** 分支名稱例如 `feature/improve-transcription` 超過分支可用寬度
- **THEN** 分支名稱 SHALL 在單列內顯示可辨識前綴並以省略號結尾
- **AND** 不得擠出 Sidebar 邊界或覆蓋其他操作

#### Scenario: 查看完整截斷內容

- **WHEN** 專案名稱或分支名稱因寬度限制遭到截斷
- **THEN** 使用者 SHALL 可透過項目既有 tooltip 取得完整專案名稱與完整分支名稱

### Requirement: 標籤寬度規則套用所有專案導覽項目

相同的專案／分支寬度分配 SHALL 套用於釘選專案與已開啟未釘選專案，避免同一專案因所在區段不同而出現不一致截斷。

#### Scenario: 專案出現在不同 Sidebar 區段

- **WHEN** 相同長名稱專案先後顯示為釘選項目或已開啟未釘選項目
- **THEN** 兩個區段 SHALL 使用相同的專案優先截斷與分支最小保留寬度規則
