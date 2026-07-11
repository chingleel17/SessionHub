# Tasks: add-mcp-config-management

## 1. 相依與基礎

- [x] 1.1 `cargo add toml_edit` 加入 TOML 格式保留編輯相依
- [x] 1.2 `serde_json` 啟用 `preserve_order` feature（`cargo add serde_json --features preserve_order`），並確認既有測試不受鍵順序改變影響
- [x] 1.3 將 `src-tauri/src/agents_config.rs` 的 `atomic_write_file` 改為 `pub(crate)` 供 MCP 模組重用

## 2. 後端 mcp_config 模組

- [x] 2.1 建立 `src-tauri/src/mcp_config.rs`：定義 `McpScope`（Global / Project）、`McpServerEntry`、`McpProviderConfig`（camelCase serde）與 provider 常數清單
- [x] 2.2 實作 `mcp_config_path(provider, scope)`：global 分支（claude=`USERPROFILE\.claude.json`、codex=`<codexRoot>\config.toml`、opencode=`~/.config/opencode/opencode.json`、copilot=`<copilotRoot>\mcp-config.json`）與 project 分支（claude=`<project>\.mcp.json`、codex=`<project>\.codex\config.toml`、opencode=`<project>\opencode.json`、copilot 讀取優先 `.github\mcp.json` fallback `.mcp.json`、寫入固定 `.github\mcp.json`）
- [x] 2.3 實作 JSON 讀寫共用（讀取為物件、缺檔視為空、pretty 寫回 + atomic write）與 TOML 讀寫（`toml_edit::DocumentMut`），皆以設定檔路徑 + 區段鍵參數化、與 scope 無關
- [x] 2.4 實作 TOML Item/Value 與 serde_json Value 的雙向轉換（object↔table、array、datetime→string、拒絕 null）
- [x] 2.5 實作 `list_mcp_configs_internal(scope)`：四平台清單、enabled 判定（codex/opencode 依 `enabled` 旗標；claude/copilot 恆為 true）、單平台錯誤隔離到 `error` 欄位
- [x] 2.6 實作停用暫存 `%APPDATA%\SessionHub\mcp-disabled.json` 的讀寫（鍵為 `<provider>::<scopeKey>`，scopeKey = "global" 或正規化專案路徑），清單合併對應 scope 暫存項目（enabled=false）
- [x] 2.7 實作 `upsert_mcp_server_internal(scope, ...)`（含改名、JSON 物件驗證、名稱非空驗證、停用暫存中項目就地更新、project 缺檔時建立父目錄與設定檔）
- [x] 2.8 實作 `delete_mcp_server_internal(scope, ...)`（設定檔與暫存同步移除、冪等）
- [x] 2.9 實作 `set_mcp_server_enabled_internal(scope, ...)`（codex/opencode 原生旗標；claude/copilot 暫存搬移/還原）
- [x] 2.10 實作 `is_codex_project_trusted(project_cwd) -> bool`：讀取 `~/.codex/config.toml` 的 `[projects."<路徑>"]` 區塊，正規化路徑後比對 `trust_level == "trusted"`，找不到區塊視為 untrusted
- [x] 2.11 撰寫單元測試：codex 註解保留與 toggle、claude 停用搬移/啟用還原、opencode 改名與原生 toggle、copilot 建檔與無效輸入拒絕、codex trust 偵測（trusted / untrusted / 區塊不存在 / 路徑大小寫差異），並涵蓋 project scope 路徑解析與 global/project 暫存隔離（以 USERPROFILE/APPDATA 環境變數覆寫隔離測試環境）

## 3. Tauri commands 與註冊

- [x] 3.1 建立 `src-tauri/src/commands/mcp_config.rs`：`list_mcp_configs`、`upsert_mcp_server`、`delete_mcp_server`、`set_mcp_server_enabled`（皆帶 `scope` 參數、`spawn_blocking`）、`check_codex_project_trust`（`spawn_blocking`）
- [x] 3.2 `commands/mod.rs` 與 `lib.rs` 註冊模組與五個 commands
- [x] 3.3 `cargo test` 全綠、`cargo build` 可編譯

## 4. 前端型別與資料流

- [x] 4.1 `src/types/index.ts` 新增 `McpScope`、`McpServerEntry`、`McpProviderConfig` 型別
- [x] 4.2 `App.tsx` 新增 global 的 `mcp-configs` useQuery（`enabled: activeView === "mcp-global"`，query key 帶 scope）與 upsert/delete/toggle mutations（帶 scope、成功後 invalidate 對應 scope + toast、失敗 toast）
- [x] 4.3 `App.tsx` 新增 project 的 `mcp-configs` useQuery（`enabled` 綁定當前專案的 mcp sub-tab 是否啟用，比照 agents query）
- [x] 4.4 `App.tsx` 新增 `codex-project-trust` useQuery（`enabled` 綁定 project scope 的 codex 分頁是否啟用，帶 `projectCwd`）
- [x] 4.5 `App.tsx` 擴充 activeView：`"mcp-global"` 加入 view 重設條件與 workspace 標題列 title/subtitle 分支

## 5. 前端 UI

- [x] 5.1 `Icons.tsx` 新增 `PlugIcon`
- [x] 5.2 建立 `src/components/McpConfigView.tsx`：收 `scope` prop，provider 分頁、工具列（設定檔路徑、外部開啟、檔案總管顯示、重新整理、新增）、server 表格（名稱/狀態 pill/摘要/操作），純顯示元件、資料與 handlers 全由 props 傳入
- [x] 5.3 實作新增/編輯 dialog（名稱 + JSON textarea、JSON 物件驗證、重名檢查、錯誤顯示）
- [x] 5.4 實作停用/啟用切換與刪除確認（沿用 ConfirmDialog）；停用中項目在表格以獨立「已停用」樣式與啟用項目並列顯示，且照常可編輯/刪除/重新啟用
- [x] 5.5 `Sidebar.tsx` footer（收合與展開兩種狀態）新增 global MCP 導覽鈕
- [x] 5.6 `ProjectView.tsx` sub-tab 列新增 "mcp" 分頁，渲染 `McpConfigView` 並傳入 project scope
- [x] 5.7 `McpConfigView` 的 project scope + codex 分頁：接收 `codexTrusted` prop，untrusted 時於分頁頂端顯示警示 banner（此專案尚未被 codex 信任、設定不會生效），trusted 或 global scope 不顯示

## 6. i18n 與收尾

- [x] 6.1 `zh-TW.ts`、`en-US.ts` 補齊 `mcp.*` 全部鍵值
- [x] 6.2 TypeScript 檢查（`tsc --noEmit`）與前端 build 通過
- [x] 6.3 手動驗證（global）：四平台實機清單載入、codex 註解保留、claude 停用/啟用還原、`.claude.json` 其餘內容不變
- [x] 6.4 手動驗證（project）：在測試專案新增各平台 server、確認寫入專案層設定檔（`.mcp.json` / `.codex\config.toml` / `opencode.json` / `.github\mcp.json`）且 global 不受影響、global 與 project 同名 server 停用互不干擾
- [x] 6.5 手動驗證（codex trust 提示）：對一個未在 `~/.codex/config.toml` 註冊的專案確認顯示未信任提示；對一個 `trust_level = "trusted"` 的專案確認不顯示提示

## 7. MCP 入口整合與專案分頁雙分區（D10 / D11）

- [x] 7.1 `McpConfigView` 改為可內嵌內容元件（去除自帶 info-card 外框），`AgentsConfigView` 新增第四個頁籤「MCP」渲染之（收 MCP 資料與 handlers props，依實例 scope）
- [x] 7.2 移除 sidebar footer 的 MCP 導覽鈕與 `activeView: "mcp-global"`（標題列分支、view 重設條件、獨立 render 區塊）；全域 Agents 頁傳入 global MCP props
- [x] 7.3 `ProjectView` 移除獨立 "mcp" sub-tab；殘留 `activeSubTab === "mcp"` 的舊狀態正規化為 "agents"
- [x] 7.4 建立可收折分區容器（標題列 + chevron + 受控狀態），專案 Agents sub-tab 改為「專案」「全域」雙分區，各渲染一份完整 `AgentsConfigView`（含 MCP 頁籤）；收折狀態 localStorage 記憶（key 帶專案路徑；預設專案展開、全域收折）；codex trust banner 僅在專案分區
- [x] 7.5 `App.tsx`：global mcp-configs 與 agents skills/commands queries 的 `enabled` 條件擴充為「agents-global 頁啟用中 或 當前專案 agents sub-tab 啟用中」；global mutations/handlers 供專案分頁全域分區重用
- [x] 7.6 摘要欄精簡（D13）：`description` > `url` > 指令 basename + 參數，單行截斷 + tooltip
- [x] 7.7 編輯 dialog 改為類型導向表單（D12）：類型下拉（HTTP/SSE、npx、本機執行檔、自訂 JSON）、依類型顯示欄位、依 provider 組裝原生 schema、編輯時反解析帶入表單、各類型驗證
- [x] 7.8 排版間距修正：MCP 工具列/清單與相鄰區塊的垂直間距，分區之間的間距
- [x] 7.9 i18n：分區標題、收折控制、類型選單與表單欄位等鍵值（zh-TW / en-US）
- [x] 7.10 TypeScript 檢查與前端 build 通過；`cargo test` 維持全綠
- [x] 7.11 手動驗證：全域 Agents 頁 MCP 頁籤正常載入；專案分頁雙分區顯示與收折記憶；全域分區操作寫入 global 且與全域頁一致；結構化表單四類型新增/編輯/反解析；摘要欄不再撐爆（已驗證，並回饋出第 8 節的版面重新設計需求）

## 8. 版面重構：單一頁籤列 + scope 分組 + 同步 modal（D10 修訂 / D14 / D15）

- [x] 8.1 後端：`SkillEntry` / `CommandEntry` 新增 `description` 欄位，掃描時自 `SKILL.md` / `.md` frontmatter 抽取 `description`（缺檔/無 frontmatter 回空值）；補單元測試；`cargo test` 全綠
- [x] 8.2 前端型別：`SkillEntry` / `CommandEntry` 新增 `description?`
- [x] 8.3 專案 Agents 分頁版面反轉：移除雙完整實例分區，改為單一 `AgentsConfigView`（或等效容器）帶共用頁籤列，頁籤內容以「專案」「全域」兩個可收折群組呈現（群組標題含計數與 chevron；收折狀態 localStorage 記憶、預設專案展開全域收折；沿用/調整 `CollapsibleSection`）
- [x] 8.4 AGENTS.md 頁籤分組：專案群組 = 專案樹狀檢視、全域群組 = 全域根目錄清單（全域 Agents 頁不分組維持現狀）
- [x] 8.5 Skills / Commands 清單改 VS Code 式：每列名稱 + 描述（截斷 + tooltip）+ 精簡狀態 badge，移除常駐狀態矩陣欄；頁籤頂部搜尋框過濾兩群組（計數同步更新）
- [x] 8.6 同步 modal：矩陣（項目/目標勾選）、同步模式、預覽/套用、結果報告整組搬入 modal；「同步」按鈕置於群組/工具列；SyncConflictDialog 疊層高於 modal；modal 關閉清除狀態
- [x] 8.7 MCP 頁籤分組：provider 分頁共用一列置頂，其下專案/全域群組各含工具列與 server 清單；codex trust banner 僅在專案群組
- [x] 8.8 i18n：搜尋框、群組計數、同步 modal、狀態 badge 等鍵值（zh-TW / en-US）
- [x] 8.9 TypeScript 檢查與前端 build 通過；`cargo test` 全綠
- [x] 8.10 手動驗證：頁籤同步切換兩群組、收折記憶、搜尋過濾、描述顯示、同步 modal 完整流程（含衝突）、MCP 分組操作寫入正確 scope

## 9. 群組內容去重與寬度自適應（D16）

- [x] 9.1 移除 Skills / Commands 群組內容區的「scope 名稱 + 計數」工具列區塊（資訊已由收折群組標題列呈現；全域 Agents 頁單一清單模式的對應呈現一併調整）
- [x] 9.2 「同步」與重新整理按鈕併入說明列右側：skills 與 `.agents` 相容性說明同列（說明文字縮排讓出空間）；commands 無說明列時以單一窄列容納按鈕
- [x] 9.3 修正水平溢出：Agents 頁內容隨可用寬度自適應（flex/grid 鏈補 `min-width: 0`、表格溢出侷限卡片內、長文字截斷），sidebar 展開/收合皆無頁面層級水平滾動條
- [x] 9.4 TypeScript 檢查與前端 build 通過
- [x] 9.5 手動驗證：sidebar 展開/收合無水平滾動條；群組內無重複標題區塊；同步/重整按鈕位於說明列右側且功能正常
