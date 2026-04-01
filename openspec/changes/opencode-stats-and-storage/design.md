## Context

應用程式目前透過 `open_opencode_db_readonly()` 直接讀取 OpenCode 的 SQLite 資料庫（`~/.local/share/opencode/opencode.db`）取得 session 清單。然而 OpenCode 的訊息、token 用量與工具執行記錄並**不儲存在 SQLite 中**，而是以 JSON 檔案系統的形式存放於 `~/.local/share/opencode/storage/` 目錄：

```
storage/
├── session/    {projectID}/{ses_*.json}     ← session metadata
├── message/    {sessionID}/{msg_*.json}     ← 訊息（含 token 統計）
├── part/       {messageID}/{prt_*.json}     ← 訊息內容塊（text/tool/step）
└── project/    {proj_*.json}               ← 專案 metadata
```

token 資料實際位置：`message/*.json` 的 `tokens` 欄位（含 input/output/reasoning/cache）以及 `part/step-finish` 的 `tokens` 欄位（per-step 統計）。

**現況問題**：`parse_session_stats_internal()` 僅為 Copilot 設計（讀取 `events.jsonl`），OpenCode session 呼叫 `get_session_stats` 時回傳全部為 0 的統計結果。

## Goals / Non-Goals

**Goals:**
- 實作 OpenCode JSON storage 讀取層，能計算出 token 用量（input/output/reasoning/cache）、互動次數、工具呼叫統計、使用模型列表、session 時長
- Session list 繼續從 SQLite 讀取（速度快，保持現有行為）
- Stats 計算改從 JSON storage 讀取（資料完整）
- 建立 opencode-storage-schema 文件規格，記錄完整的 JSON 儲存結構供未來使用
- 統計結果快取至現有 metadata SQLite，避免重複讀取大量 JSON 檔案

**Non-Goals:**
- 對話歷史瀏覽 UI（未來功能）
- 終端機切換操作（未來功能）
- Token 成本試算（未包含在此次 change）
- 時間序列統計圖表（未包含在此次 change）
- 支援 OpenCode 以外的其他 JSON-based AI 工具

## Decisions

### 決策 1：Session list 來源維持 SQLite，Stats 來源改為 JSON storage

**選擇**：分離 session list 與 stats 的資料來源。

**原因**：SQLite 查詢速度遠快於掃描數千個 JSON 檔案，session list 僅需 id/title/time/worktree 等輕量欄位，SQLite 已完整提供。Stats 計算則需要完整的 message/token 資料，只有 JSON storage 有。

**替代方案考慮**：全部改用 JSON storage → 捨棄，會大幅增加 session list 載入時間（需掃描 5000+ 檔案）。

### 決策 2：Stats 計算以 session 為單位做 JSON 資料夾掃描

**選擇**：`get_opencode_session_stats(session_id)` 掃描 `storage/message/{session_id}/` 下所有 msg_*.json，加總各訊息的 token 欄位。

**原因**：message 層已有完整 token 統計（tokens.input/output/reasoning/cache.read/write），不需深入 part 層即可得到正確總量。part 層的 step-finish tokens 是 per-step 子統計，加總後與 message 層相同，不需重複計算。

**替代方案考慮**：讀取 part/step-finish → 捨棄，資料量更大（23K files），且資料重複。

### 決策 3：沿用現有 SessionStats 型別，補齊 inputTokens 欄位

**選擇**：在 `SessionStats` 補充 `inputTokens: number`（目前只有 `outputTokens`），其餘欄位名稱維持不變。

**原因**：OpenCode message JSON 提供完整的 input/output 分離統計，而 Copilot events.jsonl 目前也僅提取 output tokens。補充 inputTokens 欄位可讓兩個 provider 的統計結構一致，不影響現有 UI。

### 決策 4：opencode-storage-schema 以 spec 文件形式記錄，不建立執行程式碼

**選擇**：建立 `openspec/specs/opencode-storage-schema/spec.md` 作為設計文件，定義各實體欄位、關聯與 ID 格式。

**原因**：此資料為研究基礎，供未來對話歷史、終端機切換等功能設計參考，不需要在程式碼中有對應的「schema 物件」，規格文件本身即為交付物。

## Risks / Trade-offs

- **[風險] JSON 檔案數量大（5799 message files）** → 緩解：stats 計算結果快取至 metadata DB，session 非 live 狀態時命中快取直接回傳，避免每次重掃
- **[風險] OpenCode storage 路徑在 Windows 為 `%LOCALAPPDATA%\..\local\share\opencode\storage`**（不同於 Linux/macOS 的 `~/.local`）→ 緩解：在 `resolve_opencode_root()` 已處理跨平台路徑，storage 子目錄使用相同邏輯延伸
- **[風險] OpenCode 版本更新可能改變 JSON 格式** → 緩解：解析時使用 `serde_json::Value` 允許未知欄位，關鍵欄位缺失時降級為 0 而非 panic
- **[Trade-off] 未整合 part 層細粒度資料** → 當前可接受，part 層資料用於未來對話歷史功能時再引入

## Migration Plan

1. 新增 `parse_opencode_session_stats()` 函式（不影響現有 Copilot 路徑）
2. 修改 `get_session_stats` Tauri command：依 `provider` 欄位分流（"copilot" → 現有邏輯，"opencode" → 新邏輯）
3. 更新 `SessionStats` 型別加入 `inputTokens`
4. 無需資料庫 migration（metadata DB 統計快取結構不變，僅新增 opencode stats entries）
5. 若新邏輯有問題，可透過清除 metadata DB 快取強制重算

## Open Questions

- OpenCode 是否會在未來版本新增 SQLite token 統計欄位？（若是，可簡化實作）
- `storage/session/` 目錄下的 projectID 子目錄是否固定存在，或有無 projectID 的 session？
