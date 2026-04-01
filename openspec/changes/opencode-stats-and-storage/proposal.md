## Why

目前應用程式對 OpenCode 的 session 統計（token 用量、互動次數、工具呼叫等）完全無法正確解析，原因是現有實作讀取的是 OpenCode 的 SQLite 資料庫（`opencode.db`），該資料庫僅存有 session metadata（標題、時間、code diff 摘要），**並未包含 token 用量與對話訊息**。實際的訊息與 token 資料存放在 `~/.local/share/opencode/storage/` 目錄下的 JSON 檔案系統中，需要改用 JSON 解析取代現有 SQLite 讀取方式，才能獲得完整的統計資料。此外，opencode 儲存格式的完整文件尚未建立，需一併記錄以利未來開發對話歷史瀏覽與終端機操控等功能。

## What Changes

- **新增** OpenCode JSON 儲存層解析邏輯（Rust 後端），從 `storage/session/`、`storage/message/`、`storage/part/` 讀取資料
- **新增** OpenCode session 統計計算：token 用量（input/output/reasoning/cache）、互動次數、工具呼叫統計、使用模型列表、session 時長
- **修改** 現有 OpenCode 掃描邏輯，session list 仍從 SQLite 讀取（效率高），但 stats 改從 JSON 儲存層讀取
- **新增** opencode-storage-schema 文件規格，記錄 JSON 儲存結構、實體關聯、ID 格式，作為未來對話歷史與終端機功能的研究基礎
- **修改** `session-stats-display`：讓 OpenCode session 的統計顯示與 Copilot 一致（顯示 tokens、turns、tools 等）

## Capabilities

### New Capabilities

- `opencode-json-parser`: 解析 `~/.local/share/opencode/storage/` JSON 檔案系統，提供 session、message、part 資料的讀取與統計計算 API
- `opencode-storage-schema`: 記錄 opencode JSON 儲存結構的完整規格，包含實體定義、欄位說明、關聯圖與 ID 格式，作為後續對話歷史、終端機切換等功能的設計基礎

### Modified Capabilities

- `session-stats-display`: OpenCode session 需支援與 Copilot session 相同的統計展示（tokens, turns, tools, models, duration），目前 OpenCode stats 欄位皆為空或不正確

## Impact

- `src-tauri/src/lib.rs`：新增 OpenCode JSON storage 讀取函式、stats 計算邏輯
- `src/types/index.ts`：確認 `SessionStats` 型別欄位能涵蓋 OpenCode 統計（目前主要針對 Copilot 設計）
- `openspec/specs/`：新增 `opencode-json-parser/spec.md`、`opencode-storage-schema/spec.md`
- 不影響現有 Copilot session 解析邏輯
- 不影響 UI 元件結構（僅後端資料填充問題）
