# Tasks: add-mcp-config-management

## 1. 相依與基礎

- [ ] 1.1 `cargo add toml_edit` 加入 TOML 格式保留編輯相依
- [ ] 1.2 `serde_json` 啟用 `preserve_order` feature（`cargo add serde_json --features preserve_order`），並確認既有測試不受鍵順序改變影響
- [ ] 1.3 將 `src-tauri/src/agents_config.rs` 的 `atomic_write_file` 改為 `pub(crate)` 供 MCP 模組重用

## 2. 後端 mcp_config 模組

- [ ] 2.1 建立 `src-tauri/src/mcp_config.rs`：定義 `McpServerEntry`、`McpProviderConfig`（camelCase serde）與 provider 常數清單
- [ ] 2.2 實作各 provider 設定檔路徑解析（claude=`USERPROFILE\.claude.json`、codex=`<codexRoot>\config.toml`、opencode=`~/.config/opencode/opencode.json`、copilot=`<copilotRoot>\mcp-config.json`）
- [ ] 2.3 實作 JSON 讀寫共用（讀取為物件、缺檔視為空、pretty 寫回 + atomic write）與 TOML 讀寫（`toml_edit::DocumentMut`）
- [ ] 2.4 實作 TOML Item/Value 與 serde_json Value 的雙向轉換（object↔table、array、datetime→string、拒絕 null）
- [ ] 2.5 實作 `list_mcp_configs_internal`：四平台清單、enabled 判定（codex/opencode 依 `enabled` 旗標；claude/copilot 恆為 true）、單平台錯誤隔離到 `error` 欄位
- [ ] 2.6 實作停用暫存 `%APPDATA%\SessionHub\mcp-disabled.json` 的讀寫，清單合併暫存項目（enabled=false）
- [ ] 2.7 實作 `upsert_mcp_server_internal`（含改名、JSON 物件驗證、名稱非空驗證、停用暫存中項目就地更新）
- [ ] 2.8 實作 `delete_mcp_server_internal`（設定檔與暫存同步移除、冪等）
- [ ] 2.9 實作 `set_mcp_server_enabled_internal`（codex/opencode 原生旗標；claude/copilot 暫存搬移/還原）
- [ ] 2.10 撰寫單元測試：codex 註解保留與 toggle、claude 停用搬移/啟用還原、opencode 改名與原生 toggle、copilot 建檔與無效輸入拒絕（以 USERPROFILE/APPDATA 環境變數覆寫隔離測試環境）

## 3. Tauri commands 與註冊

- [ ] 3.1 建立 `src-tauri/src/commands/mcp_config.rs`：`list_mcp_configs`、`upsert_mcp_server`、`delete_mcp_server`、`set_mcp_server_enabled`（皆 `spawn_blocking`）
- [ ] 3.2 `commands/mod.rs` 與 `lib.rs` 註冊模組與四個 commands
- [ ] 3.3 `cargo test` 全綠、`cargo build` 可編譯

## 4. 前端型別與資料流

- [ ] 4.1 `src/types/index.ts` 新增 `McpServerEntry`、`McpProviderConfig` 型別
- [ ] 4.2 `App.tsx` 新增 `mcp-configs` useQuery（`enabled: activeView === "mcp-global"`）與 upsert/delete/toggle mutations（成功後 invalidate + toast、失敗 toast）
- [ ] 4.3 `App.tsx` 擴充 activeView：`"mcp-global"` 加入 view 重設條件與 workspace 標題列 title/subtitle 分支

## 5. 前端 UI

- [ ] 5.1 `Icons.tsx` 新增 `PlugIcon`
- [ ] 5.2 建立 `src/components/McpConfigView.tsx`：provider 分頁、工具列（設定檔路徑、外部開啟、檔案總管顯示、重新整理、新增）、server 表格（名稱/狀態 pill/摘要/操作），純顯示元件、資料與 handlers 全由 props 傳入
- [ ] 5.3 實作新增/編輯 dialog（名稱 + JSON textarea、JSON 物件驗證、重名檢查、錯誤顯示）
- [ ] 5.4 實作停用/啟用切換與刪除確認（沿用 ConfirmDialog）
- [ ] 5.5 `Sidebar.tsx` footer（收合與展開兩種狀態）新增 MCP 導覽鈕
- [ ] 5.6 claude/copilot 分頁顯示停用暫存機制提示文字

## 6. i18n 與收尾

- [ ] 6.1 `zh-TW.ts`、`en-US.ts` 補齊 `mcp.*` 全部鍵值
- [ ] 6.2 TypeScript 檢查（`tsc --noEmit`）與前端 build 通過
- [ ] 6.3 手動驗證：四平台實機清單載入、codex 註解保留、claude 停用/啟用還原、`.claude.json` 其餘內容不變
