## ADDED Requirements

### Requirement: Claude session 掃描與列表

系統 SHALL 掃描 `claude_root/projects/` 目錄下的 `.jsonl` 檔案（包含子目錄，例如 subagent sessions），將每個檔案解析為一個 `SessionInfo`（provider=`claude`），並支援 mtime-based 增量更新。`cwd` 直接從 JSONL entry 的頂層 `cwd` 欄位取得（不從目錄名 decode）。

#### Scenario: 首次掃描 Claude sessions

- **WHEN** 使用者啟用 Claude provider 且 `claude_root/projects/` 存在
- **THEN** 系統列舉所有 `<encoded-project-dir>/**/*.jsonl` 檔案（含子目錄）
- **AND** 每個 JSONL 檔案解析為 SessionInfo，`id` 取自 `sessionId` 欄位，`cwd` 取自 entry 的頂層 `cwd` 欄位
- **AND** `session_dir` 設為 JSONL 檔案所在目錄的路徑
- **AND** 結果存入 ScanCache 與 sessions_cache SQLite 表

#### Scenario: 增量掃描（mtime diff）

- **WHEN** 觸發 Claude provider 重新掃描
- **AND** 部分 session 檔案的修改時間未變更
- **THEN** 未變更的 session 直接從 cache 讀取，不重新解析
- **AND** 僅修改時間有變動的 session 重新解析並更新 cache

#### Scenario: 子目錄 session（subagent sessions）

- **WHEN** `claude_root/projects/<project>/` 下有子目錄（如 subagent sessions）
- **THEN** 系統遞迴掃描並將子目錄中的 `.jsonl` 也納入列表
- **AND** `session_dir` 對應其實際所在目錄

#### Scenario: JSONL 解析 session metadata

- **WHEN** 解析一個 JSONL 檔案
- **THEN** 讀取第一個 `type=user` entry 的 `timestamp` 作為 `created_at`
- **AND** 讀取最後一個 entry 的 `timestamp` 作為 `updated_at`
- **AND** 讀取第一個 `type=assistant` entry 的 `message.model` 作為模型資訊
- **AND** 讀取 `cwd` 欄位作為工作目錄（`version`、`gitBranch` 也可存入 metadata）

#### Scenario: JSONL 部分行解析失敗

- **WHEN** JSONL 中有格式不合規的行（type 不明、缺少必要欄位）
- **THEN** 跳過該行，繼續處理其餘行
- **AND** 若整個檔案無法取得任何有效 metadata，設 `parse_error=true`

### Requirement: Claude session 統計解析（含 dedup）

系統 SHALL 從 Claude JSONL 中提取每個 session 的 token 用量（input、output、cache_creation、cache_read），並以 `SessionStats` 格式快取。由於同一 `message.id` 會在 JSONL 中多次出現（sidechain replay），系統 SHALL 以 `message.id` 為 dedup key，每個 message 只計一次。

#### Scenario: 解析並 dedup token 統計

- **WHEN** 系統讀取一個 Claude session JSONL
- **THEN** 只處理 `type=assistant` 且 `isSidechain=false` 的 entry
- **AND** 以 `message.id` 為 key 進行 dedup，同一 id 只取第一筆（或 token 最多的那筆）
- **AND** 累計所有不重複 assistant message 的 `message.usage.input_tokens` 與 `output_tokens`
- **AND** 累計 `cache_creation_input_tokens`（或 `cache_creation.ephemeral_1h_input_tokens + ephemeral_5m_input_tokens`）與 `cache_read_input_tokens`

#### Scenario: cache token 細分統計

- **WHEN** `message.usage.cache_creation` 物件存在
- **THEN** 分別記錄 `ephemeral_1h_input_tokens`（費率 2x）與 `ephemeral_5m_input_tokens`（費率 1x）
- **AND** 此細分資料用於成本估算

#### Scenario: 模型名稱與 service_tier 收集

- **WHEN** 解析 assistant entries
- **THEN** 收集所有不重複的 `message.model` 值（去重後存入 `models_used`）
- **AND** 收集 `message.usage.service_tier`（如 "standard"）並存入統計

#### Scenario: stats cache 失效與重建

- **WHEN** JSONL 檔案的修改時間比 `session_stats` 表中的快取時間戳更新
- **THEN** 系統重新解析並更新 SQLite 快取
- **AND** 未修改的 session 直接使用快取值

### Requirement: Claude session 成本估算

系統 SHALL 根據各型 token 數量與對應的 Claude 模型定價估算每個 session 的美元成本，並存入 `session_stats`；由於 JSONL 不含直接的 `costUSD` 欄位，成本由 SessionHub 自行計算。

#### Scenario: 從 token 計算成本

- **WHEN** 計算 session 成本
- **THEN** 使用內建模型定價表（input / output / cache_creation / cache_read 各別費率）
- **AND** 若 `speed="fast"` 則套用 fast multiplier（約 1.3x）
- **AND** 計算結果以美元儲存於 `session_stats.cost_usd`

#### Scenario: 未知模型的處理

- **WHEN** session 使用的模型不在定價表中
- **THEN** 顯示 token 數量但 cost 欄位標示為 null / 無法估算
- **AND** 不阻擋 stats 快取寫入

### Requirement: Claude provider settings 設定

系統 SHALL 提供 `claude_root` 設定項（預設 `~/.claude`），允許覆寫；Claude provider 預設停用，使用者須手動啟用。

#### Scenario: 使用者設定 claude_root

- **WHEN** 使用者在 Settings 頁輸入自訂 Claude root 路徑
- **THEN** 系統驗證路徑是否存在並儲存設定
- **AND** 下次掃描使用新路徑

#### Scenario: 啟用 Claude provider

- **WHEN** 使用者在 Settings 中啟用 Claude
- **THEN** Claude 出現在 provider filter 列表
- **AND** 下次掃描週期包含 Claude sessions
