# agents-skills-sync

## Purpose

定義 Agents Skills 的掃描描述、來源與同步相關行為，以及全域自訂正本與預設 `.agents` 位置的連結狀態，讓使用者確認技能來源、理解部署狀態，並保留既有實體內容。

## Requirements

### Requirement: 掃描結果包含 skill 描述

Skills 掃描 SHALL 為每個 skill 抽取描述並隨掃描結果回傳（`SkillEntry` 新增 `description` 欄位，camelCase serde）：描述取自該 skill 預覽來源 `SKILL.md` 的 YAML frontmatter `description` 欄位（沿用既有 frontmatter 解析工具）；檔案不存在、無 frontmatter 或無該欄位時 SHALL 回傳空值而非錯誤，且不影響該 skill 其餘掃描結果。

#### Scenario: 有描述的 skill

- **WHEN** 某 skill 的 `SKILL.md` frontmatter 含 `description` 欄位
- **THEN** 掃描結果中該 skill 的 `description` 為該欄位值

#### Scenario: 無描述不視為錯誤

- **WHEN** 某 skill 的 `SKILL.md` 無 frontmatter 或無 `description` 欄位
- **THEN** 該 skill 的 `description` 為空值，狀態矩陣與其餘欄位照常回傳

### Requirement: 全域自訂正本位置的 ~/.agents 連結狀態判定

全域範圍設定了自訂正本位置（≠ `~/.agents`）時，系統 SHALL 檢查 `~/.agents` 與正本的連結關係並回報下列狀態之一，於 Skills 子分頁以 banner 呈現；未設定自訂位置時 SHALL NOT 顯示此 banner：

- `linked`：`~/.agents` 整層是指向正本的 symlink；或 `~/.agents` 為實體目錄，且正本第一層每個子項目在 `~/.agents` 均有同名項目、該項目為解析到正本對應路徑的 symlink（逐項等效）
- `partial`：`~/.agents` 為實體目錄，正本第一層子項目僅部分有對應的正本 symlink（其餘缺漏、為實體副本或指向他處）
- `unlinked-physical`：`~/.agents` 為實體目錄，與正本無任何 symlink 對應
- `not-linked`：`~/.agents` 是 symlink 但指向非正本位置
- `missing`：`~/.agents` 不存在

實體目錄情境（`partial`、`unlinked-physical`）SHALL 以資訊性提示呈現，SHALL NOT 使用阻擋性錯誤語氣或要求使用者手動合併後整層連結。

#### Scenario: 整層 symlink 已連結

- **WHEN** `~/.agents` 是指向自訂正本位置的目錄 symlink
- **THEN** 狀態為 `linked`，banner 顯示已連結徽章

#### Scenario: 實體目錄且逐項連結等效

- **WHEN** `~/.agents` 為實體目錄，且正本第一層每個子項目（目錄與檔案）在 `~/.agents` 都有同名的 symlink 且解析到正本對應路徑
- **THEN** 狀態為 `linked`，banner 顯示已連結徽章，不要求整層重建連結

#### Scenario: 實體目錄且部分子項目已連結

- **WHEN** `~/.agents` 為實體目錄，正本第一層子項目僅部分在 `~/.agents` 有對應的正本 symlink
- **THEN** 狀態為 `partial`，banner 以資訊性提示顯示「部分子項目已連結至正本」並列出未對應的項目名稱
- **AND** 不提供整層「建立連結」自動化操作

#### Scenario: 實體目錄且與正本無關聯

- **WHEN** `~/.agents` 為實體目錄，其內容與正本無任何 symlink 對應
- **THEN** 狀態為 `unlinked-physical`，banner 以資訊性提示說明 `~/.agents` 為實體目錄、原生讀取 `.agents` 的工具將讀到實體內容而非正本
- **AND** 不顯示「無法自動連結、請先手動合併」等錯誤文案，不提供整層「建立連結」操作

#### Scenario: 僅於 ~/.agents 不存在時提供建立連結

- **WHEN** `~/.agents` 不存在（狀態 `missing`），使用者點擊「建立連結」
- **THEN** 系統建立 `~/.agents` → 自訂正本位置的目錄 symlink，成功後狀態轉為 `linked`

#### Scenario: 非 missing 狀態呼叫建立連結被拒絕

- **WHEN** 狀態為 `partial`、`unlinked-physical` 或 `not-linked` 時呼叫 `link_agents_root`
- **THEN** 系統不覆蓋、不搬移任何既有內容，回傳明確錯誤訊息

#### Scenario: symlink 指向他處

- **WHEN** `~/.agents` 是 symlink 但解析結果非自訂正本位置
- **THEN** 狀態為 `not-linked`，banner 維持現行提示要求使用者確認連結目標

#### Scenario: symlink 權限不足

- **WHEN** 於 `missing` 狀態建立 `~/.agents` symlink 因權限不足失敗
- **THEN** 系統提示可啟用 Windows 開發者模式或以系統管理員身分執行；不自動改以複製佈署

### Requirement: 探索專案內子目錄技能入口

專案 Skills 掃描 SHALL 探索專案根目錄及其實體子目錄中的已支援 provider 技能入口，包含 `.agents/skills`，並保留入口內既有巢狀技能探索。探索 SHALL 依已啟用 provider 的原有路徑規則取得資源，不將掃描到的檔案自動宣稱為 CLI 已載入。多個 provider 共用同一上下文的技能入口時 SHALL 共用該次入口遍歷結果並保留 provider 歸屬。

#### Scenario: 子專案技能可見
- **WHEN** 專案根目錄與 `packages/api` 各有 `.agents/skills`，且啟用了讀取此入口的 provider
- **THEN** 一次專案 Skills 掃描可列出兩處技能與其實際來源路徑

#### Scenario: 深層子專案不因固定深度漏掃
- **WHEN** 未被忽略的子專案位於超過八層的實體目錄中，且總探索目錄數未達限制
- **THEN** 該子專案的 `.agents/skills` 仍可被探索

#### Scenario: 技能入口內的巢狀名稱
- **WHEN** 子專案存在 `.agents/skills/team/demo/SKILL.md`
- **THEN** 找到該技能，保留相對名稱 `team/demo` 與所屬子專案

#### Scenario: 共用入口不重複遍歷
- **WHEN** 同一上下文的 `.agents/skills` 同時適用於兩個已啟用 provider
- **THEN** 該次掃描只遍歷此入口一次，兩個 provider 的技能歸屬仍可辨識

### Requirement: 子專案同名技能與同步位置隔離

系統 SHALL 以所屬子專案上下文與技能相對名稱辨識技能，在清單、搜尋、預覽、勾選與同步中區分不同子專案的同名技能。同一上下文的既有跨 provider 分組 SHALL 保持。子專案技能的正本與同步目標 SHALL 以該子專案為基準，不以顯示標籤推導目標，不將它部署到專案根目錄的技能入口；可同步條件 SHALL 依該列實際目標入口判定。

#### Scenario: 兩個子專案有同名技能
- **WHEN** `packages/api/.agents/skills/review` 與 `packages/web/.agents/skills/review` 各含 `SKILL.md`
- **THEN** 兩個技能可分別辨識及選取，預覽各自的檔案，不合併為同一列

#### Scenario: 同步子專案技能
- **WHEN** 使用者選擇 `packages/api` 的 `review` 技能同步到 Claude
- **THEN** 來源為 `packages/api/.agents/skills/review`，目標為 `packages/api/.claude/skills/review`
- **AND** 不寫入根目錄 `.claude/skills/review` 或 `.claude/skills/packages/api/review`

#### Scenario: 根目錄目標不存在但子專案目標存在
- **WHEN** 根目錄 `.claude/skills` 不存在，但子專案 `.claude/skills` 存在
- **THEN** 子專案該列仍依自己的目標狀態提供同步，不被根目錄狀態錯誤停用

### Requirement: 專案技能探索剪枝與部分結果診斷

探索 SHALL 跳過既有大型相依套件、版本控制與建置輸出排除目錄及使用者忽略路徑，且不讀取一般專案檔案內容。一般專案子目錄的 symlink／junction SHALL NOT 作為擴展搜尋入口；已找到的技能入口及技能目錄的連結 SHALL 保持可探索。總探索超過 20,000 個有效專案目錄，或部分路徑無法讀取、技能連結損壞／循環時，系統 SHALL 保留其餘已找到結果並提供不完整或失敗路徑的診斷，不靜默宣稱完整掃描。

部分結果及其診斷 SHALL 視為可快取的成功結果，清單上方 SHALL 持續呈現不完整狀態或失敗路徑與原因，不僅記錄於 log 或短暫提示。只有無法開始探索的根目錄或設定錯誤 SHALL 視為整體偵測失敗。

#### Scenario: 忽略整棵大型子樹
- **WHEN** `node_modules`、`.git`、`target` 或使用者忽略目錄內存在 `.agents/skills`
- **THEN** 不進入該子樹探索，不將其中技能列入專案結果

#### Scenario: 連結技能仍可讀取
- **WHEN** 實體子專案的 `.agents/skills` 或其中的技能目錄連結至集中管理位置
- **THEN** 可探索連結後的技能，並保留該子專案的來源入口與同步上下文

#### Scenario: 一般連結目錄不擴大搜尋
- **WHEN** 專案某一般子目錄連結到外部專案或形成迴圈
- **THEN** 不沿著該一般子目錄擴展搜尋，掃描可終止且不混入外部專案技能

#### Scenario: 限制或錯誤不靜默漏掃
- **WHEN** 探索達到總目錄上限、部分目錄無權限或技能連結損壞
- **THEN** 回傳已找到的技能並提示掃描不完整或失敗路徑，已發現的其他可讀入口仍完成處理

#### Scenario: 部分結果再次進入仍有提示
- **WHEN** 技能掃描回傳部分結果與診斷，使用者離開後再次進入
- **THEN** 直接顯示快取技能與持續可見的診斷提示，不重新掃描或隱藏不完整狀態

### Requirement: 專案技能依上下文漸進載入

專案 Skills SHALL 優先呈現根目錄上下文的技能，再背景補齊子專案上下文。每批顯示的技能 SHALL 已完成既有描述、內容指標與來源／目標同步狀態判定；合併 SHALL 保留上下文、相對名稱與 provider 身分，並維持既有探索、連結、忽略與診斷規則。

#### Scenario: 根層與子專案同名技能
- **WHEN** 根目錄與 packages/api 均有 review 技能
- **THEN** 根層完成後先顯示根目錄 review，後續加入 packages/api 的 review
- **AND** 兩者保持不同預覽與同步來源／目標，不重複或合併為一筆

#### Scenario: 深層與連結技能
- **WHEN** 未忽略的深層子專案具有技能入口或技能連結
- **THEN** 後續掃描仍依原規則找到技能，不因分層新增深度限制或跟隨一般子專案連結

#### Scenario: CLI 確認不阻擋根層
- **WHEN** 根目錄技能可由檔案確認，但 CLI 探索仍需等待
- **THEN** 先顯示檔案同步狀態，背景僅探索 CLI 一次
- **AND** 後續補齊根層及子目錄技能的 CLI 資訊，不重新讀取根層技能指紋，也不提前宣稱 CLI 已載入

#### Scenario: 未完成與不完整診斷
- **WHEN** 根層已完成而子目錄仍在掃描，或子目錄探索達到預算
- **THEN** 畫面分別顯示背景補齊或持續的不完整診斷，不將部分清單表示為完整結果
