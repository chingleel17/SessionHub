## ADDED Requirements

### Requirement: 讀取已安裝插件版本

系統 SHALL 能讀取 `~/.config/opencode/plugins/sessionhub-provider-event-bridge.ts` 的第一行 header comment，解析 `integrationVersion` 整數值。

#### Scenario: 插件已安裝且有版本 header

- **WHEN** 呼叫 `get_plugin_status` command
- **THEN** 回傳 `{ status: "up_to_date" | "outdated", installedVersion: N, currentVersion: M }`
- **AND** `installedVersion` 等於插件 header 中的 `integrationVersion`

#### Scenario: 插件檔案不存在

- **WHEN** 呼叫 `get_plugin_status` command 且目標路徑無檔案
- **THEN** 回傳 `{ status: "not_installed", installedVersion: 0, currentVersion: M }`

#### Scenario: Header comment 解析失敗

- **WHEN** 插件檔案存在但第一行不包含有效 JSON 或無 `integrationVersion`
- **THEN** 視 `installedVersion` 為 `0`，回傳 `status: "outdated"`

### Requirement: 安裝/更新插件至 opencode plugins 目錄

系統 SHALL 能將內建的最新版本插件模板寫入 `~/.config/opencode/plugins/sessionhub-provider-event-bridge.ts`，並將模板中的 bridge path 佔位符替換為目前使用者環境的實際路徑。

#### Scenario: 首次安裝插件

- **WHEN** 呼叫 `install_opencode_plugin` command 且目錄存在
- **THEN** 寫入插件檔案，`bridgePath` 與 `BRIDGE_DIR` 替換為 `%APPDATA%\SessionHub\provider-bridge\` 的實際路徑
- **AND** 回傳 `Ok(())`

#### Scenario: plugins 目錄不存在時安裝

- **WHEN** 呼叫 `install_opencode_plugin` 且 `~/.config/opencode/plugins/` 目錄不存在
- **THEN** 系統自動以 `create_dir_all` 建立目錄後再寫入檔案
- **AND** 回傳 `Ok(())`

#### Scenario: 更新已安裝的舊版插件

- **WHEN** 呼叫 `install_opencode_plugin` 且檔案已存在（版本過舊）
- **THEN** 以最新版本覆寫現有檔案
- **AND** 回傳 `Ok(())`

#### Scenario: 無寫入權限

- **WHEN** 呼叫 `install_opencode_plugin` 但目標路徑無寫入權限
- **THEN** 回傳 `Err(String)` 包含清楚的錯誤描述
