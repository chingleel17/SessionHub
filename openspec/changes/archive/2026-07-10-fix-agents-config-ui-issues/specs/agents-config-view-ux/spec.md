# agents-config-view-ux

## ADDED Requirements

### Requirement: 頁籤切換時清除操作狀態
Agents 設定頁在 AGENTS.md / Skills / Commands 頁籤之間切換時，SHALL 清除同步預覽報告（syncReport）、報告勾選項（selectedActionKeys）、內容預覽選取（selectedNode）與已載入的內容，確保各頁籤的操作狀態互不殘留。

#### Scenario: Skills 產生報告後切換至 AGENTS.md
- **WHEN** 使用者在 Skills 頁籤執行「預覽同步」產生報告後切換至 AGENTS.md 頁籤
- **THEN** 同步預覽報告區塊不再顯示，AGENTS.md 頁籤僅呈現其樹狀檢視與內容

#### Scenario: 開啟內容預覽後切換頁籤再切回
- **WHEN** 使用者在 Skills 頁籤點選項目開啟內容預覽，切換至 Commands 頁籤
- **THEN** Commands 頁籤不顯示 Skills 的預覽內容，需重新點選項目才會顯示預覽

### Requirement: 對話框疊層高於頁面內容
使用 `.dialog-backdrop` 的對話框（含 SyncConflictDialog）SHALL 以固定定位覆蓋整個視窗，其疊層（z-index）MUST 高於頁面內所有 sticky 元素（sub-tab bar、工具列、側欄），且低於全域 toast 通知。

#### Scenario: 同步衝突對話框開啟
- **WHEN** 套用同步遇到衝突而彈出 SyncConflictDialog
- **THEN** 對話框與背景遮罩完整覆蓋於 sub-tab bar 與工具列之上，無任何頁面元素穿透顯示於對話框前方

### Requirement: 可預覽各目標端內容
Skills / Commands 矩陣中，當某目標（claude / codex / opencode / copilot）的狀態非「缺少目標」（target-missing）時，該目標的狀態 pill SHALL 可點擊；點擊後系統 SHALL 載入並顯示該目標端檔案內容（skill 為 `<targetRoot>/<name>/SKILL.md`，command 為 `<targetRoot>/<name>.md`），且預覽標題 MUST 標示項目名稱與目標識別（如 `openspec-explore (opencode)`）。

#### Scenario: 檢視內容不同的目標端版本
- **WHEN** 某 skill 在 opencode 目標的狀態為「內容不同」，使用者點擊該狀態 pill
- **THEN** 預覽面板顯示 `.opencode/skills/<name>/SKILL.md` 的內容，標題標示項目名稱與 opencode

#### Scenario: 目標缺少時不可點擊
- **WHEN** 某項目在 codex 目標的狀態為「缺少目標」
- **THEN** 該狀態 pill 不可點擊，不觸發任何載入行為

#### Scenario: 點擊項目名稱預覽來源端
- **WHEN** 使用者點擊矩陣中的項目名稱
- **THEN** 詳情頁顯示來源端（`.agents` 下，若來源不存在則為探索到的既有檔案）內容

### Requirement: 內容檢視改為整頁詳情模式
Skills / Commands 矩陣中，點擊項目名稱或可點擊的目標狀態時，系統 SHALL 以整頁詳情模式取代原本在列表下方展開的預覽面板：進入詳情頁時列表（含工具列、矩陣表格、同步報告）SHALL 隱藏，改為全幅顯示所選檔案的內容檢視與一個「返回」控制項；使用者按下返回控制項後 SHALL 回到列表，且列表先前的捲動位置與勾選狀態 SHALL 保留。

#### Scenario: 進入詳情頁後返回列表
- **WHEN** 使用者於 Skills 矩陣點擊某項目名稱進入詳情頁，再按下返回控制項
- **THEN** 畫面回到 Skills 矩陣列表，先前勾選的項目仍為勾選狀態，不需重新載入掃描資料

#### Scenario: 詳情頁不需捲動至底部
- **WHEN** 使用者在矩陣中點擊任一可檢視項目
- **THEN** 內容檢視於當前視窗頂部起始顯示，使用者無需捲動至列表底部即可閱讀，且詳情頁工具列（外部開啟／檔案總管顯示）與返回控制項同時可見

### Requirement: 外部開啟與檔案總管顯示需具備 opener 路徑 scope
詳情頁工具列的「外部開啟」（`openPath`）與「檔案總管顯示」（`revealItemInDir`）SHALL 能對掃描所得的 agents 設定檔路徑（含 `.agents`、各 target root 及使用者家目錄下之路徑）成功執行。由於 Tauri opener 外掛對 `open-path` 指令套用路徑 scope 限制，`src-tauri/capabilities/default.json` 的 `opener:allow-open-path` 權限 MUST 以帶 `allow` scope 的物件形式宣告，允許應用實際會開啟的檔案路徑，避免 `openPath` 因 scope 未涵蓋而被拒絕。

#### Scenario: 開啟 skill 檔案不被權限拒絕
- **WHEN** 使用者在 Skills 詳情頁點擊「外部開啟」，目標為 `.agents/skills/<name>/SKILL.md`
- **THEN** 系統以外部程式開啟該檔案，不再出現 `Not allowed to open path` 錯誤

#### Scenario: 檔案總管顯示可定位檔案
- **WHEN** 使用者於詳情頁點擊「檔案總管顯示」
- **THEN** 系統於檔案總管中定位並選取該檔案

### Requirement: 矩陣欄位對齊與精簡狀態呈現
Skills / Commands 矩陣的排版 SHALL 修正欄位對齊問題：各目標欄（claude / codex / opencode / copilot）的欄寬 SHALL 一致，欄標題的平台勾選框與名稱 SHALL 與其下方狀態格水平置中對齊；狀態呈現 SHALL 精簡為狀態圖示（點／符號）搭配文字，配色柔和，不使用過重的邊框與全大寫粗體膠囊樣式。

#### Scenario: 欄標題與狀態格對齊
- **WHEN** 使用者檢視 Skills 矩陣
- **THEN** 每個平台欄標題的勾選框與該欄各列的狀態圖示垂直對齊於同一水平中心，各平台欄寬相等

### Requirement: Commands 名稱比對需正規化 Copilot 的 prompt 副檔名
Commands 矩陣以「來源與各 target 的邏輯名稱」聯集決定列（見 `agents-commands-sync` D5c）。由於 GitHub Copilot 的 `.github/prompts/` 目錄慣例要求檔案副檔名為 `<name>.prompt.md`，掃描與同步 SHALL 於 copilot target 一律以 `.prompt.md` 作為副檔名解析與組裝邏輯名稱；其餘 target（claude/codex/opencode）與來源端 SHALL 沿用 `.md`。同一邏輯名稱在不同 target 下的副檔名差異 MUST NOT 造成該名稱被拆分為多個矩陣列。

#### Scenario: Copilot 端 prompt 檔案正確歸併至同一 command
- **WHEN** 來源與 claude/opencode 目標存在 `opsx-apply.md`，copilot 目標存在 `opsx-apply.prompt.md`，且內容一致
- **THEN** 矩陣僅顯示一列 `opsx-apply`，其 claude/opencode/copilot 三欄狀態皆為「一致」，不會另外產生 `opsx-apply.prompt` 這個獨立列

#### Scenario: 套用同步時寫入 Copilot 目標使用正確副檔名
- **WHEN** 使用者對某 command 勾選 copilot 目標並套用同步
- **THEN** 系統寫入 `<copilotTargetRoot>/<name>.prompt.md`，而非 `<name>.md`
