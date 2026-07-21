## ADDED Requirements

### Requirement: Antigravity session scanning

系統 SHALL 掃描 `~/.gemini/antigravity/brain`、`~/.gemini/antigravity-cli/brain`、`~/.gemini/antigravity-ide/brain` 三個目錄下的 conversation 子目錄，並將**含 `.system_generated/logs/transcript.jsonl` 的** conversation 映射為一筆 `provider` 為 `"antigravity"` 的 `SessionInfo`。僅有產出物（圖片、markdown）而無 transcript 的目錄不視為 session。

#### Scenario: 掃描三個 brain 目錄

- **WHEN** Antigravity provider 已在 `enabled_providers` 中啟用且執行 session 掃描
- **THEN** 系統走訪三個 brain 目錄中所有 conversation 子目錄（以 conversation UUID 命名）

#### Scenario: 以 transcript 存在性界定 session

- **WHEN** 某 conversation 目錄含 `.system_generated/logs/transcript.jsonl`
- **THEN** 系統為其產生一筆 session

#### Scenario: 略過無對話內容的目錄

- **WHEN** 某 conversation 目錄僅含圖片或產出檔案而無 `transcript.jsonl`
- **THEN** 系統不為其產生 session

#### Scenario: brain 目錄不存在

- **WHEN** 其中一個或多個 brain 目錄不存在（例如使用者未安裝 CLI）
- **THEN** 系統略過不存在的目錄，不回報錯誤，仍回傳其他目錄掃描到的 session

#### Scenario: brain 為雲端符號連結

- **WHEN** brain 目錄是指向雲端硬碟的符號連結
- **THEN** 系統沿符號連結讀取實際內容，與一般目錄行為一致

### Requirement: Session title and workspace resolution (IDE)

對 IDE 形態（`antigravity`、`antigravity-ide`）的 session，系統 SHALL 以 `agyhub_summaries_proto.pb` 作為標題與 workspace 的主要來源，並在查無資料時以 transcript 首則 USER_REQUEST 前段文字作為標題 fallback。

#### Scenario: summaries.pb 提供標題與 workspace

- **WHEN** `agyhub_summaries_proto.pb` 含該 conversation UUID 對應的標題與 `file:///` workspace 路徑
- **THEN** 系統以該標題作為 session 摘要，並將 URL-decode 後的 workspace 路徑作為 `cwd`

#### Scenario: 標題 fallback 為首則 user request

- **WHEN** summaries.pb 查無該 conversation 的標題
- **THEN** 系統以 transcript 首則 `USER_INPUT` 的 `<USER_REQUEST>` 內文前段作為 session 摘要

#### Scenario: workspace 無法取得時留空

- **WHEN** summaries.pb 查無該 conversation 的 workspace 路徑
- **THEN** 系統將 `cwd` 留空，該 session 不歸屬任何 git 專案分組（歸入未分類），不中斷掃描

#### Scenario: 時間取自 transcript 或目錄 mtime

- **WHEN** 產生 session 需填入 `created_at` / `updated_at`
- **THEN** 系統取自 transcript 首/末則的 `created_at`，若不可得則以目錄 mtime 作為 fallback

### Requirement: Session title and workspace resolution (CLI)

對 CLI 形態（`antigravity-cli`，無 `agyhub_summaries_proto.pb`）的 session，系統 SHALL 以 transcript 內容推導標題，並以可得的來源推導 workspace，查無時留空。

#### Scenario: CLI 標題取自首則 request

- **WHEN** 掃描 `antigravity-cli` 下含 transcript 的 conversation
- **THEN** 系統以 transcript 首則 `USER_REQUEST` 內文前段作為 session 摘要

#### Scenario: CLI workspace 查無時留空

- **WHEN** 無法自 CLI session 可得來源推導 workspace
- **THEN** 系統將 `cwd` 留空並歸入未分類，不中斷掃描

### Requirement: Metadata-only parsing scope

系統 SHALL 僅解析 Antigravity session 的 metadata 層級，不深入解析 protobuf 格式的 `conversations/<id>.db` 或 `.pb` 檔案內容。

#### Scenario: 不解析 protobuf 對話內文

- **WHEN** 掃描 Antigravity session
- **THEN** 系統只讀取可讀的 JSONL 與 metadata.json，不嘗試解析 `conversations/` 下的 SQLite/protobuf blob 對話內文

### Requirement: Antigravity provider integration

系統 SHALL 將 Antigravity 納入既有 provider 架構，包含掃描迴圈、provider 開關與前端顯示標籤。

#### Scenario: provider 顯示標籤

- **WHEN** 前端需顯示 `antigravity` provider 的名稱
- **THEN** 系統顯示 `"Antigravity"`

#### Scenario: provider 開關控制掃描

- **WHEN** 使用者在設定中停用 Antigravity provider
- **THEN** 系統的 session 掃描不包含 Antigravity 目錄，列表中不出現 Antigravity session

#### Scenario: 與其他 provider session 並列排序

- **WHEN** Antigravity session 與其他 provider 的 session 一同回傳
- **THEN** 系統依 `updated_at` 由新到舊統一排序，Antigravity session 與其他 provider 無差別參與排序
