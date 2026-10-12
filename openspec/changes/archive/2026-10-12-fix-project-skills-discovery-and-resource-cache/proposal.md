# Proposal

## Why

專案 Skills 目前只探索專案根目錄下的 provider 技能入口，漏掉子專案中的 `.agents/skills`；Skills、Commands 的定時簽章檢查及 MCP 的過期重取，也讓再次進入頁面產生額外磁碟存取。使用者已透過既有重新整理按鈕管理外部變更，應改為首次載入後使用記憶體快取，讓再次進入立即呈現資料並降低重複讀取。

## What Changes

- 專案 Skills 探索涵蓋專案內的子目錄技能入口，保留子專案識別、來源位置及對應同步目標，避免同名技能合併或同步到專案根目錄。
- 探索時跳過大型相依套件、版本控制與建置輸出目錄，依既有專案忽略設定排除子樹；共用技能入口只遍歷一次。
- 專案及全域 Skills、Commands、MCP 採同一套首次載入／手動重新整理策略；停止 Skills、Commands 每 5 秒簽章輪詢，以及因資料過期、重新掛載、視窗焦點或網路恢復而自動重取。
- 快取保留至本次應用程式結束，空結果也是有效快取；切換頁籤或專案時，命中快取即呈現，不重新掃描或呼叫 CLI discovery。
- 既有重新整理按鈕只更新所屬 scope 與資源頁籤，更新期間保留原資料；成功替換快取，失敗保留上次結果並回報錯誤。
- 在工具內執行成功的 MCP 寫入、技能／指令同步或連結建立後，更新受影響快取；變更來源設定則以新的快取識別首次載入。

## Capabilities

### New Capabilities

- `agents-resource-cache`: 定義 Skills、Commands、MCP 的記憶體快取、手動重新整理、資料隔離、錯誤處理與操作後更新契約。

### Modified Capabilities

- `agents-skills-sync`: 新增子專案技能入口探索、同名技能隔離與子專案同步路徑需求。

## Impact

- 前端：`src/App.tsx` 的資源 query key、查詢選項、重新整理與 mutation handlers；`AgentsConfigView` 的技能識別及同步請求；Rust／TypeScript 技能型別同步調整。
- 後端：`src-tauri/src/agents_config.rs` 的專案技能探索與同步狀態建立、`resource_discovery.rs` 的技能入口取得及去重；延用既有 command 薄包裝與 `spawn_blocking`。
- 測試：正式 provider-aware Skills 掃描回歸，以及 query／IPC 呼叫次數、快取隔離、刷新失敗與操作後更新驗證。
- 不新增套件或磁碟快取格式。快取僅存在本次程序記憶體，重新啟動後於首次開啟資源頁籤重新偵測。
- 此次不擴充 Commands／MCP 的子專案探索，不調整 Session SQLite 快取、活動狀態、統計或其他 watcher。另案 `fix-sessionhub-opencode-global-skills` 的 CLI 環境與全域路徑修正保留原責任範圍。
