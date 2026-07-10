## ADDED Requirements

### Requirement: AGENTS.md/CLAUDE.md 遞迴掃描與狀態 Tree

系統 SHALL 遞迴掃描指定範圍內的 AGENTS.md 與 CLAUDE.md，並以 Tree 呈現每個含指示檔目錄的同步狀態。專案範圍以專案根目錄為起點；全域範圍僅檢查固定已知的 agent 根目錄位置，不得遞迴使用者家目錄。

#### Scenario: 掃描專案並顯示狀態 Tree

- **WHEN** 使用者開啟專案的 Agents 分頁（AGENTS.md 子分頁）
- **THEN** 系統遞迴掃描專案目錄（深度上限 8，忽略 node_modules、.git、dist、build、vendor、.next、.nuxt、target、.sessionhub 及專案偏好中的忽略路徑）
- **AND** 以 Tree 列出每個含 AGENTS.md 或 CLAUDE.md 的目錄，節點標示同步狀態 badge

#### Scenario: 同步狀態判定

- **WHEN** 某目錄同時存在 AGENTS.md 與 CLAUDE.md 且內容雜湊相等
- **THEN** 狀態為「一致」（in-sync）
- **WHEN** 僅存在 AGENTS.md
- **THEN** 狀態為「缺 CLAUDE.md」（target-missing）
- **WHEN** 兩者皆存在但內容不同
- **THEN** 狀態為「內容不同」（differs），且 CLAUDE.md mtime 較新時額外標示「目標較新」
- **WHEN** 僅存在 CLAUDE.md
- **THEN** 狀態為「僅有 CLAUDE.md」（source-missing）

#### Scenario: 掃描量超過上限

- **WHEN** 掃描目錄數超過上限（約 20,000）
- **THEN** 系統停止掃描、回傳已掃描結果並設 truncated 旗標
- **AND** UI 顯示截斷警告

#### Scenario: 掃描不阻塞 UI

- **WHEN** 掃描大型專案（如含 node_modules 的 repo）
- **THEN** 掃描於背景執行緒執行，UI 保持可操作，完成後更新 Tree

### Requirement: 指示檔檢視、編輯與開啟操作

系統 SHALL 支援對 Tree 中選取的 AGENTS.md / CLAUDE.md 進行檢視（markdown 渲染）、就地編輯儲存、以外部編輯器開啟、在檔案總管顯示。

#### Scenario: 檢視指示檔

- **WHEN** 使用者點選 Tree 中的檔案節點
- **THEN** 右側面板以 markdown 渲染顯示檔案內容

#### Scenario: 就地編輯與儲存

- **WHEN** 使用者切換至編輯模式、修改內容並儲存
- **THEN** 系統將內容寫回原檔（寫入暫存檔後原子替換）
- **AND** 寫入路徑須通過範圍根目錄的路徑防護檢查（canonicalize + 包含檢查，拒絕路徑穿越）
- **AND** 儲存後重新整理該節點狀態

#### Scenario: 外部開啟

- **WHEN** 使用者點擊「以外部編輯器開啟」或「在檔案總管顯示」
- **THEN** 系統分別以設定的外部編輯器開啟該檔案、或在檔案總管中顯示該檔案

### Requirement: AGENTS.md → CLAUDE.md 同步（agents-sync 語意）

系統 SHALL 提供內建同步引擎：以 AGENTS.md 為預設來源，將內容複製為同目錄的 CLAUDE.md。同步前 SHALL 提供 dry-run 預覽並允許逐項勾選，實際寫入採原子替換。

#### Scenario: dry-run 預覽

- **WHEN** 使用者點擊「預覽同步」
- **THEN** 系統執行與實際同步完全相同的判定管線但不寫入任何檔案
- **AND** 顯示逐項動作清單（建立 / 覆蓋 / 略過（已一致）/ 衝突 / 錯誤）供勾選

#### Scenario: 套用同步

- **WHEN** 使用者於預覽清單勾選項目並點擊「套用」
- **THEN** 系統僅對勾選項目執行寫入：目標不存在時建立、內容不同時依方向覆蓋、已一致者略過
- **AND** 完成後顯示結果摘要（建立/覆蓋/略過/錯誤數）並更新 Tree 狀態

#### Scenario: 反向同步

- **WHEN** 使用者於衝突對話框選擇「目標→來源」方向
- **THEN** 系統以 CLAUDE.md 內容覆蓋 AGENTS.md

### Requirement: 同步衝突詢問與記住選擇

當（a）目標檔比來源檔新且內容不同、或（b）來源檔不存在但目標檔存在（`source-missing`），且專案無已記住的衝突選擇時，系統 SHALL 暫停該項寫入並詢問使用者覆蓋方向；使用者 MAY 勾選「記住此專案的選擇」使後續同步自動套用。

#### Scenario: 目標較新時詢問

- **WHEN** 同步時發現 CLAUDE.md 的 mtime 比 AGENTS.md 新且內容不同，且專案偏好中無記住的衝突選擇
- **THEN** 系統不寫入該項並將其標記為衝突
- **AND** UI 顯示衝突對話框，逐項提供「來源→目標」「目標→來源」「略過」選項與「套用到全部」

#### Scenario: 來源缺失時詢問

- **WHEN** 同步時發現 AGENTS.md 不存在但 CLAUDE.md 存在（source-missing），且專案偏好中無記住的衝突選擇
- **THEN** 系統不寫入該項並將其標記為衝突（不比較 mtime）
- **AND** UI 顯示衝突對話框，提供「目標→來源」（以 CLAUDE.md 內容補回 AGENTS.md）或「略過」選項
- **AND** 啟用「強制覆蓋」不改變此行為——來源缺失時仍必須詢問，force 不會自動決定以目標覆蓋來源

#### Scenario: 記住選擇

- **WHEN** 使用者於衝突對話框勾選「記住此專案的選擇」並確認
- **THEN** 系統將選擇（source-wins / target-wins）寫入專案 agents 偏好
- **AND** 後續同步遇到相同型態衝突時自動套用該方向，不再詢問

#### Scenario: 強制覆蓋

- **WHEN** 使用者啟用「強制覆蓋」選項執行同步，且該項目來源存在
- **THEN** 系統跳過衝突詢問，一律以來源覆蓋目標
- **AND** 若該項目為來源缺失（source-missing），強制覆蓋選項不生效，仍依「來源缺失時詢問」流程處理

### Requirement: 全域範圍指示檔管理

系統 SHALL 於全域 Agents 頁面列出各 agent 全域根目錄下的指示檔（如 `~/.claude/CLAUDE.md`、`~/.codex/AGENTS.md`、`~/.config/opencode/AGENTS.md`），支援與專案範圍相同的檢視、編輯與開啟操作。

#### Scenario: 全域指示檔清單

- **WHEN** 使用者開啟全域 Agents 頁面的 AGENTS.md 分頁
- **THEN** 系統僅檢查固定已知的 agent 根目錄位置（依設定頁的根目錄覆蓋值解析）並列出存在的指示檔
- **AND** 不存在的位置標示為未建立

#### Scenario: 掃描 ~/.agents/instructions

- **WHEN** 使用者另外維護 `~/.agents/instructions/AGENTS.md`
- **THEN** 全域 AGENTS 掃描 SHALL 納入 `~/.agents/instructions/` 作為固定已知位置之一
- **AND** 若同目錄存在 `CLAUDE.md`，其狀態判定與一般 AGENTS/CLAUDE 對位一致
