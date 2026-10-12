# Spec Delta

## ADDED Requirements

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
