## ADDED Requirements

### Requirement: 跨 provider 對話內容搜尋指令

系統 SHALL 提供 Tauri 指令 `search_session_content`，接收關鍵字與待搜尋的 session 清單，讀取各 session 的逐字稿來源進行關鍵字比對，並回傳命中的 session ID 清單。

#### Scenario: 命中對話內容

- **WHEN** 前端以關鍵字與 session 清單呼叫 `search_session_content`
- **THEN** 指令 SHALL 回傳逐字稿中任一訊息文字包含該關鍵字（不分大小寫）的 session ID 清單

#### Scenario: 逐 provider 擷取

- **WHEN** 指令處理清單中的 session
- **THEN** 系統 SHALL 依 session 的 `provider` 欄位分派至對應的內容擷取實作
- **AND** claude 與 codex SHALL 讀取 session 目錄下的 `.jsonl` 逐字稿
- **AND** copilot SHALL 讀取 `events.jsonl`
- **AND** antigravity SHALL 讀取 `transcript.jsonl`
- **AND** opencode SHALL 查詢其 SQLite 資料庫的訊息內容

#### Scenario: 只比對對話文字

- **WHEN** 系統擷取逐字稿內容
- **THEN** 系統 SHALL 只比對使用者與助理的訊息文字
- **AND** SHALL NOT 將工具呼叫參數、檔案內容快照等非對話欄位納入比對

#### Scenario: 單一 session 讀取失敗

- **WHEN** 某個 session 的逐字稿檔案不存在、無法讀取或解析失敗
- **THEN** 系統 SHALL 略過該 session 並繼續處理其餘 session
- **AND** SHALL NOT 使整個指令回傳錯誤

#### Scenario: 空關鍵字

- **WHEN** 傳入的關鍵字為空字串或僅含空白
- **THEN** 指令 SHALL 直接回傳空清單而不讀取任何檔案

#### Scenario: 對話內容不落地

- **WHEN** 指令讀取任一 session 的逐字稿內容
- **THEN** 系統 SHALL NOT 將該內容寫入 `metadata.db`、寫入任何檔案，或存放於跨呼叫的記憶體快取
- **AND** 內容 SHALL 只在單次指令執行期間存在於記憶體，比對完即釋放
- **AND** 指令回傳值 SHALL 只包含 session ID，不含任何訊息文字

#### Scenario: 逐字稿來源已被 provider 清除

- **WHEN** 某 session 的逐字稿曾存在但已被其 provider 自行清理
- **THEN** 該 session SHALL 不再被內容搜尋命中
- **AND** 系統 SHALL NOT 因為任何殘留的快取或索引而回報命中

### Requirement: 對話內容搜尋 UI 整合

篩選工具列 SHALL 提供「搜尋對話內容」開關，讓使用者將關鍵字比對範圍擴及 session 逐字稿。

#### Scenario: 啟用內容搜尋

- **WHEN** 使用者啟用「搜尋對話內容」開關並輸入關鍵字
- **THEN** 前端 SHALL 於輸入停止後呼叫 `search_session_content`
- **AND** 關鍵字比對 SHALL 為「session ID / summary / notes / tags 命中」**OR**「對話內容命中」，亦即啟用開關只會擴大而非縮小關鍵字的命中範圍
- **AND** 該關鍵字結果 SHALL 再與其餘篩選條件（provider、tag、空對話、日期區間）以 AND 組合

#### Scenario: 只有 summary 命中

- **WHEN** 關鍵字出現在 session 的 summary 但未出現在其對話內容中
- **THEN** 該 session SHALL 仍顯示於列表中，不因啟用內容搜尋而消失

#### Scenario: 輸入節流

- **WHEN** 使用者連續輸入關鍵字
- **THEN** 前端 SHALL 以 debounce 延後呼叫，避免每次按鍵都觸發檔案掃描

#### Scenario: 搜尋進行中

- **WHEN** 內容搜尋指令尚未回傳
- **THEN** 系統 SHALL 顯示搜尋中狀態指示
- **AND** SHALL 保留前一次的結果直到新結果回傳，避免列表閃爍為空

#### Scenario: 停用內容搜尋

- **WHEN** 使用者關閉「搜尋對話內容」開關
- **THEN** 系統 SHALL 立即回到僅比對 session ID、summary、notes、tags 的同步搜尋行為
- **AND** SHALL 捨棄尚未回傳的內容搜尋結果

#### Scenario: 指令執行失敗

- **WHEN** `search_session_content` 回傳錯誤
- **THEN** 前端 SHALL 透過 showToast 顯示錯誤訊息
- **AND** 列表 SHALL 退回為僅套用同步篩選條件的結果
