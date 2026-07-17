# agents-skills-sync（delta）

## ADDED Requirements

### Requirement: 全域自訂正本位置的 ~/.agents 連結狀態判定

全域範圍設定了自訂正本位置（≠ `~/.agents`）時，系統 SHALL 檢查 `~/.agents` 與正本的連結關係並回報下列狀態之一，於 Skills 子分頁以 banner 呈現；未設定自訂位置時 SHALL NOT 顯示此 banner：

- `linked`：`~/.agents` 整層是指向正本的 symlink；或 `~/.agents` 為實體目錄，且正本第一層每個子項目在 `~/.agents` 均有同名項目、該項目為解析到正本對應路徑的 symlink（逐項等效）
- `partial`：`~/.agents` 為實體目錄，正本第一層子項目僅部分有對應的正本 symlink（其餘缺漏、為實體副本或指向他處）
- `unlinked-physical`：`~/.agents` 為實體目錄，與正本無任何 symlink 對應
- `not-linked`：`~/.agents` 是 symlink 但指向非正本位置
- `missing`：`~/.agents` 不存在

實體目錄情境（`partial`、`unlinked-physical`）SHALL 以資訊性提示呈現，SHALL NOT 使用阻擋性錯誤語氣或要求使用者手動合併後整層連結。

#### Scenario: 整層 symlink 已連結

- **WHEN** `~/.agents` 是指向自訂正本位置的目錄 symlink
- **THEN** 狀態為 `linked`，banner 顯示已連結徽章

#### Scenario: 實體目錄且逐項連結等效

- **WHEN** `~/.agents` 為實體目錄，且正本第一層每個子項目（目錄與檔案）在 `~/.agents` 都有同名的 symlink 且解析到正本對應路徑
- **THEN** 狀態為 `linked`，banner 顯示已連結徽章，不要求整層重建連結

#### Scenario: 實體目錄且部分子項目已連結

- **WHEN** `~/.agents` 為實體目錄，正本第一層子項目僅部分在 `~/.agents` 有對應的正本 symlink
- **THEN** 狀態為 `partial`，banner 以資訊性提示顯示「部分子項目已連結至正本」並列出未對應的項目名稱
- **AND** 不提供整層「建立連結」自動化操作

#### Scenario: 實體目錄且與正本無關聯

- **WHEN** `~/.agents` 為實體目錄，其內容與正本無任何 symlink 對應
- **THEN** 狀態為 `unlinked-physical`，banner 以資訊性提示說明 `~/.agents` 為實體目錄、原生讀取 `.agents` 的工具將讀到實體內容而非正本
- **AND** 不顯示「無法自動連結、請先手動合併」等錯誤文案，不提供整層「建立連結」操作

#### Scenario: 僅於 ~/.agents 不存在時提供建立連結

- **WHEN** `~/.agents` 不存在（狀態 `missing`），使用者點擊「建立連結」
- **THEN** 系統建立 `~/.agents` → 自訂正本位置的目錄 symlink，成功後狀態轉為 `linked`

#### Scenario: 非 missing 狀態呼叫建立連結被拒絕

- **WHEN** 狀態為 `partial`、`unlinked-physical` 或 `not-linked` 時呼叫 `link_agents_root`
- **THEN** 系統不覆蓋、不搬移任何既有內容，回傳明確錯誤訊息

#### Scenario: symlink 指向他處

- **WHEN** `~/.agents` 是 symlink 但解析結果非自訂正本位置
- **THEN** 狀態為 `not-linked`，banner 維持現行提示要求使用者確認連結目標

#### Scenario: symlink 權限不足

- **WHEN** 於 `missing` 狀態建立 `~/.agents` symlink 因權限不足失敗
- **THEN** 系統提示可啟用 Windows 開發者模式或以系統管理員身分執行；不自動改以複製佈署

## REMOVED Requirements

### Requirement: 全域自訂正本位置的 ~/.agents 連結檢查

**Reason**: 原需求將 `~/.agents` 為實體目錄一律視為 conflict 並要求手動合併，誤傷逐項 symlink 與純實體等合法佈局。由「全域自訂正本位置的 ~/.agents 連結狀態判定」取代。
**Migration**: 前端 `AgentsRootLinkStatus` 型別移除 `conflict`，新增 `partial` 與 `unlinked-physical`；`agents.rootLink.conflict.*` 文案 keys 由新狀態對應的資訊性文案取代。
