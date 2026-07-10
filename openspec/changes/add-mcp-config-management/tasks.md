# Tasks: add-mcp-config-management

## 1. 相依與基礎

- [ ] 1.1 `cargo add toml_edit` 加入 TOML 格式保留編輯相依
- [ ] 1.2 `serde_json` 啟用 `preserve_order` feature（`cargo add serde_json --features preserve_order`），並確認既有測試不受鍵順序改變影響
- [ ] 1.3 將 `src-tauri/src/agents_config.rs` 的 `atomic_write_file` 改為 `pub(crate)` 供 MCP 模組重用

## 2. 後端 mcp_config 模組

- [ ] 2.1 建立 `src-tauri/src/mcp_config.rs`：定義 `McpScope`（Global / Project）、`McpServerEntry`、`McpProviderConfig`（camelCase serde）與 provider 常數清單
- [ ] 2.2 實作 `mcp_config_path(provider, scope)`：global 分支（claude=`USERPROFILE\.claude.json`、codex=`<codexRoot>\config.toml`、opencode=`~/.config/opencode/opencode.json`、copilot=`<copilotRoot>\mcp-config.json`）與 project 分支（claude=`<project>\.mcp.json`、codex=`<project>\.codex\config.toml`、opencode=`<project>\opencode.json`、copilot 讀取優先 `.github\mcp.json` fallback `.mcp.json`、寫入固定 `.github\mcp.json`）
- [ ] 2.3 實作 JSON 讀寫共用（讀取為物件、缺檔視為空、pretty 寫回 + atomic write）與 TOML 讀寫（`toml_edit::DocumentMut`），皆以設定檔路徑 + 區段鍵參數化、與 scope 無關
- [ ] 2.4 實作 TOML Item/Value 與 serde_json Value 的雙向轉換（object↔table、array、datetime→string、拒絕 null）
- [ ] 2.5 實作 `list_mcp_configs_internal(scope)`：四平台清單、enabled 判定（codex/opencode 依 `enabled` 旗標；claude/copilot 恆為 true）、單平台錯誤隔離到 `error` 欄位
- [ ] 2.6 實作停用暫存 `%APPDATA%\SessionHub\mcp-disabled.json` 的讀寫（鍵為 `<provider>::<scopeKey>`，scopeKey = "global" 或正規化專案路徑），清單合併對應 scope 暫存項目（enabled=false）
- [ ] 2.7 實作 `upsert_mcp_server_internal(scope, ...)`（含改名、JSON 物件驗證、名稱非空驗證、停用暫存中項目就地更新、project 缺檔時建立父目錄與設定檔）
- [ ] 2.8 實作 `delete_mcp_server_internal(scope, ...)`（設定檔與暫存同步移除、冪等）
- [ ] 2.9 實作 `set_mcp_server_enabled_internal(scope, ...)`（codex/opencode 原生旗標；claude/copilot 暫存搬移/還原）
- [ ] 2.10 實作 `is_codex_project_trusted(project_cwd) -> bool`：讀取 `~/.codex/config.toml` 的 `[projects."<路徑>"]` 區塊，正規化路徑後比對 `trust_level == "trusted"`，找不到區塊視為 untrusted
- [ ] 2.11 撰寫單元測試：codex 註解保留與 toggle、claude 停用搬移/啟用還原、opencode 改名與原生 toggle、copilot 建檔與無效輸入拒絕、codex trust 偵測（trusted / untrusted / 區塊不存在 / 路徑大小寫差異），並涵蓋 project scope 路徑解析與 global/project 暫存隔離（以 USERPROFILE/APPDATA 環境變數覆寫隔離測試環境）

## 3. Tauri commands 與註冊

- [ ] 3.1 建立 `src-tauri/src/commands/mcp_config.rs`：`list_mcp_configs`、`upsert_mcp_server`、`delete_mcp_server`、`set_mcp_server_enabled`（皆帶 `scope` 參數、`spawn_blocking`）、`check_codex_project_trust`（`spawn_blocking`）
- [ ] 3.2 `commands/mod.rs` 與 `lib.rs` 註冊模組與五個 commands
- [ ] 3.3 `cargo test` 全綠、`cargo build` 可編譯

## 4. 前端型別與資料流

- [ ] 4.1 `src/types/index.ts` 新增 `McpScope`、`McpServerEntry`、`McpProviderConfig` 型別
- [ ] 4.2 `App.tsx` 新增 global 的 `mcp-configs` useQuery（`enabled: activeView === "mcp-global"`，query key 帶 scope）與 upsert/delete/toggle mutations（帶 scope、成功後 invalidate 對應 scope + toast、失敗 toast）
- [ ] 4.3 `App.tsx` 新增 project 的 `mcp-configs` useQuery（`enabled` 綁定當前專案的 mcp sub-tab 是否啟用，比照 agents query）
- [ ] 4.4 `App.tsx` 新增 `codex-project-trust` useQuery（`enabled` 綁定 project scope 的 codex 分頁是否啟用，帶 `projectCwd`）
- [ ] 4.5 `App.tsx` 擴充 activeView：`"mcp-global"` 加入 view 重設條件與 workspace 標題列 title/subtitle 分支

## 5. 前端 UI

- [ ] 5.1 `Icons.tsx` 新增 `PlugIcon`
- [ ] 5.2 建立 `src/components/McpConfigView.tsx`：收 `scope` prop，provider 分頁、工具列（設定檔路徑、外部開啟、檔案總管顯示、重新整理、新增）、server 表格（名稱/狀態 pill/摘要/操作），純顯示元件、資料與 handlers 全由 props 傳入
- [ ] 5.3 實作新增/編輯 dialog（名稱 + JSON textarea、JSON 物件驗證、重名檢查、錯誤顯示）
- [ ] 5.4 實作停用/啟用切換與刪除確認（沿用 ConfirmDialog）；停用中項目在表格以獨立「已停用」樣式與啟用項目並列顯示，且照常可編輯/刪除/重新啟用
- [ ] 5.5 `Sidebar.tsx` footer（收合與展開兩種狀態）新增 global MCP 導覽鈕
- [ ] 5.6 `ProjectView.tsx` sub-tab 列新增 "mcp" 分頁，渲染 `McpConfigView` 並傳入 project scope
- [ ] 5.7 `McpConfigView` 的 project scope + codex 分頁：接收 `codexTrusted` prop，untrusted 時於分頁頂端顯示警示 banner（此專案尚未被 codex 信任、設定不會生效），trusted 或 global scope 不顯示

## 6. i18n 與收尾

- [ ] 6.1 `zh-TW.ts`、`en-US.ts` 補齊 `mcp.*` 全部鍵值
- [ ] 6.2 TypeScript 檢查（`tsc --noEmit`）與前端 build 通過
- [ ] 6.3 手動驗證（global）：四平台實機清單載入、codex 註解保留、claude 停用/啟用還原、`.claude.json` 其餘內容不變
- [ ] 6.4 手動驗證（project）：在測試專案新增各平台 server、確認寫入專案層設定檔（`.mcp.json` / `.codex\config.toml` / `opencode.json` / `.github\mcp.json`）且 global 不受影響、global 與 project 同名 server 停用互不干擾
- [ ] 6.5 手動驗證（codex trust 提示）：對一個未在 `~/.codex/config.toml` 註冊的專案確認顯示未信任提示；對一個 `trust_level = "trusted"` 的專案確認不顯示提示
