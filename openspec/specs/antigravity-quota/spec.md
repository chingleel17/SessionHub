## ADDED Requirements

### Requirement: Antigravity quota via local Language Server

系統 SHALL 透過本機 Antigravity Language Server 的 Connect RPC 取得 quota，不使用雲端 OAuth 流程。

#### Scenario: 探測 Language Server 進程與 port

- **WHEN** 執行 Antigravity quota 更新
- **THEN** 系統找出 `language_server.exe` 進程並取得其在 `127.0.0.1` 監聽的 port（可能多個，逐一嘗試）

#### Scenario: 取得 CSRF token 後呼叫 RPC

- **WHEN** 已取得候選 port
- **THEN** 系統先 GET `http://127.0.0.1:<port>/` 從回應中解析 `csrfToken`，再以 `POST` 帶 `x-codeium-csrf-token` 與 `Origin` 標頭呼叫 `/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary`

#### Scenario: 不使用 OAuth 憑證

- **WHEN** 取得 Antigravity quota
- **THEN** 系統不讀取或刷新 `oauth_creds.json`，認證完全由本機 Language Server 處理

### Requirement: Quota bucket mapping

系統 SHALL 將 `RetrieveUserQuotaSummary` 回傳的模型群組 bucket 映射為 `QuotaWindow`，其中 `utilization = 1 - remainingFraction`、`resets_at = resetTime`。

#### Scenario: 映射 5 小時與每週視窗

- **WHEN** 回傳的群組含 `window` 為 `5h` 與 `weekly` 的 bucket
- **THEN** 系統各產生一個 `QuotaWindow`，`utilization` 由 `remainingFraction` 換算，`resets_at` 取自 `resetTime`

#### Scenario: 保留群組區分

- **WHEN** 回傳含「Gemini Models」與「Claude and GPT models」兩個群組
- **THEN** 系統保留兩個群組各自的 5 小時與每週視窗資料，供不同顯示情境使用

### Requirement: Quota display placement

系統 SHALL 依顯示情境呈現 Antigravity quota：底部狀態列僅顯示 Gemini 群組，Dashboard 顯示 Gemini 與 Claude/GPT 兩群組。

#### Scenario: 狀態列只顯示 Gemini 群組

- **WHEN** 底部狀態列顯示 Antigravity quota
- **THEN** 系統只呈現 Gemini 群組的 5 小時與每週視窗

#### Scenario: Dashboard 顯示兩群組

- **WHEN** Dashboard 顯示 Antigravity quota
- **THEN** 系統呈現 Gemini 群組與 Claude/GPT 群組各自的視窗

### Requirement: Graceful degradation when unavailable

系統 SHALL 在 Antigravity Language Server 不可用時優雅降級，不影響其他 provider 的 quota 與 session 功能。

#### Scenario: IDE 未執行

- **WHEN** 找不到 `language_server.exe` 進程（Antigravity IDE／agy 未執行）
- **THEN** 系統將 Antigravity quota 標記為不可用狀態（附說明），不回報崩潰錯誤，其他 provider 的 quota 正常顯示

#### Scenario: RPC 呼叫失敗

- **WHEN** 找到 port 但 RPC 呼叫失敗或無法解析回應
- **THEN** 系統回傳錯誤狀態的 quota snapshot 並附錯誤訊息，不中斷其他 provider 的更新

### Requirement: Quota adapter registration

系統 SHALL 以實作 `QuotaAdapter` trait 的方式提供 Antigravity quota，並註冊進 `QuotaManager`，受 `quota_enabled_providers` 開關控制。

#### Scenario: 註冊並受開關控制

- **WHEN** Antigravity 在 `quota_enabled_providers` 中啟用且執行 quota 更新
- **THEN** `QuotaManager` 呼叫 Antigravity adapter 的 `fetch_snapshot` 並回傳其結果

#### Scenario: 停用時不呼叫

- **WHEN** Antigravity 不在 `quota_enabled_providers` 中
- **THEN** `QuotaManager` 不呼叫 Antigravity adapter
