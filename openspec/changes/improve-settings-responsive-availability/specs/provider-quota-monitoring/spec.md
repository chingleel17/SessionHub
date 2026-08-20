## ADDED Requirements

### Requirement: quota provider 使用全域顯示順序

Quota 設定、Overview、Overlay、Tray 與狀態列 SHALL 依 Claude Code、OpenCode、Codex、GitHub Copilot CLI、Antigravity 的順序顯示 provider，並略過未啟用或沒有資料的項目。

#### Scenario: 顯示部分 quota provider

- **WHEN** 只有 OpenCode、Codex 與 GitHub Copilot CLI 啟用或具有 quota 資料
- **THEN** 顯示順序為 OpenCode、Codex、GitHub Copilot CLI

### Requirement: quota provider 選項反映資料根目錄可用性

設定頁的 quota provider 選項 SHALL 依各 provider 資料根目錄偵測結果決定是否可操作，未偵測的 provider SHALL 保留可見但不得選取。

#### Scenario: provider 資料根目錄已偵測

- **WHEN** 某 quota provider 的資料根目錄存在
- **THEN** 該 provider 選項以正常樣式呈現並可切換

#### Scenario: provider 資料根目錄未偵測

- **WHEN** 某 quota provider 的資料根目錄不存在
- **THEN** 該 provider 選項以低對比不可用樣式呈現
- **AND** checkbox 停用且不接受選取變更

#### Scenario: 已選 quota provider 暫時未偵測

- **WHEN** 已儲存於 quota provider 清單的 provider 根目錄暫時不存在
- **THEN** 設定頁保留其已選狀態但停用控制項
- **AND** 系統不得只因本次偵測失敗自動刪除該設定值
