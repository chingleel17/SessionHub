## ADDED Requirements

### Requirement: Claude 5 小時用量區間追蹤

Claude Code Pro/Max 訂閱採用**滾動 5 小時用量窗口**（非月份）作為配額機制。系統 SHALL 偵測目前活躍的 5 小時用量區間，顯示該區間內的累計 token 用量，並在達到限制時（解析 JSONL 中的 API 錯誤訊息）顯示重置時間。

#### Scenario: 計算活躍 5 小時區間

- **WHEN** 讀取 Claude session stats
- **THEN** 將所有 assistant messages 依時間排序，以「距上一條訊息超過 5 小時」作為區間分界
- **AND** 目前時間在最後一筆訊息的 5 小時內則標示區間為 `active`
- **AND** 顯示該活躍區間的 input_tokens + output_tokens + cache tokens 累計

#### Scenario: 從錯誤訊息解析重置時間

- **WHEN** JSONL 中出現 `is_api_error_message=true` 的 entry
- **AND** 錯誤內容包含用量限制相關文字（如 "usage limit"）
- **THEN** 嘗試從錯誤訊息中解析重置時間戳（格式：`|<unix_seconds>` 後綴）
- **AND** 顯示「將於 HH:MM 重置」提示

#### Scenario: 無活躍區間時的顯示

- **WHEN** 最後一筆訊息距現在已超過 5 小時
- **THEN** 顯示上一個已結束區間的統計（標示為已結束）
- **AND** 提示「目前無活躍用量窗口」

### Requirement: 跨平台月累計成本追蹤

系統 SHALL 在 `provider_quota` SQLite 資料表中記錄各 provider 當月累計的 token 用量與估算成本（USD），以月份（YYYY-MM）與使用者設定的重置日為計算週期。

#### Scenario: 月累計計算

- **WHEN** session stats 被解析或更新
- **THEN** 以 `(provider, billing_period)` 為 key 更新 `provider_quota` 表
- **AND** `billing_period` 從使用者設定的每月重置日計算（預設為每月 1 日）
- **AND** 累計 input_tokens、output_tokens、cache_creation_tokens、cache_read_tokens、cost_usd

#### Scenario: 自訂重置日

- **WHEN** 使用者在 Settings 為某 provider 設定重置日（1–28）
- **THEN** 該日期後的 sessions 歸入新的帳單週期
- **AND** 系統依此重新計算當前週期的累計值

#### Scenario: 查詢當期用量

- **WHEN** 系統呼叫 `get_provider_quota` command
- **THEN** 回傳各 provider 當前帳單週期的 token 累計與 cost_usd
- **AND** 包含使用者設定的方案上限值（若未設定則為 null）
- **AND** 包含下次重置日期

### Requirement: 訂閱方案上限設定

系統 SHALL 允許使用者為每個 provider 手動設定每月方案 token 上限或費用上限，系統將以進度條形式顯示用量佔比。

#### Scenario: 設定方案上限

- **WHEN** 使用者在 Settings 輸入 provider 的月 token 上限（或費用上限）
- **THEN** 系統儲存該值並更新用量進度條
- **AND** 進度條顯示「已用 / 上限」格式

#### Scenario: 用量超過閾值時警示

- **WHEN** 當期累計用量超過上限的 90%
- **THEN** 進度條顯示橙色警示
- **AND** 用量超過 100% 時顯示紅色
- **AND** 若背景執行功能啟用，觸發 Windows toast 通知

#### Scenario: 未設定上限時的顯示

- **WHEN** 使用者未設定方案上限
- **THEN** 顯示純累計數字與估算成本（不顯示進度條）
- **AND** 提供「設定上限」引導按鈕

### Requirement: StatusBar 三欄用量摘要顯示

系統 SHALL 將現有 StatusBar 重新設計為三欄結構：左欄保留 Bridge 事件通知，中欄顯示 session 狀態數量（active / waiting / idle / done），右欄顯示各啟用 provider 的當期用量進度條。

#### Scenario: StatusBar 三欄顯示

- **WHEN** 應用程式主視窗顯示中
- **THEN** StatusBar 左欄顯示最後一筆 bridge event（provider、eventType、狀態、cwd）
- **AND** 中欄顯示 active / waiting / idle / done 各別計數
- **AND** 右欄顯示各啟用 provider 的本期 token 用量與進度條（有設定上限時）

#### Scenario: 右欄 provider 用量 hover 詳情

- **WHEN** 使用者 hover 右欄某 provider 的用量指示
- **THEN** 顯示 tooltip：input tokens、output tokens、cache tokens、估算成本分開列示
- **AND** 顯示「此為 SessionHub 記錄的用量，非實際帳號用量」說明

### Requirement: Dashboard 依啟用清單過濾 quota 顯示

系統 SHALL 讓 Dashboard 的 Quota 卡片（`QuotaOverview`）只顯示使用者於設定頁 `quotaEnabledProviders` 中勾選啟用的 provider，行為 SHALL 與 StatusBar 的 quota chip 顯示範圍一致。

#### Scenario: 使用者停用某 provider 的 quota 監控

- **WHEN** 使用者在設定頁取消勾選某個 provider（例如 OpenCode）的 quota 監控並儲存
- **THEN** Dashboard 的 Quota 卡片 SHALL 不再顯示該 provider 的用量資訊
- **AND** 即使後端快取或資料庫中仍留有該 provider 過去的 snapshot 資料

#### Scenario: 重新開啟設定頁或 Dashboard 後仍保持過濾

- **WHEN** 使用者停用某 provider 的 quota 監控後，重新導覽至設定頁或 Dashboard
- **THEN** 該 provider SHALL 不會重新出現在 Dashboard 的 Quota 卡片中
- **AND** 直到使用者重新勾選啟用該 provider 為止

#### Scenario: 使用者重新啟用某 provider 的 quota 監控

- **WHEN** 使用者重新勾選啟用某 provider 的 quota 監控並儲存
- **THEN** Dashboard 的 Quota 卡片 SHALL 在下次刷新後顯示該 provider 的用量資訊

### Requirement: 儲存設定時清除已停用 provider 的 quota 快取

系統 SHALL 在使用者儲存設定時，依最新的 `quotaEnabledProviders` 清單清除記憶體快取與資料庫中已停用 provider 的 quota snapshot，而不僅限於使用者手動觸發全量「重新整理」時才清除。

#### Scenario: 儲存設定觸發快取清除

- **WHEN** 使用者將某 provider 從 `quotaEnabledProviders` 移除並儲存設定
- **THEN** 後端 SHALL 立即從記憶體快取與資料庫移除該 provider 的既有 quota snapshot
- **AND** 後續 `get_quota_snapshots` 呼叫不再回傳該 provider 的資料

#### Scenario: 應用程式啟動時載入快取遵循目前啟用清單

- **WHEN** 應用程式啟動並從資料庫載入既有 quota snapshot 至記憶體快取
- **THEN** 系統 SHALL 排除目前 `quotaEnabledProviders` 中未啟用的 provider 資料

### Requirement: 背景執行時持續追蹤用量

當應用程式最小化至系統匣後，系統 SHALL 持續接收 bridge 事件並更新用量統計，確保 hook 觸發的 session 更新不因視窗隱藏而遺漏。

#### Scenario: 視窗最小化後 bridge 事件仍正常處理

- **WHEN** 應用程式最小化至系統匣
- **AND** Claude Code stop hook 觸發寫入 bridge 檔案
- **THEN** SessionHub 後端仍偵測到 bridge 事件
- **AND** 對應 session stats 更新並累加至 provider_quota 表

#### Scenario: 用量超限時背景通知

- **WHEN** 應用程式在背景執行
- **AND** 當期累計用量超過使用者設定上限的 90%
- **THEN** 系統送出 Windows toast 通知（整合現有 intervention-notification 機制）
