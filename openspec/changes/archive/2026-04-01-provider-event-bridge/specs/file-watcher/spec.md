## MODIFIED Requirements

### Requirement: 目錄變更即時偵測
系統 SHALL 以 provider bridge 事件作為 session 更新的主要來源，並保留 OS filesystem watch 作為 fallback。fallback watcher MUST 僅監看關鍵路徑與檔案，且在通知前端前先完成事件過濾與 cheap verify。

#### Scenario: provider bridge 可用時的即時更新
- **WHEN** Copilot hook 或 OpenCode plugin 發送標準化 bridge 事件
- **THEN** 系統以該 bridge 事件觸發 session refresh
- **AND** 不以一般 filesystem event 作為主要刷新依據

#### Scenario: bridge 不可用時使用 fallback watcher
- **WHEN** provider integration 尚未安裝、失效或需要手動設定
- **THEN** 系統啟用 fallback watcher 維持基本即時更新
- **AND** 只有在 cheap verify 判定 session 清單可能變更時才通知前端

#### Scenario: 無關檔案事件被忽略
- **WHEN** watcher 收到與 session 清單無關的檔案事件
- **THEN** 系統忽略該事件
- **AND** 不更新最近活動或同步狀態
