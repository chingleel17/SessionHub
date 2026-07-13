# agents-config-view-ux

## ADDED Requirements

### Requirement: Agents 頁去卡片化

Agents 頁（AGENTS.md / Skills / Commands / MCP）SHALL 採 Minimal 去卡片版面：內容容器（矩陣、清單、收折分區、詳情視圖、指示檔樹側欄）SHALL NOT 使用邊框 + 圓角 + 陰影 + 背景色塊構成的多層卡片，改以留白與極淡 hairline 分隔建立層級，使頁面呈現為連續畫布。僅浮層類元件（同步 modal 內的 conflict 項目、檔案預覽 modal）SHALL 保留卡片邊界。

#### Scenario: 清單與矩陣無卡片框

- **WHEN** 使用者檢視 Skills / Commands 清單或同步矩陣
- **THEN** 清單/矩陣容器無外框、無圓角色塊、無陰影，列與列之間以極淡 hairline 或 hover 背景區分

#### Scenario: 收折分區無卡片框

- **WHEN** 使用者檢視「專案」「全域」收折分區
- **THEN** 分區以標題列 + hairline 分界呈現，SHALL NOT 為每個分區加卡片邊框/圓角/陰影/背景色塊

#### Scenario: 指示檔樹側欄融入畫布

- **WHEN** 使用者檢視 AGENTS.md 頁的指示檔樹側欄
- **THEN** 側欄背景透明（融入畫布），僅以右邊界 hairline 與內容區分隔，明暗主題皆不呈現明顯色塊

#### Scenario: 硬編碼淺色一律 token 化

- **WHEN** 於深色主題檢視 Agents 頁任一元件
- **THEN** 各元件 SHALL NOT 因硬編碼淺色（如 `rgba(255,255,255,…)`）而在深色下露出白底；背景、邊框、hover 皆使用主題 token

### Requirement: 收折分區標題列整合操作與路徑

Agents 頁與 MCP 頁的收折分區（「專案」「全域」）標題列 SHALL 整合該分區的操作按鈕與設定檔路徑：路徑以小字顯示於標題右側（可截斷），操作按鈕群置於標題列最右側；SHALL NOT 於分區內容內另設獨立的「標題 + 說明 + 操作」工具列行。單一 scope（無收折分區）情境 SHALL 改以內容頂部的內嵌操作列呈現同樣的路徑與操作。

`CollapsibleSection` SHALL 提供 `titleMeta`（標題旁小字）與 `actions`（標題列操作按鈕）兩個可選插槽，且 `actions` 不參與收折點擊。

#### Scenario: 操作按鈕位於收折標題列

- **WHEN** 使用者檢視 MCP 頁「專案」分區
- **THEN** 外開 / 資料夾 / 重整 / 新增 MCP Server 按鈕顯示於「專案 (N)」收折標題列右側，設定檔路徑以小字顯示於標題與按鈕之間
- **AND** 分區內容內 SHALL NOT 再出現重複的 provider 名稱 + 路徑 + 操作的獨立工具列行

#### Scenario: 收折標題「新增」自動展開

- **WHEN** 分區收折時使用者點擊標題列的「新增 MCP Server」
- **THEN** 分區展開並開啟新增編輯器

#### Scenario: 單一 scope 內嵌操作列

- **WHEN** 使用者於全域 Agents 頁（無專案/全域分組）檢視 AGENTS.md 或 MCP
- **THEN** 操作按鈕與路徑以內容頂部的單一內嵌列呈現，不另設卡片式工具列

### Requirement: 儲存按鈕圖示與顯示時機

AGENTS.md 內容編輯的「儲存」按鈕 SHALL 使用語意正確的儲存圖示（`SaveIcon`，非同步圖示），且 SHALL 僅於編輯狀態顯示；非編輯（預覽）狀態不顯示儲存按鈕。

#### Scenario: 儲存按鈕僅編輯時出現

- **WHEN** 使用者處於預覽狀態
- **THEN** 僅顯示「編輯」按鈕，不顯示「儲存」按鈕

- **WHEN** 使用者點擊「編輯」進入編輯狀態
- **THEN** 顯示「儲存」按鈕，圖示為 SaveIcon

### Requirement: 檔案預覽 modal 的滾動與關閉

Agents 頁的檔案預覽 modal SHALL 使其內容滾動落在圓角範圍內（不使 scrollbar 溢出圓角外緣），並於 header 提供關閉按鈕。

#### Scenario: 滾動條不溢出圓角

- **WHEN** 使用者開啟一份較長的 SKILL.md 預覽
- **THEN** modal 以固定高度 + 內容區內部滾動呈現，scrollbar 位於圓角內，不出現於 modal 圓角外緣

#### Scenario: header 具關閉按鈕

- **WHEN** 使用者檢視預覽 modal header
- **THEN** header 除「返回列表」外，最右側 SHALL 提供關閉（×）按鈕，點擊後關閉 modal
