## Context

專案頁面 sessions sub-tab 的篩選邏輯目前集中在 `src/components/ProjectView.tsx` 的 `filterAndSortSessions`——一個同步純函式，對 `project.sessions` 這個記憶體陣列做 filter + sort。所有現有條件（provider、tag、空 session、更新時間、關鍵字）都能在此完成，因為比對的欄位全都已經在 `SessionInfo` 內。

本次六項調整中，前五項（移除排序選項、加寬搜尋框、移除路徑比對、自訂日期區間、chip 文案與預設值）都落在這個同步邊界之內。第六項「搜尋對話內容」則不同：逐字稿內容不在 `SessionInfo` 裡，必須從磁碟讀取，且各 provider 的儲存格式各異——

| Provider | 逐字稿來源 |
| --- | --- |
| claude | session 目錄下的 `.jsonl` |
| codex | session 目錄下的 `.jsonl` |
| copilot | `events.jsonl` |
| antigravity | `transcript.jsonl` |
| opencode | SQLite（`message` / `part` 表） |

這是非同步、I/O-bound、且需要 per-adapter 實作的工作，無法塞進 `filterAndSortSessions`。

另有兩項既有約束需要處理：`openspec/specs/session-filter/spec.md` 目前要求篩選工具列維持單行、高度不超過 72px；加寬搜尋框並新增日期區間輸入後此限制不再成立，已在 spec delta 中放寬為允許換行的彈性佈局。

## Goals / Non-Goals

**Goals:**

- 前五項純前端調整可獨立完成並驗收，不被第六項的後端工作阻塞
- 對話內容搜尋以最小改動接上：新增一個 Rust 指令回傳命中的 session ID，前端與現有同步篩選取交集，`filterAndSortSessions` 維持純函式
- 各 provider 的內容擷取實作落在既有的 `sessions/<provider>.rs` 慣例內，透過 `mod.rs` 泛用分派

**Non-Goals:**

- 不建立全文索引（SQLite FTS）。索引建立、增量更新與儲存空間管理的工程量遠大於本次需求，且尚無證據顯示即時掃描的延遲不可接受
- 不將逐字稿內容以任何形式複製、快取或持久化至 `metadata.db`（詳見「決策七：對話內容不落地」）
- 不做搜尋結果的關鍵字上下文片段預覽（highlight snippet）。本次只需要「這個 session 有沒有提過」的是非判斷
- 不支援正則表示式或進階查詢語法，只做不分大小寫的子字串比對
- 不改動 session 掃描、快取或分頁機制

## Decisions

### 決策一：內容搜尋採「新增 Rust 指令即時掃描」而非 FTS 索引

新增 `search_session_content(query: String, sessions: Vec<SessionSearchTarget>) -> Result<Vec<String>, String>`，其中 `SessionSearchTarget` 至少含 `id`、`provider`、`session_dir`。回傳命中的 session ID。

**為何不選 FTS 索引**：索引方案需要處理首次建立（可能數千個 session 的全量掃描）、增量更新（session 持續寫入）、schema 遷移與磁碟佔用。以 YAGNI 原則，在沒有實測延遲問題前不引入。即時掃描的搜尋範圍已被大幅收窄——只掃描「當前專案分組內、且已通過其他同步篩選」的 session，而非全域所有 session，實際檔案數通常在數十個量級。

**為何回傳 ID 清單而非過濾後的 session**：讓後端只負責「內容有沒有命中」這一件事，session 的排序、分頁、其他篩選維度全部留在前端既有邏輯內，兩邊職責不重疊。

### 決策七：對話內容不落地（硬性約束）

`search_session_content` 及其擷取實作 SHALL NOT 將任何逐字稿內容寫入 `metadata.db`、寫入任何檔案，或存放於跨呼叫的記憶體快取。內容只在單次指令執行期間存在於記憶體，比對完即釋放；指令的回傳值只有 session ID 清單，不含任何訊息文字。

**這是刻意的架構約束，不是實作細節**。任何後續變更若要引入內容快取或索引，都必須明確推翻本決策並重新評估以下三項風險，而不能作為效能優化順手加上。

**理由一：避免 DB 膨脹**。逐字稿是本應用資料量最大的一類資料——單一 session 的對話動輒數百 KB 至數 MB。`metadata.db` 目前只存 session 中繼資料、stats 與列表快取，量級可控；若複製逐字稿進來，DB 大小將由使用者的對話總量決定，而非 session 數量，且會與 provider 各自的儲存重複一份。

**理由二：迴避快取一致性問題**。各 provider 對逐字稿的保留政策不明——是否自行清理、何時清理、清理粒度為何，都不在本應用掌控範圍內。即時掃描下這個未知數的後果極輕：檔案在就搜得到、檔案沒了就搜不到，永遠反映當下實情，不存在「原始檔已刪除但快取仍命中」的孤兒資料，也不需要任何失效偵測或同步機制。若改為快取，就必須額外處理 provider 靜默刪檔、session 續寫導致內容變更、以及跨機器同步時的不一致。

**理由三：縮小敏感資料的擴散面**。對話內容可能包含使用者貼入的機密資訊。不落地代表本應用不成為這類資料的第二個儲存位置，資料的生命週期仍完全由原 provider 掌控。

**代價**：搜尋成本落在延遲而非儲存空間。已透過三項手段收窄——只掃描當前專案分組且已通過非文字篩選的 session、逐行串流解析不整檔讀入、命中即提前結束該 session 掃描。若日後實測延遲不可接受，決策一保留的升級路徑（指令介面不變、換掉內部實作）仍然成立，但屆時需連同本決策一併重新評估。

### 決策二：內容擷取逐 provider 實作，共用一個 trait-free 分派函式

在 `sessions/mod.rs` 新增 `extract_session_texts(provider: &str, session_dir: &Path) -> Vec<String>`，依 provider 分派至各 `<provider>.rs` 的擷取函式。claude / codex / copilot / antigravity 四者皆為逐行 JSONL 解析，可各自沿用該檔既有的 entry struct；claude 已有 `entry_type == "user" | "assistant"` 且 `is_meta != Some(true)` 的判斷邏輯，內容擷取直接複用同一條件。opencode 走 SQLite 查詢。

**為何不抽共用 JSONL parser**：四個 provider 的 JSON schema 欄位名不同（entry type 欄位、訊息內容欄位皆異），強行抽象只會產生一層需要五個分支的 config，不如各自直起直落。

**只比對對話文字**：工具呼叫參數與檔案內容快照會夾帶大量程式碼與路徑，納入比對會讓幾乎任何關鍵字都命中，使搜尋失去鑑別力。

### 決策三：前端以 debounce + 交集整合，維持 `filterAndSortSessions` 純度

新增 `searchInContent: boolean` state 與 `contentMatchIds: Set<string> | null` state。當 `searchInContent` 為 true 且關鍵字非空時，以 300ms debounce 呼叫指令，結果存入 `contentMatchIds`。`filterAndSortSessions` 新增一個 `contentMatchIds` 參數，關鍵字判斷改為 `matchesTextFields || contentMatchIds?.has(session.id)`——為 `null` 時（未啟用或未回傳）退化為原本的純文字欄位比對，非 `null` 時則擴大命中範圍。函式本身仍是同步純函式。

注意此處是 **OR 而非 AND**：啟用內容搜尋只應擴大關鍵字的命中範圍，不應讓原本靠 summary 命中的 session 因為對話內容沒提到而消失。該關鍵字結果再與 provider、tag、空對話、日期區間以 AND 組合。`contentMatchIds` 為前端記憶體內的暫時狀態，隨元件卸載即釋放，不持久化（見決策七）。

搜尋進行中保留前一次結果、只顯示 loading 指示，避免列表閃爍為空。關閉開關時將 `contentMatchIds` 設回 `null` 並以 request id 或 AbortController 捨棄在途結果，防止舊回應覆寫新狀態。

### 決策四：日期區間作為「更新時間」選單的第四個選項

`SessionUpdatedRange` 由 `"all" | "week" | "month"` 擴充為 `"all" | "week" | "month" | "custom"`。選為 `custom` 時，工具列展開兩個 `<input type="date">`。`getUpdatedRangeStart` 現行回傳單一起始時間戳，需改為回傳 `{ start: number | null; end: number | null }` 以支援上下界。

**為何擴充而非並存兩個時間控制項**：兩個同時生效的時間條件會讓使用者難以判斷實際生效範圍，且在已經變擁擠的工具列再加一個控制項並不划算。作為第四個選項則語意單一——時間篩選永遠只有一個來源。

起訖日期顛倒時顯示提示並維持前一次有效結果，而非套用空區間讓列表無故清空。

### 決策五：空 session 的 state 由 `hideEmptySessions` 更名為 `showEmptySessions`

預設值由「不隱藏」翻轉為「隱藏」，同時 chip 語意由「隱藏」反轉為「顯示」。若只翻轉預設值而保留 `hideEmptySessions` 命名，會得到一個標籤為「空對話」但啟用時反而隱藏空對話的 chip，語意與變數名相反，後續維護必然踩坑。因此連同 `App.tsx` 的 state、prop 與 `onHideEmptySessionsChange` callback 一併更名。

**「空」的定義對齊為 `hasEvents`**：既有的 `session-filter` / `session-list` spec 將空 session 描述為「summary 為空且 summary_count 為 0」，但 `filterAndSortSessions` 實際判斷的是 `!session.hasEvents`。本次 spec delta 將描述對齊為程式碼實情（`hasEvents`）。

需注意 `session-actions` spec 的「刪除空 session（批次）」仍沿用 summary_count 描述——該功能是 `sessions/copilot.rs` 的 `delete_empty_sessions_internal`，是與列表篩選完全獨立的程式碼路徑，本次不修改其行為，因此不為其撰寫 delta。兩處「空」的定義在本次之後仍有落差，屬既有狀況而非本次引入；若日後要統一，應作為獨立的 change 處理。

**無資料遷移風險**：`hideEmptySessions` 是 `App.tsx:251` 的 `useState(false)`，未寫入 settings.json 或 SQLite；`sortKey` 同樣是 `ProjectView` 內的 local state。因此移除排序選項與翻轉預設值都不影響既有使用者資料。（`showArchived` 則確實持久化於 settings，本次只改文案不動預設值與儲存。）

### 決策六：`hiddenCount` 提示語意跟著反轉

現行「隱藏空 session」chip 在啟用時附帶顯示被隱藏的筆數。反轉後預設即為隱藏狀態，提示應改為在 chip **未**啟用時顯示「有 N 筆空對話被隱藏」，讓使用者知道有東西被收起來、可以點開。

## Risks / Trade-offs

- **[大量 session 時內容搜尋延遲明顯]** → 搜尋範圍限縮為當前專案分組且已通過其他同步篩選的 session，而非全域；前端 debounce 300ms 避免逐鍵觸發；顯示 loading 指示讓等待可預期。若日後實測仍不可接受，再評估 FTS 索引（本次刻意保留此升級路徑：指令介面回傳 ID 清單，換成索引實作時前端無需改動）
- **[逐字稿檔案可能極大，全量讀入記憶體有壓力]** → JSONL 採逐行串流解析（`reader.lines()`），不整檔讀入；命中即可提前結束該 session 的掃描
- **[空 session 改為預設隱藏，既有使用者可能誤以為 session 消失]** → chip 未啟用時顯示被隱藏的筆數提示，提供明確的復原路徑
- **[移除 cwd 搜尋後，少數依賴路徑搜尋的使用情境失效]** → 同一專案分組下路徑高度重複，該條件實質上等同無效；跨專案的路徑查找由專案列表本身承擔
- **[工具列改為可換行後，垂直空間在窄視窗下增加]** → 原本的 72px 單行限制本就與新增控制項衝突；換行優於壓縮控制項至不可用寬度。既有的篩選收合（`isFilterExpanded`）機制仍在，使用者可收起整個工具列

## Migration Plan

分兩階段實作，階段一完成即可獨立驗收：

1. **階段一（純前端）**：排序選項移除、搜尋框加寬、移除 cwd 比對、自訂日期區間、chip 文案與預設值翻轉。涉及 `ProjectView.tsx`、`types/index.ts`、`App.tsx`、兩份 locale 與 CSS
2. **階段二（後端 + 前端）**：Rust 內容擷取與 `search_session_content` 指令、command 註冊、前端開關與 debounce 整合

回滾策略：兩階段皆為獨立 commit，可分別 revert。階段二的後端指令若出問題，前端關閉「搜尋對話內容」開關即退回階段一行為。

## Open Questions

- opencode 的 SQLite 訊息內容實際存放於哪張表與哪個欄位（`message` 或 `part`，以及內容是純文字或 JSON blob），需於實作階段讀 `sessions/opencode.rs` 與實機 DB 確認後決定查詢語句
