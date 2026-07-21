## Purpose

定義 Cols 模式左側 change 列的狀態提示與 slash command 複製互動，讓使用者能直接從目前進度執行下一步 OpenSpec 操作。

## Requirements

### Requirement: Cols 模式顯示 change 狀態與對應 slash command

Cols 模式左側每個 change 列 SHALL 在進度條下方顯示一個「action 列」，內含狀態標籤與複製指令按鈕，依 change 當前進度自動決定對應的 slash command。

#### Scenario: 未 propose 狀態顯示 propose 指令

- **WHEN** change 的 `proposal` artifact 不存在（無子節點或 tone 為 `not_started`）
- **THEN** action 列 SHALL 顯示「待 propose」標籤
- **AND** 對應指令 SHALL 為 `/opsx:propose <change-name>`

#### Scenario: 有 proposal 但無 tasks 顯示 apply 指令

- **WHEN** `proposal` artifact 存在，但 `tasks` artifact 不存在或 progress 為 null
- **THEN** action 列 SHALL 顯示「可 apply」標籤
- **AND** 對應指令 SHALL 為 `/opsx:apply <change-name>`

#### Scenario: tasks 進行中顯示進度與 apply 指令

- **WHEN** `progress.done < progress.total`（且 total > 0）
- **THEN** action 列 SHALL 顯示「進行中 X/Y」標籤（X=done, Y=total）
- **AND** 對應指令 SHALL 為 `/opsx:apply <change-name>`

#### Scenario: tasks 全部完成顯示 archive 指令

- **WHEN** `progress.done === progress.total`（且 total > 0）
- **THEN** action 列 SHALL 顯示「可封存」標籤
- **AND** 對應指令 SHALL 為 `/opsx:archive <change-name>`

### Requirement: 一鍵複製 slash command

使用者 SHALL 能透過點擊 action 列的複製按鈕將對應 slash command 複製至剪貼簿。

#### Scenario: 點擊複製按鈕後成功複製

- **WHEN** 使用者點擊 action 列中的複製按鈕
- **THEN** 系統 SHALL 呼叫 `navigator.clipboard.writeText()` 寫入對應 slash command
- **AND** 按鈕圖示 SHALL 短暫切換為「✓」（持續 500ms）後恢復原狀

#### Scenario: 複製按鈕 hover 才顯示

- **WHEN** 使用者未 hover 該 change 列
- **THEN** 複製按鈕 SHALL 隱藏（opacity: 0）
- **WHEN** 使用者 hover 該 change 列
- **THEN** 複製按鈕 SHALL 顯示（opacity: 1）

#### Scenario: 剪貼簿 API 不可用時靜默失敗

- **WHEN** `navigator.clipboard.writeText()` 拋出例外
- **THEN** 系統 SHALL 靜默忽略錯誤，不顯示 toast 或錯誤訊息
- **AND** 按鈕不 SHALL 切換為「✓」圖示
