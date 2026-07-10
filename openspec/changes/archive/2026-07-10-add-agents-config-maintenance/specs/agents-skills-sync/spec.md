## ADDED Requirements

### Requirement: Skills 來源掃描與 per-target 狀態矩陣

系統 SHALL 以 `.agents/skills/`（專案範圍）或 `~/.agents/skills/`（全域範圍）為 skills 主要來源，並額外從各同步目標（claude / codex / opencode / copilot 的 skills 目錄）反向探索目標端已存在的 skill，取兩者的聯集列出所有 skill（含 SKILL.md 的子目錄），對每個目標計算同步狀態，以矩陣呈現。此聯集探索與 agents-commands-sync 的 target 端顯示行為一致，避免來源缺失時整個項目消失。

#### Scenario: 顯示 skills 狀態矩陣

- **WHEN** 使用者開啟 Agents 分頁的 Skills 子分頁
- **THEN** 系統列出來源目錄與各目標目錄的 skill 聯集，每列一個 skill、每欄一個目標 agent
- **AND** 每格顯示該 skill 於該目標的狀態：一致（✓）/ 缺少目標（–）/ 內容不同（≠）/ 僅有目標（–）/ 目標較新（較新!）/ 已連結（🔗）/ 連結失效（⚠）

#### Scenario: 目錄層級狀態判定

- **WHEN** 計算某 skill 對某目標的狀態，且目標非 symlink
- **THEN** 系統逐檔比對來源 skill 目錄與目標 skill 目錄的內容雜湊（雙側皆套用與 agents-md-sync 相同的忽略清單，排除目錄內混入的 node_modules、建置產物等雜訊）
- **AND** 全部檔案存在且雜湊相等 → 一致；目標目錄不存在 → 缺少目標；來源目錄不存在但目標存在 → 僅有目標（source-missing）；任一檔案缺少或不同 → 內容不同

#### Scenario: target 端既有 skill 仍應顯示於矩陣

- **WHEN** `.agents/skills/` 尚未建立或缺少某 skill，但某 target 端（如 `.claude/skills/`、`.opencode/skills/` 或 `.github/skills/`）已存在同名 skill
- **THEN** 系統 SHALL 在矩陣中顯示該 skill 列，該目標狀態標示為「僅有目標」（source-missing），讓使用者可檢視現況並透過衝突流程決定是否回補來源
- **AND** 實際同步時仍以預期來源路徑 `.agents/skills/<name>/` 作為 canonical source path
- **AND** 內容預覽在來源端 SKILL.md 不存在時，改以探索到的目標端 SKILL.md 為預覽來源

#### Scenario: 目標根目錄解析

- **WHEN** 解析各 agent 的 skills 目標目錄
- **THEN** 專案範圍為 `<project>/.claude/skills`、`<project>/.codex/skills`、`<project>/.opencode/skills`、`<project>/.github/skills`
- **AND** 若專案仍使用舊慣例 `<project>/.copilot/skills`，系統 SHALL 相容讀取並優先採用實際存在者
- **AND** 全域範圍為 `~/.claude/skills`、`~/.codex/skills`、`~/.config/opencode/skills`、`~/.copilot/skills`（依設定頁根目錄覆蓋值解析）
- **AND** 目標根目錄不存在時於欄標題標示

#### Scenario: 檢視 SKILL.md

- **WHEN** 使用者點擊矩陣中的 skill 名稱
- **THEN** 右側面板以 markdown 渲染該 skill 的 SKILL.md
- **AND** 提供「在檔案總管顯示」與「以外部編輯器開啟」操作

#### Scenario: 切換頁面後保留 skills 掃描結果

- **WHEN** 使用者已開啟某專案的 Skills 子分頁並完成掃描，之後切換到其他頂層頁面或其他專案，再切回原分頁
- **THEN** 系統優先重用既有快取與 UI 狀態（包含目前分頁、已選列、已展開內容）
- **AND** 未超過 staleTime 時不應重新顯示「找不到任何來源」或重置為初始空狀態

### Requirement: Skills 同步（整目錄複製）

系統 SHALL 支援將勾選的 skills 複製到勾選的目標 agent 目錄（整個 skill 目錄逐檔複製），沿用 dry-run 預覽、衝突詢問與記住選擇機制。v1 不刪除目標端多餘檔案，UI SHALL 註明此限制。

#### Scenario: 同步選取的 skills

- **WHEN** 使用者勾選若干 skills 與目標、點擊「預覽同步」後套用
- **THEN** 系統將每個勾選 skill 的來源目錄逐檔複製到各勾選目標的對應 skill 目錄（必要時建立目錄）
- **AND** 已一致的檔案略過；完成後更新矩陣狀態並顯示摘要

#### Scenario: 目標端 skill 檔較新

- **WHEN** 同步時某目標端檔案 mtime 比來源新且內容不同，且無記住的衝突選擇
- **THEN** 系統依 agents-md-sync 的衝突流程詢問方向，支援記住選擇

#### Scenario: 啟用/停用同步目標

- **WHEN** 使用者於矩陣欄標題取消勾選某 agent 目標
- **THEN** 該目標不參與後續同步
- **AND** 選擇持久化至專案 agents 偏好（enabledTargets）

### Requirement: Skills 連結同步模式

系統 SHALL 提供「連結」同步模式作為「複製」模式之外的可選項：於目標位置建立指向來源 skill 目錄的目錄符號連結（symlink），使目標與來源永遠內容一致，修改來源即時反映到所有連結端，不需重複執行同步。使用者 SHALL 能在 Skills 分頁切換同步模式，預設為複製。

#### Scenario: 選擇連結模式同步

- **WHEN** 使用者將同步模式切換為「連結」並對勾選的 skills／目標執行同步
- **THEN** 系統於各目標路徑嘗試建立指向來源 skill 目錄的目錄 symlink（而非複製檔案）
- **AND** 建立成功後矩陣狀態顯示「已連結」

#### Scenario: 已連結狀態判定免逐檔比對

- **WHEN** 目標路徑是指向對應來源 skill 目錄的 symlink
- **THEN** 系統直接判定為一致（已連結），不逐檔計算雜湊

#### Scenario: 權限不足時自動退回複製

- **WHEN** 建立目錄 symlink 因權限不足失敗（一般使用者未啟用開發者模式或非管理員身分）
- **THEN** 系統自動改以複製模式完成該項目的同步
- **AND** 結果標示為「連結失敗，已改為複製」，並提示使用者可透過啟用 Windows 開發者模式或以系統管理員身分執行以取得連結能力

#### Scenario: 連結指向錯誤來源或既有實體內容

- **WHEN** 目標路徑已是 symlink 但指向不同來源、或目標路徑已存在非 symlink 的實體內容
- **THEN** 系統將其標記為衝突，依衝突流程詢問「以連結取代目標」或「略過」

#### Scenario: 連結失效偵測

- **WHEN** 掃描時發現目標為 symlink 但其指向的來源 skill 目錄已不存在
- **THEN** 系統標示該項為「連結失效」錯誤狀態，不視為一致
- **AND** UI 提示使用者來源已被刪除或搬移，建議移除失效連結或改為複製模式重新同步
