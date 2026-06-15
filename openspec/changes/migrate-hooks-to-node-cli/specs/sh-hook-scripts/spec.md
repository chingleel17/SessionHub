## MODIFIED Requirements

### Requirement: claude.rs 同時產生 command 與 commandWindows

`managed_hook_group` 產生的 hook JSON SHALL 以 `node <script.js>` 作為主軌命令，並以 sh 腳本作為 fallback；SHALL NOT 再產生 PowerShell / `commandWindows` 命令：
- 主軌命令使用 `.js` 入口腳本（`node "<path>/on-xxx.js" --bridge-path "<path>" --provider claude`）
- fallback 使用 sh 腳本路徑（`sh "<path>/on-xxx.sh" --bridge-path "<path>" --provider claude`）

#### Scenario: 重新安裝後 hook group 主軌為 node

- **WHEN** 呼叫 `install_or_update_claude_integration`
- **THEN** Claude settings.json 中每個 hook group 的主命令指向 `node <script.js>`
- **AND** 不含任何 PowerShell / `commandWindows` 欄位

#### Scenario: hook group 保留 sh fallback

- **WHEN** 呼叫 `install_or_update_claude_integration`
- **THEN** 每個 hook group 仍包含指向 `.sh` 腳本的 fallback 命令欄位

#### Scenario: 重複安裝不產生重複 hook group

- **WHEN** 對同一 Claude settings.json 執行兩次安裝
- **THEN** 每個事件下仍只有一個 SessionHub hook group（冪等）

## REMOVED Requirements

### Requirement: sh 版本 hook 模組結構

**Reason**: sh 由主軌降為 fallback，主邏輯改由 `node-hook-scripts` 的 `record-event.js` 承載；sh 模組行為維持現狀但不再是受規範的主結構，避免與 node 主軌重複定義。

**Migration**: 主軌邏輯參見 `node-hook-scripts` 規格的「共用 record-event.js 模組」需求；既有 `record-event.sh` 等 sh 模組檔案保留作 fallback，無需移除。

### Requirement: sh 版本入口腳本

**Reason**: sh 入口腳本由主軌降為 fallback，主軌入口改由 `node-hook-scripts` 的 `.js` 入口腳本定義。

**Migration**: 主軌入口腳本參見 `node-hook-scripts` 規格；既有 `.sh` 入口腳本保留作 fallback。
