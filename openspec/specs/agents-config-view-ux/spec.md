# agents-config-view-ux

## ADDED Requirements

### Requirement: Agents 頁新增 MCP 頁籤

`AgentsConfigView` SHALL 在 AGENTS.md / Skills / Commands 之外新增第四個頁籤「MCP」，內容渲染 `McpConfigView`（依實例自身的 scope 顯示與操作對應 scope 的 MCP 設定）。全域 Agents 頁（sidebar 進入）與專案 Agents sub-tab 皆因此具備 MCP 管理能力；sidebar SHALL NOT 提供獨立的 MCP 導覽項。

#### Scenario: 全域 Agents 頁的 MCP 頁籤

- **WHEN** 使用者自 sidebar 開啟全域 Agents 頁並切換到 MCP 頁籤
- **THEN** 顯示 global scope 的四平台 MCP server 清單，操作寫入 global 設定檔

#### Scenario: 頁籤切換清除操作狀態

- **WHEN** 使用者在 MCP 頁籤開啟編輯 dialog 後切換至 Skills 頁籤再切回
- **THEN** MCP 頁籤回到清單狀態，dialog 不殘留

### Requirement: 專案 Agents 分頁以單一頁籤列與 scope 分組呈現

專案 Agents sub-tab SHALL 只有一列共用頁籤（AGENTS.md / Skills / Commands / MCP），切換頁籤同時作用於專案與全域內容；頁籤內容 SHALL 以「專案」「全域」兩個可收折群組呈現，群組標題列含名稱、項目計數與展開/收折控制。全域群組的資料與操作 SHALL 與全域 Agents 頁共用（同 query 與 handlers，操作後兩處一致）；全域 Agents 頁本身不分組。首次進入預設 SHALL 為專案群組展開、全域群組收折；收折狀態 SHALL 記憶於 localStorage（依專案路徑區分）。MCP 頁籤的 provider 分頁 SHALL 於兩群組之上共用一列，codex 專案信任提示僅顯示於專案群組。系統 SHALL NOT 以兩份各自帶頁籤列的完整實例呈現（頁籤不同步且過度佔用垂直空間）。

#### Scenario: 頁籤同步切換

- **WHEN** 使用者於專案 Agents 分頁將頁籤切換至 Skills
- **THEN** 專案與全域兩個群組同時顯示 Skills 內容，無需分別切換

#### Scenario: 群組計數與收折

- **WHEN** 使用者展開 Skills 頁籤且專案有 3 個 skill、全域有 16 個 skill
- **THEN** 群組標題分別顯示「專案 3」「全域 16」；點擊標題可收折該群組，收折狀態於重新進入時還原

#### Scenario: 在全域群組操作寫入全域

- **WHEN** 使用者在專案分頁 MCP 頁籤的全域群組停用一個 server
- **THEN** 變更寫入 global 設定（或 global 停用暫存），專案設定檔不變，且全域 Agents 頁顯示一致

### Requirement: 群組內容精簡且不產生水平溢出

收折群組的內容區 SHALL NOT 重複顯示群組標題列已呈現的 scope 名稱與項目計數（不得再渲染帶邊框圓角的「scope 名稱 + 計數」工具列區塊）；群組層級的操作（同步、重新整理）SHALL 以精簡按鈕併入內容區頂部既有的說明列右側（說明文字縮排讓出空間），無說明列的頁籤則置於清單頂部右側的單一窄列。Agents 頁內容 SHALL 隨可用寬度自適應（含 sidebar 展開/收合切換時），SHALL NOT 出現水平滾動條；寬度不足時內文以截斷或換行處理，不得將版面撐寬。

#### Scenario: 群組內不重複標題

- **WHEN** 使用者展開 Skills 頁籤的「專案 (8)」群組
- **THEN** 群組內容區直接顯示說明列與清單，不再出現「專案 / 8 個項目」的重複工具列區塊

#### Scenario: 同步與重整按鈕位於說明列右側

- **WHEN** 使用者檢視 Skills 群組內容
- **THEN** 「同步」與重新整理按鈕顯示於 .agents 相容性說明列的右側，說明文字縮排讓出按鈕空間

#### Scenario: sidebar 展開不產生水平滾動條

- **WHEN** 使用者展開 sidebar 使內容區可用寬度縮小
- **THEN** Agents 頁內容縮排適應新寬度，無水平滾動條出現，過長文字以截斷呈現

### Requirement: Skills 與 Commands 以名稱加描述的清單呈現並支援搜尋

Skills / Commands 頁籤的清單 SHALL 以每列「名稱 + 描述」呈現（描述為次要文字、單行截斷並以 tooltip 提供完整內容），SHALL NOT 於清單內常駐 per-target 狀態矩陣欄；每列 MAY 顯示精簡的同步狀態摘要 badge。描述 SHALL 取自來源檔 YAML frontmatter 的 `description` 欄位（skill 為 `SKILL.md`、command 為對應 `.md`），無 frontmatter 或無該欄位時留空。頁籤內容頂部 SHALL 提供搜尋框，依名稱與描述過濾清單（專案分頁同時過濾兩個群組）。點擊列名稱 SHALL 維持既有整頁詳情模式預覽。

#### Scenario: 清單顯示描述

- **WHEN** 某 skill 的 `SKILL.md` frontmatter 含 `description: Automate web browsers...`
- **THEN** 清單該列於名稱下方顯示該描述（過長時截斷並以 tooltip 顯示完整內容）

#### Scenario: 搜尋過濾

- **WHEN** 使用者於搜尋框輸入關鍵字
- **THEN** 兩個群組僅顯示名稱或描述含該關鍵字的項目，群組計數同步更新

#### Scenario: 無描述項目

- **WHEN** 某 command 的 `.md` 無 frontmatter
- **THEN** 該列僅顯示名稱，描述區留空，不顯示錯誤

### Requirement: 同步操作集中於同步 modal

Skills / Commands 的 per-target 同步矩陣、目標欄勾選、同步模式選擇、預覽/套用與結果報告 SHALL 自清單頁移除，改由「同步」按鈕開啟 modal 承載：modal 內提供該 scope 的項目勾選、目標勾選、同步模式（copy/link，僅 skills）、「預覽同步」「套用同步」與結果報告（含報告列勾選重跑）。衝突處理沿用既有 SyncConflictDialog，其疊層 MUST 高於同步 modal。modal 關閉時 SHALL 清除預覽報告與勾選狀態。AGENTS.md 頁籤的單目錄同步行為不變。

#### Scenario: 開啟同步 modal 並套用

- **WHEN** 使用者於 Skills 頁籤點擊「同步」按鈕、於 modal 勾選項目與目標後按「套用同步」
- **THEN** 系統執行同步並於 modal 內顯示結果報告；清單頁不出現常駐的矩陣或報告區塊

#### Scenario: modal 中遇到衝突

- **WHEN** 套用同步回報衝突
- **THEN** SyncConflictDialog 疊加顯示於同步 modal 之上，解決後結果更新於 modal 報告

#### Scenario: 關閉 modal 清除狀態

- **WHEN** 使用者關閉同步 modal 後再次開啟
- **THEN** modal 回到初始狀態，先前的報告與勾選不殘留
