## ADDED Requirements

### Requirement: Skills 來源掃描與 per-target 狀態矩陣

系統 SHALL 以 `.agents/skills/`（專案範圍）或 `~/.agents/skills/`（全域範圍）為 skills 來源，列出所有 skill（含 SKILL.md 的子目錄），並對每個同步目標（claude / codex / opencode / copilot 的 skills 目錄）計算同步狀態，以矩陣呈現。

#### Scenario: 顯示 skills 狀態矩陣

- **WHEN** 使用者開啟 Agents 分頁的 Skills 子分頁
- **THEN** 系統列出來源目錄下的所有 skill，每列一個 skill、每欄一個目標 agent
- **AND** 每格顯示該 skill 於該目標的狀態：一致（✓）/ 缺少（–）/ 內容不同（≠）/ 目標較新（較新!）

#### Scenario: 目錄層級狀態判定

- **WHEN** 計算某 skill 對某目標的狀態，且目標非 symlink
- **THEN** 系統逐檔比對來源 skill 目錄與目標 skill 目錄的內容雜湊（雙側皆套用與 agents-md-sync 相同的忽略清單，排除目錄內混入的 node_modules、建置產物等雜訊）
- **AND** 全部檔案存在且雜湊相等 → 一致；目標目錄不存在 → 缺少；任一檔案缺少或不同 → 內容不同

#### Scenario: 只列出來源存在的 skill

- **WHEN** 某 target 目錄下存在一個來源已不存在的同名 skill（例如來源被刪除但目標仍保留舊副本）
- **THEN** 系統不為其產生矩陣列，該 skill 對使用者不可見且不參與同步
- **AND** 此為已知限制：v1 不提供「反向探索目標端獨有項目」或鏡像刪除功能

#### Scenario: 目標根目錄解析

- **WHEN** 解析各 agent 的 skills 目標目錄
- **THEN** 專案範圍為 `<project>/.claude/skills`、`<project>/.codex/skills`、`<project>/.opencode/skills`、`<project>/.copilot/skills`
- **AND** 全域範圍為 `~/.claude/skills`、`~/.codex/skills`、`~/.config/opencode/skills`、`~/.copilot/skills`（依設定頁根目錄覆蓋值解析）
- **AND** 目標根目錄不存在時於欄標題標示

#### Scenario: 檢視 SKILL.md

- **WHEN** 使用者點擊矩陣中的 skill 名稱
- **THEN** 右側面板以 markdown 渲染該 skill 的 SKILL.md
- **AND** 提供「在檔案總管顯示」與「以外部編輯器開啟」操作

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
