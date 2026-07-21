# 架構探索：Runtime 與 UI 分離

> 狀態：**探索中（討論記錄，非正式提案）**
> 最後更新：2026-07-21（新增 quota 帳號辨識、跨環境專案聚合、操作分流三層模型、runtime 部署與散布、本機 in-process 修正）
> 目的：記錄「將環境綁定工作抽離為獨立 runtime」的方向與討論脈絡，供後續維護與深化。成熟後再轉為 OpenSpec change。

## 動機

未來開發環境可能從 Windows 原生轉移到 WSL、Docker、遠端 SSH 等不同環境。目前 AI CLI 的 session 資料、hook 事件都產生在各自的環境中，但工具（Tauri app）跑在 Windows，會出現「Provider 在 WSL、工具在 Windows 卻管不到」的落差。

核心構想：把「接收 hook 監聽、掃描 session」這類與環境綁定的工作做成獨立 **runtime**，UI 不再處理這些，變得更輕量，也能同時支援多個環境（本機 / WSL / Docker / SSH）。

## 已拍板的討論前提

- **Runtime 形態**：**遠端**環境（WSL / Docker / SSH）各一個常駐 agent（監聽本地 hook、掃描本地 session，透過網路回報中央）。**本機不起獨立進程**，維持 in-process（見判斷 4）。
- **溝通協定**：核心走 HTTP/WebSocket API；底層保留本地 IPC 作為優化選項。
- **成熟度**：方向尚不完整，暫不建立 OpenSpec proposal，先以本文件累積討論。

## 關鍵架構判斷

### 1. 不是所有工作都沿同一軸切分（最重要）

原始構想是「把環境綁定工作搬進 per-env runtime」，但實際上後端工作分兩類：

| 類別 | 內容 | 綁定對象 | 歸屬 |
|------|------|----------|------|
| **環境本地** | 檔案監聽（watcher）、session 掃描、hook 接收（provider-bridge） | 機器 / 檔案系統 | 每環境一個 runtime |
| **帳號全域** | quota 查詢 | 供應商帳號 / 憑證，**不綁機器** | 必須維持**單例** |

**quota 是最容易做錯的地方**：它跟著帳號走，不跟機器走。若多個環境的 runtime 各自對同一上游帳號輪詢 quota，API 呼叫數會乘上 N 倍。現有程式碼（`src-tauri/src/app_setup.rs` 的 quota poller）刻意「固定每 30 分鐘刷新一次，避免與 Claude Code CLI 撞到 429 限流」；多環境複製會直接打破此保護。

**結論**：quota 需維持單例——留在 UI，或指定某一個 runtime 作為 quota owner，不可每環境各做一份。

#### 單例的粒度是「帳號」，不是「機器」

實際情境未必是「所有環境同一組帳號」。SSH / WSL 不排除會用**不同帳號**登入，此時來源不只一處、quota 結果也真的不同，各來源都該回報並顯示；只有**帳號相同**時才需要去重。因此單例的正確粒度是**每個帳號一份**，而非全域一份。

要做到這件事，quota 需要一個穩定的**帳號身分鍵（`account_id`）**，而現有程式碼今天就已經拿得到、只是沒當 key 用：

- `QuotaSnapshot`（`src-tauri/src/quota/claude.rs`）目前只回傳用量，**不含帳號身分**——這是要補的欄位。
- 身分來源（優先序）：OAuth `access_token` 本身是 JWT，本地即可解出 `sub`（account id）/ email，**不需打 API**；其次是 usage API 回應中的帳號 / organization 識別；再者是憑證檔內的 email / `sub`。雜湊後作為穩定鍵。

去重流程（此順序可成立，是本判斷的關鍵）：

1. 各 runtime **本地解 token 的 `sub`**，得出各自的 `account_id`（此步不打 API）。
2. 中央用 `account_id` 分組：
   - 相同 → 同一帳號 → 只指定**一個 owner**（某 runtime 或本地）去打 usage API，維持 30 分鐘防 429 節流。
   - 不同 → 不同帳號 → 各自的 quota 都保留、都顯示。

即「先本地辨識帳號 → 中央決定 owner → 只有 owner 打 API」，避免「要打 API 才知道帳號」的雞生蛋問題。

### 1b. 跨環境同專案聚合：以 git remote 為身分鍵

同一個專案可能在兩個環境開發（同一人不同機器 / 裝置，或先 Windows 後改 WSL），兩邊都有 session 資料。理想呈現是**收進同一個專案卡、用來源標籤區分**，token 用量與 session 資訊就能跨環境合計。

專案身分鍵已經現成——`src-tauri/src/sessions/git.rs` 目前已在跑 `git config --get remote.origin.url`，只是拿來解 repo 顯示名。**normalize 後的 remote URL 正是跨環境專案身分鍵**：同一專案不論本機路徑是 `D:\ching\...` 還是 `/home/...`，只要 remote 指向同一 git 遠端就是同一專案。

- **分組 key**：normalize 過的 `remote.origin.url`（去 `.git`、統一 ssh/https、小寫），取代目前用本機路徑分組。
- **資料形狀**：`SessionInfo` 已有 `repo_root` / `repo_name`，只需再加 `repo_remote`（normalize 後）當聚合 key，前端即可分組。改動小。
- **無 remote 的專案**（純本機、未 push）：fallback 回本機路徑，這類本就只存在一處、無法也無需跨環境聚合。

#### 斷線 / 遺失情境——本判斷最需設計的部分

跨環境聚合最大的難點是「開啟功能可能失敗、SSH 連不上、session 遺失」。設計原則：**「看」永遠可行（靠快取），「操作」才需要來源在線。**

- **離線存活**：中央將各 runtime 回報的 session **快取於本地 sqlite（現有 `db.rs` 已具此能力）**。WSL 關閉 / SSH 斷線時，該批 session 仍以「最後已知狀態」顯示，不消失。
- **來源狀態標記**：每個來源標籤帶 online / offline / stale 狀態。

詳細的看 / 操作分界見下方判斷 2b。

### 2b. 操作類動作的執行模型：分流三層，非二選一

（缺口 2 的收斂）操作類動作不一定要 RPC 到遠端 runtime 執行——很多時候 **UI 本機用一條連線指令即可直達目標位置開啟**。存在兩種執行模型：

- **模型甲（本機連線指令直達）**：UI 在本機組命令穿進去。WSL → `wsl -d <distro> --cd <path> <cmd>`；Docker → `docker exec -w <path> <container> <cmd>`；SSH → `ssh user@host -t 'cd <path> && <cmd>'`（憑證在就直通，沒了則跳密碼由使用者當場輸）。
- **模型乙（RPC 到 runtime）**：UI 送請求給該環境 runtime，由 runtime 在它自己那端 `Command::new` 執行。

**模型甲通常更好**：不依賴 runtime 在線（只要環境本身可連即可開終端），且複用現有機制——目前 `src-tauri/src/commands/tools.rs` 等已在本機 `Command::new` 開終端，模型甲只是**把命令前綴換成 wsl/docker/ssh**。

分流原則：

| 動作類型 | 走哪個模型 | 依賴 |
|---------|-----------|------|
| 開終端 / 開編輯器 / 前景 git | **模型甲**（本機連線指令直達） | 連線可達 + 憑證在（或當場輸密碼） |
| 即時 tail session / 觸發重掃 / 需 runtime 內部狀態 | **模型乙**（RPC 到 runtime） | runtime 在線 |
| 唯讀看歷史 / token 用量 | 都不用，靠本地快取 | 無 |

因此「看 / 操作分界」實為**三層**：**「看」靠快取；「開東西」靠連線層（獨立於 runtime）；「要 runtime 內部狀態的操作」才需 runtime 在線。**

要點與邊界：

- **兩個獨立狀態**：連線可達性（能否開終端）與 runtime 在線性（session 資料新不新）須分開判斷——可能「WSL 開著能開終端，但 runtime 進程掛了、資料是 stale 快取」。
- **路徑轉換逃不掉**：session 的 `cwd` 是目標端格式（`/home/...`），組指令時須用該端路徑格式（兩模型皆然）。
- **SSH 密碼互動**：前景動作（開終端）可把使用者直接丟進終端自行輸密碼；**背景動作**（如默默跑 git status）沒地方輸密碼，實務上要求「憑證已備妥（key/agent）」。
- **可先於 runtime 獨立實作**：模型甲就是「把現有 `Command::new` 前綴換成 wsl/docker/ssh」，**今天的單機架構即可加**，甚至在 runtime 計畫啟動前它本身就是有用的小功能（在 UI 直接開 WSL 專案終端）。

### 2. 真正的工作量在 `AppHandle` 耦合，不在掃描邏輯

- 掃描邏輯已幾乎 runtime-ready：`get_sessions_internal`（`src-tauri/src/sessions/mod.rs`）已只依賴 `Connection` + `ScanCache`，回傳 `Vec<SessionInfo>`（serde 型別，今天就已穿過 Tauri IPC 邊界）。線材格式基本免費。
- 真正耦合的是 **watcher** 與 **tray/quota**：每個 watcher 都持有 `&tauri::AppHandle`，直接呼叫 `emit_provider_refresh(&handle, …)`（`src-tauri/src/watcher.rs`）。

**遷移核心** = 把「emit 到 Tauri 前端」這個 sink，換成一個 event-bus / broadcast 抽象，讓 HTTP/WS 層去訂閱。這是主要成本。

### 3. 可大幅縮小範圍的簡化：runtime 緊貼 CLI

hook 接收機制（`watcher.rs` 的 provider-bridge）是**檔案監聽式**。只要 runtime 跑在每個環境內、緊貼 CLI，現有 watcher 完全不用改——**不需要**把 hook 改成網路呼叫。

- CLI → runtime：維持檔案式（不變）
- runtime → UI：唯一需要新傳輸的 hop（HTTP/WS）

網路邊界只有一跳，範圍顯著縮小。

### 4. 本機不需要獨立 runtime 進程——in-process 即可

「本機要不要 runtime」其實是兩個被混在一起的問題，要拆開：

1. **本機掃描邏輯要不要抽到 core crate？** → **一定要**（見判斷 2、Phase 1）。這與本機跑不跑獨立進程無關。
2. **本機這包邏輯用「獨立進程 + HTTP/WS」還是「in-process 函式呼叫」跑？** → **用 in-process。**

本機硬拆獨立進程的代價（KISS/YAGNI 反對）：多一個會孤兒化 / 殘留 / 佔埠 / 關 App 沒關乾淨的進程；使用者工作管理員多一個看不懂的東西；本機純屬自找的 IPC/序列化開銷；憑空多出「進程沒起來 / 埠被佔 / 握手失敗」等本來不存在的失敗模式。

**「本機也當成 localhost 上的一個 runtime」的本意是「不讓兩份掃描邏輯分裂」，不是「非得起進程」。** 用抽象介面即可達成，不需進程：

```
trait SessionSource {              // UI 只認這個介面
    fn get_sessions(...) -> Vec<SessionInfo>;
    fn subscribe(...) -> EventStream;
}

LocalSource   → 直接 in-process 呼叫 core crate（本機，零進程，最快最可靠）
RemoteSource  → 走 HTTP/WS 連遠端 runtime
```

邏輯同為一份 core crate（不分裂），但本機不付進程代價。**不分裂靠 `SessionSource` 抽象 + 共用 crate，非靠「本機也起進程」。**

**唯一該讓本機起獨立進程的情況**：希望 session 監聽 / 掃描在 App 關掉後仍持續（App 沒開時也累積 hook 事件 / 算 token）。以目前使用情境（開著 App 才看 session），本機 in-process 是最優解；獨立進程只在遠端才必要（那些環境的 CLI 本就在 App 之外獨立運作，非有常駐者去接不可）。

（此判斷修正了 Phase 3 舊述「本機也當 localhost runtime」，見下方分階段路徑。）

## 建議的分階段路徑

可搭上進行中的 `split-large-rust-modules` 重構作為墊腳石：

1. **Phase 1（純重構，零行為變化）**：把 watch + scan + db + quota 抽成 library crate，Tauri app in-process 消費。風險最低，是後兩階段的地基。
2. **Phase 2**：headless binary，在該 crate 上開 HTTP/WS 服務。
3. **Phase 3**：Tauri 加入「remote runtime」模式。**關鍵：本機與遠端都走同一個 `SessionSource` 介面**（本機 `LocalSource` in-process、遠端 `RemoteSource` 走 HTTP/WS），避免兩份實作分裂。注意——「走同一介面」不等於「本機也起進程」；本機仍 in-process，見判斷 4。

## Runtime 部署與散布（缺口 1 的收斂）

「runtime 進程怎麼跑到目標環境、怎麼活著」——這是傳輸（SSH tunnel）沒回答的部分。關鍵在把**散布（binary 從哪來）**與**存活（進程怎麼起、怎麼不死）**分開，並沿「本機 vs 遠端」切開：

| 目標 | 執行模型 | 散布方式 | 存活方式 | 時機 |
|------|---------|---------|---------|------|
| **本機** | core crate **in-process**（`LocalSource`，不起進程，見判斷 4） | 隨 App（本機根本不需散布） | App 生命週期內 | 較早、單純 |
| **WSL/Docker/SSH** | 獨立 runtime 進程 + HTTP/WS（`RemoteSource`） | runtime 獨立打包 → 由 App 經 SSH 推送、`chmod +x` | systemd user service / nohup / tmux | 真要上遠端時 |
| **獨立 repo** | 同上 | `git clone` + build / 下載 release | 同上 | 開源 / 給他人用才抽，是**結果非起點** |

**前提約束（從第一天守）**：runtime crate **絕對不能 `use` 任何 Tauri 東西**（`AppHandle` / `Emitter` 等）。只要相依 Tauri 就永遠抽不出去、也獨立打包不了。故「現在不抽 repo」不等於「可以偷懶耦合」——反而要在**同一 workspace 內用 crate 邊界強制隔離**：

- `sessionhub-core`（runtime crate）：watch + scan + db + quota + HTTP/WS 殼，**零 Tauri 依賴**。
- Tauri app：本機 in-process 依賴 core crate，遠端經 HTTP/WS 連 core。

如此「本機 in-process 的邏輯」與「推到 WSL 的 runtime」**是同一份 crate 編出來的**，只差散布路徑，避免兩份實作分裂。

散布方式評估備忘：

- **App 內安裝（本機 / 複用 hook 內嵌模式）**：現有 hook 安裝已是成熟的「`include_str!` 內嵌 → `fs::write` 到目標 → 掛設定 → `.version` 版控」模式（`src-tauri/src/provider/claude.rs`）。但 hook 是「觸發即結束」的腳本，runtime 是常駐進程，多出「存活」問題須另解。
- **npm 包**：跨平台 / 版控（`npm update`）天生免費，但**適合 JS/TS runtime**；本專案 runtime 是 Rust，走 npm 等於用 napi-rs 包 native addon（複雜）或用 JS 重寫（丟掉 Rust core）——與「抽 Rust core crate」方向衝突，**勸退**。
- **跨平台 cross-compile**：完整方案（App 推 Linux/其他 arch 的 runtime）需為各目標平台 cross-compile 並打包，是遠端散布的主要成本，留待 Phase 3。

## 安全性

先不自造 auth / token 層（呼應 KISS / YAGNI）：

- Runtime 綁 `127.0.0.1`。
- 遠端（WSL / SSH / Docker）一律走 **SSH tunnel** 轉發。

此策略即涵蓋所有遠端情境，又不需發明認證系統。待真有跨網段需求再擴充。

## 待處理 / 開放問題

- **Session ID 命名空間**：多 runtime 後，session ID 需加 runtime 前綴 + 顯示來源，否則跨環境會撞 ID。（Phase 3）
- **設定拆分**：per-runtime（掃哪些 root）vs per-UI（overlay 位置、顯示偏好）需分離。（Phase 3）
- **Runtime 生命週期 / 發現 / 註冊**：散布已收斂（見上），但「UI 如何發現並註冊一個 runtime、重開機後是否自動復活、如何更新」仍待設計。（Phase 2/3）
- **協定 / 版本偏移**：獨立部署 → runtime 與 UI 必然版本漂移（舊 WSL agent vs 新 UI）。需版本握手。（Phase 2 就要有）
- **快取 / cursor 的權威位置**：增量掃描的 cursor 必須跟著「誰掃描」走——各 runtime 應各自持有其 `db.rs` + `ScanCache` cursor，UI 端持聚合視圖。此歸屬需寫明。（Phase 2）
- **時鐘偏移**：跨 runtime 以 `updated_at` 排序，各機時鐘不同會排錯；單機內 `mtime` 增量無虞，跨 runtime 合併排序才有。（Phase 3）
- **值不值得做 / 優先級**：若短期主力仍在 Windows，此方向「優雅但不急」，可先記錄、待實際遷入 WSL 再啟動（YAGNI）。

已收斂（移入上方判斷段落，保留於此供追蹤）：

- ~~多 runtime 的 UI 心智模型~~ → 見 [1b] 以 git remote 聚合、來源標籤區分。
- ~~離線 / 斷線情境~~ → 見 [1b] 快取離線存活；[2b] 操作分流三層。
- ~~quota 多帳號~~ → 見 [1] account_id 粒度單例。
- ~~操作類動作如何在遠端執行~~ → 見 [2b] 模型甲（連線直達）/ 模型乙（RPC）分流。
- ~~本機是否需要 runtime 進程~~ → 見 [4] 否，in-process；用 `SessionSource` 抽象避免分裂。
- ~~runtime 如何部署 / 散布~~ → 見「Runtime 部署與散布」；本機 App 內裝、遠端獨立打包，零 Tauri 依賴。

## 前端影響（尚未盤點）

前端 `listen` 事件清單（`provider-refresh`、`quota-snapshots-updated` 等）在改為訂閱 event-bus / WS 後需重新對應，屬設計階段細節，此處先不展開。
