## Purpose

定義 Claude Code 工具授權請求的 hook 原生通知、Notification 相容備援，以及 PermissionRequest hook 的整合生命週期。

## Requirements

### Requirement: Claude 工具授權請求觸發原生介入通知

SessionHub SHALL 以 Claude Code `PermissionRequest` hook 作為工具授權請求的主要訊號，在 Bash、檔案讀寫、跨專案／跨目錄存取或其他工具真正要求使用者授權時，直接發送 hook 原生介入通知。

#### Scenario: Bash 指令要求授權

- **WHEN** Claude Code 對 Bash 工具觸發 `PermissionRequest`
- **THEN** SessionHub hook 發送「需要您授權」類 Windows Toast
- **AND** 通知以 payload 的 `session_id` 作為 session 識別

#### Scenario: 檔案與跨目錄存取要求授權

- **WHEN** Claude Code 對 Read、Edit、Write 或其他檔案工具觸發 `PermissionRequest`
- **AND** 原因為一般檔案權限或跨專案／跨目錄存取
- **THEN** SessionHub hook 同樣發送「需要您授權」類 Windows Toast
- **AND** 不以工具名稱或路徑種類白名單排除事件

#### Scenario: 工具不需授權時不得誤報

- **WHEN** Claude Code 觸發 `PreToolUse`，但沒有觸發 `PermissionRequest`
- **THEN** 系統僅記錄既有 `tool.pre` 活動事件
- **AND** 不因 `PreToolUse` 本身發送「需要您授權」通知

#### Scenario: 授權通知不洩漏工具輸入

- **WHEN** `PermissionRequest` payload 的 `tool_input` 包含 Bash 指令、完整檔案路徑或其他敏感參數
- **THEN** bridge record、log 與 Toast 只使用 session id 與工具名稱等必要欄位
- **AND** 不寫入或顯示原始 `tool_input`、命令內容或檔案內容

### Requirement: Claude Notification 權限提示相容備援

SessionHub SHALL 從 Claude Code `Notification` hook payload 的 `notification_type` 判斷通知語意，並將 `permission_prompt` 作為舊版或替代事件路徑的授權通知備援。

#### Scenario: permission_prompt 使用正確欄位

- **WHEN** `Notification` payload 的 `notification_type` 為 `permission_prompt`
- **THEN** hook 發送「需要您授權」類 Windows Toast
- **AND** 判斷不得依賴不存在於 payload 契約的 `matcher` 欄位

#### Scenario: idle_prompt 維持等待回應通知

- **WHEN** `Notification` payload 的 `notification_type` 為 `idle_prompt`
- **THEN** hook 發送「等待您回應」類 Windows Toast

#### Scenario: 主路徑與備援同時觸發

- **WHEN** 同一 session 的單次授權先後觸發 `PermissionRequest` 與 `Notification.permission_prompt`
- **THEN** 兩條路徑使用相同的 `sessionhub-{session_id}` 通知識別
- **AND** Windows 通知取代既有通知，不堆疊為兩則

### Requirement: Claude PermissionRequest 整合生命週期

SessionHub SHALL 將 `PermissionRequest` 納入 Claude integration 的安裝、完整性偵測、更新與移除流程，並保留使用者既有的其他 hook 設定。

#### Scenario: 更新舊 Claude integration

- **WHEN** 已安裝的 Claude integration 缺少 SessionHub `PermissionRequest` hook 或使用舊版通知腳本
- **THEN** 整合狀態顯示為 outdated
- **AND** 使用者執行更新後，`PermissionRequest` 與修正後的 `Notification` hook 均已安裝

#### Scenario: 合併與移除不影響使用者 hook

- **WHEN** SessionHub 安裝、更新或移除 Claude integration
- **THEN** 系統只新增、替換或移除帶 SessionHub marker 的 managed hook group
- **AND** 使用者自行設定的 `PermissionRequest`、`Notification` 或其他 hooks 保持不變
