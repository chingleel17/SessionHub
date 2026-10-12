# Design

## Context

動機見 `proposal.md`。目前 `resource_discovery::skill_roots` 只產生 project cwd 根目錄下的 provider 入口；`collect_skill_dirs` 已能遞迴探索入口內的技能，漏掃發生在「入口所在的子專案」未被找到，而非技能名稱的巢狀深度。

`App.tsx` 以 Skills／Commands 的 5 秒 signature polling 組成資料 query key；首次空 signature 與隨後取得的 signature 可能觸發兩次掃描。MCP 與 Codex trust 查詢採有限 staleTime。`refreshAgentsQueries` 一次失效同 scope 的所有 Agents 資源。QueryClient 使用預設值，不能只移除輪詢而忽略 focus／reconnect 等預設重取。

技能清單目前以名稱分組，同步請求以 `targetRoot + name` 推導目標；子專案探索若只附加路徑字串，容易把同名技能合併或將子專案技能放進根目錄的 `.claude/skills`。

## Goals / Non-Goals

**Goals:**

- 同一快取識別首次載入只做一次資源查詢，命中快取時不再呼叫掃描、metadata signature、CLI discovery 或信任狀態偵測。
- 探索實體子專案技能入口，保留原生 provider 路徑規則、技能相對名稱與上下文。
- 快取與重新整理跨專案／全域畫面一致，且不藉由增加磁碟快取寫入換取效能。

**Non-Goals:**

- 不加入跨啟動的持久快取、自動資源 watcher、定時重驗證或整機搜尋。
- 不擴大 Commands／MCP 子專案探索，亦不調整 Session cache、live stats 等其他 I/O。
- 不改變各 CLI 對子專案技能的實際載入規則；檔案被 SessionHub 找到，不等同 CLI 已在根目錄工作階段載入。

## Decisions

### D1. 使用程序生命週期的 React Query 記憶體快取

Skills／Commands／MCP 的 project、global queries 統一採 `staleTime: Infinity`、`gcTime: Infinity`、`refetchOnMount: false`、`refetchOnWindowFocus: false`、`refetchOnReconnect: false`、`refetchInterval: false`、`retry: false`、`retryOnMount: false`。不使用 `'static'`，因為成功寫入後仍需要明確 invalidation。僅改資源 query 的選項，不更動 QueryClient 全域預設。

移除資源 UI 的 Skills／Commands signature queries 及 signature key 成分；後端 signature helper 若仍有其他使用者可保留，但不得由這些畫面呼叫。首次資料 query 直接掃描，避免空 signature 再切正式 signature 的雙掃描。

Codex project trust、Skills 連結狀態及專案 Agents prefs 為顯示相依資料，採同樣生命週期，納入對應頁籤的刷新；AGENTS.md 清單維持原查詢策略。頁籤不啟用時不預先掃描三種資源；專案頁既有的專案／全域雙分區維持同時首次載入，共用全域 query。

空結果可快取。首次查詢失敗時記錄該 key 已嘗試，保留錯誤畫面，切換頁面不再隱式重試；使用者重新整理才允許再試。需以逐 key 的 attempted／in-flight 狀態配合 enabled，不能只依賴 `retryOnMount: false` 阻擋 disabled→enabled 的自動重試。

逐 key 狀態轉移為 `unloaded → loading → ready | error`；重新整理由 `ready | error → loading`，保留最後成功 data；來源改變建立獨立的 `unloaded` key。另以 `dirty` 標記成功寫入後待更新資料，開啟時才進行一次明確更新。attempted 僅屬於當下 key，不阻止新來源首次載入；手動刷新直接發起該 key 的一次請求，開始與結束時更新 attempted／in-flight，而非清除所有 key 的歷史。

選擇記憶體而非 SQLite／localStorage 資料快取，是為了降低寫入並保持簡單；MCP 資料可能含憑證，無需再儲存副本。重新啟動後首次開啟才載入，沒有啟動時的全部資源預熱。

### D2. 穩定且完整的快取識別

每種資源使用穩定 key，包含資源種類、scope、正規化專案路徑、排序去重後的 enabled providers、影響該資源的來源路徑設定，以及 Skills 探索使用的 ignored paths。來源路徑識別涵蓋 agents／claude／codex／copilot／opencode／antigravity 的相關 root，使用已載入的設定值建立，不為命中查詢再存取檔案系統。

具體 key 格式維持既有 prefix：`[resourcePrefix, scopeKey, { providers, sourcePaths, ignoredPaths }]`，其中 prefix 為 `agents-skills`／`agents-commands`／`mcp-configs`，scopeKey 為 `global` 或正規化 project cwd。sourcePaths 使用固定欄位順序的物件，依下表取已載入設定的 camelCase 欄位，不將整份 settings 放進 key：

| 資源／scope | sourcePaths | ignoredPaths |
| --- | --- | --- |
| Skills／project | 空物件（入口及目標由 project cwd 決定） | 專案 prefs 的 ignoredPaths |
| Skills／global | agentsSourceRoot、claudeRoot、codexRoot、copilotRoot、antigravityRoot | 空陣列 |
| Commands／project | 空物件（路徑由 project cwd 決定） | 空陣列 |
| Commands／global | agentsSourceRoot、claudeRoot、codexRoot、copilotRoot、antigravityRoot | 空陣列 |
| MCP／project、global | codexRoot、copilotRoot（含 Codex trust／可能的全域來源顯示相依） | 空陣列 |

provider 字串與 ignoredPaths 排序、去重；路徑及 ignoredPaths 統一 `/`、Windows 小寫、移除非根路徑的尾端分隔符，忽略路徑移除首尾 `/`；未設定來源欄位使用空字串表示既有預設解析。process 的 USERPROFILE／APPDATA／OpenCode config home 與 CLI 環境視為本次啟動期間固定，不因外部設定／環境檔修改建立新 key，該變更由手動刷新重新讀取。`opencodeRoot` 是 Session 儲存 root，不是目前的 OpenCode 資源 config root，不能因它改變而重掃資源；若另案新增可設定資源 config root，須一併擴充對應 sourcePaths。

相依 query 分別使用 `agents-prefs + scopeKey`、`agents-root-link + agentsSourceRoot`、`codex-project-trust + project cwd + codexRoot`。其他同步偏好不影響探索結果，成功修改時更新 prefs 快取，不放進清單 key。

先等設定及必要的專案 prefs 查詢完成再啟用資料 query，避免以暫時空 providers／舊忽略設定先掃一次。Windows 路徑用既有前端正規化慣例統一大小寫與分隔符；不把 mtime、signature、時間戳、顯示設定或 MCP 憑證放進 key。

跨 scope／來源 key 切換不使用上一個 key 的 `placeholderData`：有精確快取就立即顯示，沒有就顯示首次載入狀態。重新整理同 key 時 React Query 保留原資料。修改來源設定形成新的首次載入上下文，其他顯示設定變更不觸發掃描。

### D3. 明確的 scope／kind 刷新與寫入後更新

在 App 層將刷新入口參數化為 scope 與當前 AgentsTab；子元件只傳 callback。Skills 刷新包含技能資料、連結 banner 與必要 prefs；Commands 包含指令資料與必要 prefs；MCP 包含清單及 project Codex trust；AGENTS.md 按鈕僅刷新原 AGENTS.md 資料與 prefs。不可沿用「全部資源一起 invalidate」。刷新 prefs 後若 key 改變，只載入新 key 一次，不再載入舊 key。

對同 key 的首次載入／刷新維持 single-flight；按鈕於 fetching 時停用，不取消後重啟查詢。被明確要求刷新的 inactive query 可直接 fetch；其他 scope／kind 不得因 broad prefix invalidation 被讀取。失敗保留最後成功 data，用 toast 呈現並維持可再試；不可以空資料替代失敗或開始自動 retry。

成功的技能／指令同步只更新有實際寫入的資源；dry-run、全部 skip 或失敗不刷新。成功 MCP upsert／delete／toggle／copy 刷新受影響 scope 的 MCP；HTTP 測試連線不刷新。全域 `.agents` 連結建立更新全域 Skills 與連結狀態；若已載入的全域 Commands 也依賴該連結，將其標記待更新。

寫入後 active query 明確重取一次；inactive 已載入 query 標記待更新，下次開啟再讀一次；從未載入的 query 不預熱。共享來源導致另一個已載入 scope 也受影響時，依實際寫入路徑標記對應 cache，避免全域畫面與專案全域分區不同步。標記在成功 query 更新後清除。

刷新與寫入完成競爭時，序列化同 key 更新；若寫入成功時先前掃描仍在進行，等其結束後做一次寫入後重取，不把 pre-write 結果視為最終新資料。操作固定捕捉當下 scope/key，不能在 await 後以目前 activeProject 覆蓋另一專案快取。

### D4. 單次探索專案上下文，再解析 provider 入口

新增供正式 provider-aware scan 使用的專案技能上下文探索：從 project cwd 向下列舉實體目錄，對每個上下文套用現有 `skill_roots` 的 provider 相對入口規則。因此子目錄 `.agents/skills` 與其他已支援 provider 入口均一致可被找到，不新增 provider 規則。

遍歷不讀普通專案檔案內容。遇到 `.agents`、`.claude`、`.codex`、`.opencode`、`.github`、`.gemini` 等 provider 設定子樹，在父上下文完成入口檢查後剪枝，不把技能附件逐目錄當成子專案。沿用 `FIXED_IGNORED_DIRS` 與專案 ignored paths，Windows 比對大小寫不敏感，以 `filter_entry` 或 `skip_current_dir` 真正跳過整棵子樹，而非只過濾結果。

不沿用 AGENTS.md 的深度 8 限制；有較深層實體子專案仍需探索。沿用 `MAX_SCANNED_DIRS` 20,000 個有效專案目錄的總預算，不把被剪枝子樹計入。超出時保留結果並回傳 diagnostics 說明掃描不完整，不能靜默聲稱完整。無權限、目錄消失等錯誤亦回傳可理解的路徑診斷，其餘入口繼續。

診斷沿用 `SkillsScanResult.diagnostics: DiscoveryDiagnostic[]`，既有 Rust／TS 契約為 `{ providerId, kind: "skill", scope: "project", message }`，不另造回傳 envelope。受影響 provider 的 message 包含上限或失敗路徑與原因；共用入口錯誤可各自對應 provider，UI 按 scope/message 去重。存在部分結果時整個 command 回傳 `Ok`，結果與 diagnostics 一起快取；入口根目錄本身不存在或不可解析、設定無法解析等無法開始掃描的錯誤才回傳 `Err`。全域既有 CLI diagnostics 和 MCP per-provider error 亦維持部分成功語意，不因單一 provider 診斷丟棄其餘結果。

Skills 清單上方呈現資訊性診斷區：達預算時以 i18n 標示「掃描結果不完整」及上限／原因，路徑失敗顯示對應路徑與原因；不得只記錄後端 log 或只靠短暫 toast。再次進入須保留快取診斷，成功刷新以新結果替換，失敗刷新維持原診斷並另外顯示操作錯誤。

不追蹤一般專案子目錄 symlink／junction，以免跨出專案、重複搜尋或迴圈；實際發現的技能入口及技能目錄則延用 `follow_links(true)`，支援集中管理的連結技能，回報損壞連結／迴圈，不因單一錯誤終止整個探索。

依 `(上下文, 正規化入口路徑)` 快取入口遍歷結果，讓同一 `.agents/skills` 在多 provider 間重用；仍保留每個 provider 的 configured／CLI 語意。雜湊亦可在單次 scan 內依實體檔案／目錄路徑重用，不延伸成另一套跨查詢後端快取。CLI discovery 保持原本 scope 呼叫頻率，不逐子專案啟動 CLI。

### D5. 子專案識別與同步路徑獨立於顯示名稱

`SkillEntry` 新增可空的 `projectRelativeDir`：全域為 null，專案根目錄為空字串，子專案為從 project cwd 算出的相對路徑。`name` 保留技能在入口內的相對名稱（如 `team/demo`），`sourceDir`／`skillMdPath` 保留實際路徑；Rust serde 與 TS 同步新增欄位。

前端分組、選取、React key 與 provider 對照使用序列化的 `(projectRelativeDir, name)`，不再只用 name。名稱列可顯示 `packages/api · demo`，預覽來源路徑維持可查；根目錄／全域名稱呈現不變。

來源正本與同步目標在各自上下文計算。例如 `packages/api/.agents/skills/demo` 對應 `packages/api/.claude/skills/demo`，不可改成根目錄 `.claude/skills/packages/api/demo`。`TargetStatus.targetRoot` 使用該上下文的 provider 目標根，並新增可空 `targetRootExists` 提供 Skills 列級可同步條件；Commands 沿用既有 TargetInfo 條件。同步 modal 的 provider 欄仍是每個 provider 一欄，不因子專案入口重複欄位；技能同步用該列 raw name 組合目標，不能使用帶上下文的顯示標籤。

不同上下文的同名技能不合併；同一上下文跨 provider 的原有分組與多位置優先序維持。技能入口中 `team/demo` 等名稱仍可遞迴發現；同一技能實體被多個上下文連結時可共用遍歷／雜湊，但不能遺失上下文與部署位置。

### D6. 用呼叫次數驗證快取，不以硬碟壽命作量化承諾

後端測試須呼叫正式 `scan_agents_skills_with_providers`，涵蓋 sibling 子專案、同名技能、深層入口、忽略子樹、連結與同步目標。前端以既有 Bun／happy-dom、QueryClient／QueryObserver 與 mock IPC 驗證首次 1 次、切換／focus／reconnect／超過原 staleTime 0 次、手動刷新 1 次、錯誤無自動重試；不新增測試相依套件。

只統計本次資源命令，不把 Session、quota 等其他系統 I/O 混進驗收。快取命中的「立即呈現」定義為不等待後端掃描的同步快取讀取，不承諾所有硬體下固定毫秒數。

官方參考：TanStack Query v5 [Important Defaults](https://tanstack.com/query/latest/docs/framework/react/guides/important-defaults)、[Query Invalidation](https://tanstack.com/query/latest/docs/framework/react/guides/query-invalidation)。

## Risks / Trade-offs

- [外部修改暫時不可見] → 明確使用既有重新整理更新，操作後同步更新受影響資料，不加入背景監看。
- [快取隨已開啟專案增加占用記憶體] → 僅載入使用過的資源頁籤，儲存清單／metrics 而非完整附件或預覽內容；程序結束釋放，不加時間到期導致隱式重掃。
- [首次探索大型 repository 較慢] → 目錄剪枝、20,000 目錄預算及部分結果診斷、共用入口去重，並以 blocking worker 執行。
- [新型別或 UI 分組導致錯誤同步] → 以原始相對名稱、上下文目標根與列級 rootExists 建請求，回歸測試檢查實際 dry-run 路徑。
- [另案 OpenCode 全域技能修正重疊] → 本案移除 signature UI 用法並處理 project 上下文；另案維持 global CLI 環境與有效路徑責任，合併時不得重新加入輪詢。

## Migration Plan

1. 同步更新 Rust／TS 回傳欄位、探索與 UI 識別，讓根目錄／全域行為向後相容。
2. 啟用 stable-key 記憶體快取與明確刷新，移除 UI signature polling，補測操作後更新及錯誤路徑。
3. 不遷移 settings、SQLite 或 provider 檔案；既有重新整理按鈕保留。
4. 回復程式版本即可回復舊查詢策略；本案不新增待清理的磁碟快取。
