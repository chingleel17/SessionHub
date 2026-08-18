## Context

目前 `scan_agents_skills_internal` 以 `<project>/.agents/skills` 為正本，但 project target 實際只有 Claude；前端卻固定建立 Claude、Codex、OpenCode、Copilot 四個晶片，缺少的 status 因此被誤標為未安裝。Commands 雖掃描四家固定目錄，仍未依 `enabledProviders` 篩選，也只比較檔案 fingerprint。`list_mcp_configs_internal` 同樣固定解析四家設定。參見 `proposal.md` 的動機。

本機實測顯示 CLI 能力不一致：OpenCode 1.18.16 的 `debug skill` 與 `debug config` 可輸出合法 JSON，且專案／中立目錄分別得到 25/15 skills 與 21/16 commands，能反映 scope 差異。Copilot 的 `skill list --json` 在目前版本會產生無效 JSON，不納入；其 `mcp list --json` 與 Codex `mcp list --json` 可機器解析。Claude MCP 與 OpenCode MCP 只有文字輸出或查詢不穩定，第一版不納入。

## Goals / Non-Goals

**Goals:**

- 只為通過實測的 OpenCode Skills/Commands、Copilot MCP、Codex MCP 建立 adapter，在正確工作目錄執行唯讀 CLI 查詢。
- 以 `AppSettings.enabledProviders` 作為 Skills、Commands、MCP 唯一 provider 集合，並以 provider root registry 掃描各工具真正可使用的 project/global roots。
- 保留現有檔案同步與 MCP 設定編輯能力，將「檔案部署」和「provider 實際解析／執行」拆開。
- 讓支援項目的 CLI 缺失、版本差異、逾時或解析失敗只隱藏該次 CLI 狀態，不拖垮整頁。
- 以最小 CSS 範圍修正資源閱讀器穿透，同時遵循既有主題 token 與 modal 捲動模式。

**Non-Goals:**

- 不啟動互動式 provider session，也不要求模型呼叫來探測資源。
- 不為缺乏原生列舉能力的 provider 偽造「已載入」結論。
- 不為未驗證介面實作文字 parser、修補無效 JSON 或每次重新探測能力；新增支援須另行完成 CLI 實測與 adapter 測試。
- 不改變同步的 copy/link、衝突處理或 MCP 設定寫入格式。
- 不在此變更新增 provider、安裝／升級 CLI 或修改 provider 設定。

## Decisions

### D1：新增唯讀 CLI resource adapter，回傳正規化查詢結果

後端新增獨立的 resource discovery 模組，由支援的 Skills、Commands 與 MCP 查詢共用。核心模型包含：

- `ResourceKind`: skill、command、mcp
- `DiscoveryScope`: global、project、effective
- `DiscoverySource`: cli
- `ResourceState`: available、configured、disabled
- provider 級 `DiscoveryDiagnostic`: 查詢時間與經清理的非阻擋錯誤摘要

每個 adapter 宣告固定 argv 與 JSON parser，不透過 shell 拼接命令。第一版能力矩陣固定為：OpenCode Skills 使用 `opencode debug skill`、OpenCode Commands 使用 `opencode debug config` 的 resolved command 區段、Copilot MCP 使用 `copilot mcp list --json`、Codex MCP 使用 `codex mcp list --json`。其餘組合不註冊 adapter，因此不啟動 CLI，也不建立 fallback discovery 狀態。

選擇固定能力矩陣而非啟動時猜測 `--help` 或解析文字，可避免版本、本地化與格式漂移造成誤判。沒有 adapter 的 provider 仍沿用既有檔案同步或 MCP 設定管理，不顯示 CLI 執行狀態。

### D2：provider 集合完全取自 enabledProviders

前端不再宣告固定 `CHIP_PLATFORMS`；後端 Skills、Commands、MCP command 接收當次 settings snapshot 中的 `enabled_providers`，先依已知 provider registry 去重並保持設定頁順序。停用 provider 不解析 root、不掃描檔案、不啟動 CLI、不建立 UI 欄位。未知 provider id 忽略並產生非阻擋 diagnostic，避免舊設定造成整頁失敗。

`enabledTargets` 保留為同步 modal 的目標勾選偏好，但只能在啟用 provider 的交集內生效，且不得控制主清單 provider 可見性或被翻譯為安裝狀態。

替代方案是在前端過濾固定四平台，但後端仍會做無效 I/O，且 Antigravity 無法正確加入，因此選擇前後端共用 settings provider 集合。

### D3：以 provider root registry 掃描實際相容目錄

Registry 對每個 provider 分別定義 Skills 與 Commands 的 project/global roots及檔案格式。第一版依官方文件與既有相容行為定案如下；`<root>` 使用 AppSettings 對應 provider root，`<agentsRoot>` 為 provider 實際讀取的 `~/.agents`，自訂 `agentsSourceRoot` 仍是 SessionHub 同步正本，除非已連結到 `<agentsRoot>`，不得冒充 provider 可見 root。

| Provider | Project Skills roots | Global Skills roots | Project Commands roots | Global Commands roots |
|---|---|---|---|---|
| claude | `.claude/skills` | `<claudeRoot>/skills` | `.claude/commands` | `<claudeRoot>/commands` |
| codex | `.codex/skills`, `.agents/skills` | `<codexRoot>/skills`, `<agentsRoot>/skills` | `.codex/prompts` | `<codexRoot>/prompts` |
| opencode | `.opencode/skill`, `.opencode/skills`, `.claude/skills`, `.agents/skills` | `<opencodeConfigRoot>/skill`, `<opencodeConfigRoot>/skills`, `<claudeRoot>/skills`, `<agentsRoot>/skills` | `.opencode/command`, `.opencode/commands` | `<opencodeConfigRoot>/command`, `<opencodeConfigRoot>/commands` |
| copilot | `.github/skills`, `.agents/skills`, `.claude/skills` | `<copilotRoot>/skills`, `<agentsRoot>/skills` | `.github/prompts`, `.copilot/prompts`（既有 fallback） | `<copilotRoot>/prompts` |
| antigravity | `.gemini/skills`, `.agents/skills` | `<antigravityRoot>/skills`, `<agentsRoot>/skills` | `.gemini/commands` | `<antigravityRoot>/commands` |

Codex、OpenCode、Copilot、Gemini/Antigravity 的 `.agents/skills` 相容性已有官方依據；Claude 只登記 `.claude/skills`。未經查證的 cross-provider root 不加入 registry。OpenCode 與 Codex 的複數 roots 均需掃描，不能因 `.agents` 存在便跳過 provider-native root。

同一 provider 的同名資源 SHALL 保留所有 discovered locations；除非官方機器介面（第一版僅 OpenCode）能確認 effective path，root scanner 不自行猜測優先序或把其餘位置標為 shadowed。不同 provider 的同名資源不互相覆蓋。清單可依名稱合併成一列，但每個 provider 狀態顯示其 locations；OpenCode 另標示 CLI 確認的 effective path。同步仍以 SessionHub canonical source 執行，discovery 與 sync 不共用同一個「存在」判斷。

Skills 以子目錄中的 `SKILL.md` 為有效項目。Claude、Codex、OpenCode Commands 使用 `.md`，Copilot prompts 使用 `.prompt.md`（legacy `.copilot/prompts` 仍依既有行為處理），Gemini Commands 使用 `.toml`；scanner MUST 依 provider 格式解析名稱，不得用單一 `.md` glob 掃所有平台。

### D4：以工作目錄控制 project 與 global CLI 查詢語意

Project 查詢的 child process `current_dir` 設為 `project_cwd`，讓 provider 自行套用向上探索、workspace 設定、plugin 與 user 設定。Global 查詢使用 `%APPDATA%\SessionHub\cli-probe` 中立目錄；該目錄不放 provider 設定且不位於使用者專案 Git worktree，避免誤載目前專案資源。

CLI 能回傳 source/path 時，adapter 將項目歸類為 project、global 或 plugin/builtin；只回傳 effective 清單時則標示 `effective`，不以名稱或猜測路徑硬判 scope。全域群組採 global probe，專案群組採 project probe；因此兩者可同時呈現且符合 provider 自身優先序。

替代方案是為每家 CLI 注入「忽略 project 設定」旗標，但各家旗標不一致或不存在，中立 CWD 較可預測。

### D5：CLI 查詢有界且不洩漏機密

runner 使用直接 executable + argv、清空互動需求的環境設定、限制 stdout/stderr 容量，並以 `try_wait` 輪詢實作逾時；逾時後 kill child。查詢只接受白名單唯讀子命令，不把使用者可編輯字串當 argv。parser 僅保留名稱、狀態、來源類型與必要路徑；錯誤摘要移除 ANSI、截斷長度，且不包含完整 JSON config、headers、env 或 token。

同一重新整理生命週期每個支援的 provider/resource/scope 只跑一次，避免每列啟動 process。全域與專案 query key 分開，防止結果互相覆蓋；不支援的組合不進入 runner。

### D6：以 stable identity 合併 CLI、provider roots 與同步資料

Skills／Commands 先合併啟用 provider 的 root registry 掃描結果，再以 `(provider, kind, case-normalized name)` 建立 provider discovery；只有 OpenCode 另合併 CLI effective 狀態與 CLI-only 項目。每個 entry 的同步 targets 只包含「啟用且需要 SessionHub 佈署」的平台，不再假設每個 provider 都有單一 target。MCP 只有啟用的 Copilot 與 Codex 合併 CLI effective/configured 狀態；啟用的 OpenCode 與 Claude 維持設定資料；Antigravity 尚無現有 MCP 管理 adapter，不為了湊齊 enabled provider 建立空白分頁。

成功的 OpenCode CLI 查詢中未出現的檔案項目只代表未出現在 effective 清單，不得標為 notInstalled。支援的 CLI 查詢失敗時不產生 discovery badge，僅在群組層顯示非阻擋提示；檔案同步與設定資料照常顯示。

### D7：前端同時呈現 discovery 與 sync 兩個維度

Rust 與 TypeScript 同步新增 provider discovery 與 effective path 型別。`AgentsConfigView` 依後端回傳的 enabled provider order 渲染，不宣告固定平台陣列；每個 provider 顯示其 root 掃描可見性，OpenCode 再顯示 CLI effective 驗證。同步狀態以獨立摘要呈現，不使用 `enabledTargets` 推導 loaded/notInstalled。MCP 依 enabled provider 與現有 adapter 交集顯示，其中 Copilot/Codex 可顯示 CLI effective/configured 狀態。載入新結果時保留 React Query 前次資料並顯示非阻塞 updating 標記。

### D8：掃描與 CLI 檢查採頁籤驅動，不背景輪詢

只有首次切入 Skills、Commands 或 MCP 頁籤時掃描該類型的 enabled provider roots，並執行該類型已註冊的 CLI adapter；專案 Agents 頁分別查 project 與 global。使用者手動重新整理、enabledProviders 設定變更、Skills/Commands 同步成功、或 MCP 新增／編輯／刪除／啟停成功後，使對應 kind/scope query 失效並重查。Skills/Commands 使用五分鐘 stale time，MCP 使用三十秒 stale time；不設定背景輪詢，也不在只開啟 AGENTS.md 頁籤時啟動這些掃描或 CLI。

### D9：只讓資源預覽 modal 的內容 surface 不透明

`agents-preview-modal` 保留浮層陰影與 backdrop；對 modal 本體、detail header、`.explorer-content` 與 markdown body 套用不透明的 `--color-surface-*` token，不更動通用 `.dialog-card` 玻璃規則。這能修正截圖中的穿透，又不影響其他 modal。內容仍由既有 flex/min-height/overflow 結構在 modal 內捲動，並以 light/dark browser 測試確認。

## Risks / Trade-offs

- [CLI 版本更新造成 JSON 格式改變] → parser 採寬鬆欄位讀取並使用 fixture；解析失敗隱藏該次 CLI 狀態並保留檔案管理功能。
- [未支援 provider 缺少執行狀態] → 明確接受此限制；只顯示可證實的同步或設定狀態，不以文字解析補齊。
- [每次刷新啟動多個 process 有延遲] → provider 查詢並行、每個 scope/kind 去重、設定短逾時並保留前次畫面。
- [中立目錄仍可能受環境層設定影響] → 這些設定屬有效 global 環境，應保留；只確保中立目錄不含 project config 或 Git 向上探索來源。
- [CLI 額外列出的 plugin/builtin 資源不可編輯] → 顯示來源並停用寫入操作，不假裝它位於 SessionHub 管理的設定檔。
- [未來 CLI 新增穩定 JSON 介面] → 經人工實測 project/global scope 並加入 fixture 後，再擴充固定能力矩陣。
- [Provider 路徑規則隨版本改變] → root registry 集中管理並以 fixture/temporary directory 測試；只納入已查證路徑，新增 alias 不散落於掃描函式。

## Migration Plan

1. 先加入精簡 discovery 型別、runner、四個 JSON adapter 與 fixture 測試，不改現有 UI。
2. 擴充現有 scan/list 回傳，保留原欄位以維持同步與編輯流程。
3. 前端改用 discovery status 與新文案，移除從 sync 推導 loaded/notInstalled 的程式碼。
4. 套用閱讀器不透明 surface，完成 light/dark 與長內容檢查。
5. 若發生不可接受的 CLI 相容問題，可移除該固定 adapter 與前端 discovery 顯示；既有檔案同步及設定編輯資料未遷移，無持久資料 rollback。
