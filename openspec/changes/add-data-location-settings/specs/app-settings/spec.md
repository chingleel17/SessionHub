## ADDED Requirements

### Requirement: 設定頁提供資料位置區塊

設定頁 SHALL 提供「資料位置」區塊入口，呈現各 provider 與 SessionHub 自身資料目錄的現況檢視與搬遷引導（詳細行為見 `data-location-settings` capability）。

#### Scenario: 區塊入口顯示

- **WHEN** 使用者開啟設定頁
- **THEN** 頁面包含「資料位置」區塊，僅列出已啟用 provider 與 SessionHub 自身的項目

### Requirement: 搬遷完成後自動更新 provider root 設定

搬遷引導完成時，系統 SHALL 自動將 AppSettings 中對應的 `claude_root` / `codex_root` / `copilot_root` / `opencode_root` 更新為新目錄並持久化，不需使用者手動填寫。

#### Scenario: root 欄位自動同步

- **WHEN** 某 provider 的搬遷流程成功完成
- **THEN** 對應 root 欄位寫入新路徑並儲存至 settings.json
- **AND** 設定頁的 root 輸入欄位即時反映新值
