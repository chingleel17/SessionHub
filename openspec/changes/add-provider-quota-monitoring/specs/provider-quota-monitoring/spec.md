## ADDED Requirements

### Requirement: 系統提供統一的 provider quota snapshot

系統 SHALL 以統一的 quota snapshot 模型表示各 quota provider 的可用額度資訊，至少包含 provider、狀態、資料來源、最後更新時間，以及在可取得時的使用量、剩餘量與 reset 時間。

#### Scenario: 成功取得 quota snapshot
- **WHEN** 後端 quota manager 成功向某個 provider adapter 取得資料
- **THEN** 系統回傳標準化 quota snapshot
- **AND** 前端不需要理解 provider-specific 原始格式

#### Scenario: provider 尚未支援
- **WHEN** 使用者啟用了某個平台，但 SessionHub 尚未支援其 quota provider
- **THEN** 系統仍回傳對應 provider 的 snapshot
- **AND** 狀態標示為 `unsupported` 或等效不可用狀態

### Requirement: quota monitoring 以內建 adapter 為主，保留插件式 connector 擴充點

系統 SHALL 支援內建 quota adapters，並保留以外部 connector 擴充新 quota provider 的能力。

#### Scenario: 使用內建 adapter
- **WHEN** 某個 provider 由 SessionHub 內建支援
- **THEN** quota manager 直接呼叫內建 adapter 取得 quota snapshot

#### Scenario: 使用外部 connector
- **WHEN** 某個 provider 未內建但已配置外部 connector
- **THEN** quota manager 透過 connector 取得 quota snapshot
- **AND** 前端仍收到相同的標準化輸出格式

### Requirement: quota 資料支援背景 refresh 與手動 refresh

系統 SHALL 支援應用啟動時刷新、背景輪詢刷新與使用者手動刷新 quota 資料。

#### Scenario: 應用啟動時刷新
- **WHEN** SessionHub 啟動且 quota monitoring 已啟用
- **THEN** 系統自動執行一次 quota refresh

#### Scenario: 使用者手動刷新
- **WHEN** 使用者在 Dashboard 或 Settings 點擊 quota refresh
- **THEN** 系統重新查詢可用的 quota providers
- **AND** 更新最新 snapshot 與最後刷新時間

### Requirement: bridge 事件可觸發節流 quota refresh

系統 SHALL 能在收到 provider bridge 事件後，以節流方式觸發 quota refresh。

#### Scenario: provider bridge 事件後刷新
- **WHEN** 系統收到新的 provider bridge event
- **THEN** 系統可排程 quota refresh
- **AND** 在短時間內重複事件不得造成無限制重複查詢

### Requirement: quota 查詢失敗時保留診斷資訊

系統 SHALL 在 quota 查詢失敗時保留錯誤狀態、最後成功資料或失敗訊息，供 UI 顯示診斷。

#### Scenario: quota source 查詢失敗
- **WHEN** 某個 provider quota source 因 auth、網路或格式錯誤而失敗
- **THEN** 系統回傳包含錯誤狀態與訊息的 snapshot
- **AND** 不得阻斷其他 provider 的 quota 結果
