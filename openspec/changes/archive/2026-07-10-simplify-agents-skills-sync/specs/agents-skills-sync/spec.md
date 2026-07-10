## MODIFIED Requirements

### Requirement: Skills 來源掃描與 per-target 狀態矩陣

系統 SHALL 以 canonical source（專案範圍為 `<project>/.agents/skills/`；全域範圍為設定頁自訂正本位置，未設定時為 `~/.agents/skills/`）為 skills 正本，並從 `.claude/skills`（專案）／`~/.claude/skills`（全域）反向探索僅存在於 claude 端的 skill，取聯集列出所有 skill。矩陣欄位 SHALL 固定為 **agents** 與 **claude** 兩欄：agents 欄呈現正本狀態，claude 欄呈現 claude 端相對正本的同步狀態。系統 SHALL NOT 掃描或顯示 codex / opencode / copilot 的 skills 目錄（該三者原生讀取 `.agents/skills`，無需同步）。

#### Scenario: 顯示 skills 狀態矩陣

- **WHEN** 使用者開啟 Agents 分頁的 Skills 子分頁
- **THEN** 系統列出正本目錄與 claude 目錄的 skill 聯集，每列一個 skill，欄位為 agents 與 claude 兩欄
- **AND** agents 欄顯示：正本（skill 存在於正本）／未收錄（僅存在 claude 端）；全域自訂正本位置時改顯示 `~/.agents` 端相對正本的同步狀態（已連結／一致／內容不同／未安裝）
- **AND** claude 欄顯示該 skill 於 claude 端的狀態：一致／未安裝／內容不同／僅存此端／目標較新（較新!）／已連結／連結失效

#### Scenario: 表頭顯示原生相容說明

- **WHEN** 使用者檢視 Skills 子分頁
- **THEN** 表頭區域顯示「.agents 原生相容：codex / opencode / copilot（無需同步）」說明，標明僅 Claude Code 需同步至 `.claude/skills`

#### Scenario: 欄標題路徑提示與開啟目錄

- **WHEN** 使用者將滑鼠停留於 agents 或 claude 欄標題
- **THEN** 系統以 tooltip 顯示該欄根目錄的完整路徑
- **AND** 點擊欄標題名稱時，系統於檔案總管開啟該目錄；目錄不存在時不可點擊並於 tooltip 註明

#### Scenario: 目錄層級狀態判定

- **WHEN** 計算某 skill 對 claude 端（或全域自訂正本時的 `~/.agents` 端）的狀態，且該端非 symlink
- **THEN** 系統逐檔比對正本 skill 目錄與該端 skill 目錄的內容雜湊（雙側皆套用與 agents-md-sync 相同的忽略清單）
- **AND** 全部檔案存在且雜湊相等 → 一致；該端目錄不存在 → 未安裝；正本目錄不存在但該端存在 → 僅存此端（source-missing）；任一檔案缺少或不同 → 內容不同

#### Scenario: claude 端既有 skill 仍應顯示於矩陣

- **WHEN** 正本尚未建立或缺少某 skill，但 claude 端已存在同名 skill
- **THEN** 系統 SHALL 在矩陣中顯示該 skill 列，agents 欄標示「未收錄」、claude 欄標示「僅存此端」（source-missing），讓使用者可檢視現況並透過衝突流程決定是否回補正本
- **AND** 實際同步時仍以 canonical source 路徑作為正本路徑
- **AND** 內容預覽在正本 SKILL.md 不存在時，改以 claude 端 SKILL.md 為預覽來源

#### Scenario: 目標根目錄解析

- **WHEN** 解析 skills 的掃描與同步根目錄
- **THEN** 專案範圍：正本為 `<project>/.agents/skills`、同步目標為 `<project>/.claude/skills`
- **AND** 全域範圍未自訂正本位置：正本為 `~/.agents/skills`、同步目標為 `~/.claude/skills`（依設定頁 claude 根目錄覆蓋值解析）
- **AND** 全域範圍已自訂正本位置：正本為 `<自訂位置>/skills`、同步目標為 `~/.agents/skills` 與 `~/.claude/skills` 兩者
- **AND** 目標根目錄不存在時於欄標題標示

#### Scenario: 檢視 SKILL.md

- **WHEN** 使用者點擊矩陣中的 skill 名稱
- **THEN** 系統以整頁詳情模式渲染該 skill 的 SKILL.md
- **AND** 提供「在檔案總管顯示」與「以外部編輯器開啟」操作

#### Scenario: 切換頁面後保留 skills 掃描結果

- **WHEN** 使用者已開啟某專案的 Skills 子分頁並完成掃描，之後切換到其他頂層頁面或其他專案，再切回原分頁
- **THEN** 系統優先重用既有快取與 UI 狀態（包含目前分頁、已選列、已展開內容）
- **AND** 未超過 staleTime 時不應重新顯示「找不到任何來源」或重置為初始空狀態

### Requirement: Skills 同步（整目錄複製）

系統 SHALL 支援將勾選的 skills 同步至勾選的目標（claude 端；全域自訂正本位置時另含 `~/.agents` 端），沿用 dry-run 預覽、衝突詢問與記住選擇機制。v1 不刪除目標端多餘檔案，UI SHALL 註明此限制。既有偏好（enabledTargets）中的 codex / opencode / copilot 值 SHALL 於讀取時忽略、寫回時不再保留。

#### Scenario: 同步選取的 skills

- **WHEN** 使用者勾選若干 skills 與目標、點擊「預覽同步」後套用
- **THEN** 系統將每個勾選 skill 的正本目錄同步到各勾選目標的對應 skill 目錄（必要時建立目錄）
- **AND** 已一致的檔案略過；完成後更新矩陣狀態並顯示摘要

#### Scenario: 目標端 skill 檔較新

- **WHEN** 同步時某目標端檔案 mtime 比正本新且內容不同，且無記住的衝突選擇
- **THEN** 系統依 agents-md-sync 的衝突流程詢問方向，支援記住選擇

#### Scenario: 啟用/停用同步目標

- **WHEN** 使用者於矩陣欄標題取消勾選 claude 目標
- **THEN** 該目標不參與後續同步
- **AND** 選擇持久化至專案 agents 偏好（enabledTargets），持久化值僅含 agents / claude

### Requirement: Skills 連結同步模式

系統 SHALL 提供「連結」同步模式作為「複製」模式之外的可選項：於目標位置建立指向正本 skill 目錄的目錄符號連結（symlink），使目標與正本永遠內容一致。使用者 SHALL 能在 Skills 分頁切換同步模式，**預設為連結**。

#### Scenario: 選擇連結模式同步

- **WHEN** 使用者以「連結」模式對勾選的 skills／目標執行同步
- **THEN** 系統於各目標路徑嘗試建立指向正本 skill 目錄的目錄 symlink（而非複製檔案）
- **AND** 建立成功後矩陣狀態顯示「已連結」

#### Scenario: 已連結狀態判定免逐檔比對

- **WHEN** 目標路徑是指向對應正本 skill 目錄的 symlink
- **THEN** 系統直接判定為一致（已連結），不逐檔計算雜湊

#### Scenario: 權限不足時自動退回複製

- **WHEN** 建立目錄 symlink 因權限不足失敗（一般使用者未啟用開發者模式或非管理員身分）
- **THEN** 系統自動改以複製模式完成該項目的同步
- **AND** 結果標示為「連結失敗，已改為複製」，並提示使用者可透過啟用 Windows 開發者模式或以系統管理員身分執行以取得連結能力

#### Scenario: 連結指向錯誤來源或既有實體內容

- **WHEN** 目標路徑已是 symlink 但指向不同來源、或目標路徑已存在非 symlink 的實體內容
- **THEN** 系統將其標記為衝突，依衝突流程詢問「以連結取代目標」或「略過」

#### Scenario: 連結失效偵測

- **WHEN** 掃描時發現目標為 symlink 但其指向的正本 skill 目錄已不存在
- **THEN** 系統標示該項為「連結失效」錯誤狀態，不視為一致
- **AND** UI 提示使用者正本已被刪除或搬移，建議移除失效連結或改為複製模式重新同步

## ADDED Requirements

### Requirement: 全域自訂正本位置的 ~/.agents 連結檢查

全域範圍設定了自訂正本位置（≠ `~/.agents`）時，系統 SHALL 檢查 `~/.agents` 是否為指向自訂位置的 symlink，並於 Skills 子分頁以 banner 呈現檢查結果與必要操作。未設定自訂位置時 SHALL NOT 顯示此 banner。

#### Scenario: 尚未建立連結

- **WHEN** 全域 scope 已設定自訂正本位置，且 `~/.agents` 不存在或不是指向該位置的 symlink
- **THEN** banner 顯示「~/.agents 尚未連結至自訂正本位置」與「建立連結」按鈕
- **AND** 點擊後系統建立 `~/.agents` → 自訂位置的目錄 symlink，成功後 banner 轉為已連結狀態

#### Scenario: ~/.agents 已存在實體內容

- **WHEN** 使用者點擊「建立連結」但 `~/.agents` 已是含實體內容的一般目錄
- **THEN** 系統不覆蓋、不搬移，回報衝突並提示使用者先手動合併內容後再建立連結

#### Scenario: 已正確連結

- **WHEN** `~/.agents` 是指向自訂正本位置的 symlink
- **THEN** banner 顯示已連結狀態；矩陣 agents 欄各 skill 呈現「已連結」

#### Scenario: symlink 權限不足

- **WHEN** 建立 `~/.agents` symlink 因權限不足失敗
- **THEN** 系統提示可啟用 Windows 開發者模式或以系統管理員身分執行；不自動改以複製佈署
