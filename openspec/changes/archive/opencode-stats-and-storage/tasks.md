## 1. Rust 後端 - OpenCode JSON Storage 讀取層

- [x] 1.1 在 `lib.rs` 新增 `resolve_opencode_storage_root()` 函式，根據 opencode root 路徑組出 `storage/` 子目錄路徑
- [x] 1.2 新增 `parse_opencode_message_json()` 函式，解析單一 msg_*.json 為 `OpencodeMessage` struct（含 id、role、tokens、modelID、time 欄位）
- [x] 1.3 新增 `scan_opencode_message_dir()` 函式，掃描 `storage/message/{sessionID}/` 目錄，回傳所有 `OpencodeMessage`
- [x] 1.4 新增 `parse_opencode_part_json()` 函式，解析 prt_*.json（僅解析 type=tool 的 part，取得 state.tool 工具名稱）
- [x] 1.5 新增 `scan_opencode_parts_for_message()` 函式，掃描 `storage/part/{messageID}/` 目錄，回傳工具呼叫清單

## 2. Rust 後端 - OpenCode Stats 計算

- [x] 2.1 新增 `calculate_opencode_session_stats()` 函式，接受 session_id 與 storage_root，計算並回傳 `SessionStats`
- [x] 2.2 在計算函式中實作：user 訊息計算 interactionCount、assistant 訊息累計 tokens（input/output/reasoning/cache）
- [x] 2.3 在計算函式中實作：掃描每個 assistant message 的 part 目錄，累計 toolCallCount 與 toolBreakdown
- [x] 2.4 在計算函式中實作：從 message 的 time 欄位計算 durationMinutes（最新 - 最舊，單位毫秒轉分鐘）
- [x] 2.5 在計算函式中實作：從 assistant 訊息的 modelID 欄位收集 modelsUsed（去重 Vec）
- [x] 2.6 欄位缺失時降級為 0，不 panic（使用 `Option::unwrap_or_default()`）

## 3. Rust 後端 - Stats 快取與 Command 整合

- [x] 3.1 在 `get_session_stats` Tauri command 中，依 `provider` 欄位分流：`"copilot"` 走現有邏輯，`"opencode"` 呼叫 `calculate_opencode_session_stats()`
- [x] 3.2 將 OpenCode stats 計算結果寫入 metadata DB 快取（沿用現有 Copilot stats 快取 table 結構）
- [x] 3.3 快取讀取邏輯：非 live session 先查快取，命中則直接回傳；live session 或無快取則重新計算
- [x] 3.4 確認 `resolve_opencode_storage_root()` 在 Windows 路徑下正確運作

## 4. TypeScript 前端 - 型別更新

- [x] 4.1 在 `src/types/index.ts` 的 `SessionStats` 型別中新增 `inputTokens: number` 欄位
- [x] 4.2 確認現有 Copilot stats 呼叫路徑在 `inputTokens` 缺失時降級為 0（向後相容）

## 5. 前端 - SessionStatsPanel 顯示 inputTokens

- [x] 5.1 在 `SessionStatsPanel.tsx` 新增 inputTokens 顯示區塊，僅在 `inputTokens > 0` 時顯示
- [x] 5.2 在 `src/locales/zh-TW.ts` 與 `en-US.ts` 新增 `inputTokens` 對應 i18n key（「輸入 Token」/ 「Input Tokens」）

## 6. 驗證與測試

- [x] 6.1 手動驗證：在 UI 中切換至 OpenCode session，確認 SessionStatsBadge 顯示非零的 token 與 turns 數值
- [x] 6.2 手動驗證：SessionStatsPanel 顯示正確的 toolBreakdown、modelsUsed、durationMinutes
- [x] 6.3 手動驗證：Copilot session 統計顯示不受影響
- [x] 6.4 手動驗證：`storage/message/` 目錄不存在的 OpenCode session 顯示全零值而非錯誤
- [x] 6.5 執行 `cargo build` 確認 Rust 編譯無錯誤
