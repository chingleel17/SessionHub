## ADDED Requirements

### Requirement: 專案設定資料夾建立開關

系統 SHALL 於全域設定提供 `allowCreateProjectConfigDir` 開關（預設關閉）。此開關**僅控制是否允許新建 / 寫入**專案根目錄下的 `.sessionhub/` 資料夾，不影響讀取行為——即使開關關閉，若專案內已存在 `.sessionhub/agents.json`（例如他人建立或先前已建立），系統仍會讀取並繼續寫回該既有檔案。

#### Scenario: 設定頁顯示開關

- **WHEN** 使用者開啟設定頁
- **THEN** 系統於「Agents」區塊顯示「允許在專案內建立 .sessionhub 設定資料夾」開關與說明文字（明確標註僅影響新建/寫入，不影響既有檔案的讀取）
- **AND** 預設為關閉

#### Scenario: 開啟時新偏好存於專案內

- **WHEN** `allowCreateProjectConfigDir` 為開啟，且系統需要儲存專案 agents 偏好，且專案內尚無該檔案
- **THEN** 系統建立 `<project>/.sessionhub/agents.json` 並寫入
- **AND** 首次建立時 UI 顯示提示，建議使用者自行決定是否將 `.sessionhub/` 加入 `.gitignore`（系統不自動修改 .gitignore）

#### Scenario: 關閉且專案內無既有檔案時走 APPDATA fallback

- **WHEN** `allowCreateProjectConfigDir` 為關閉，且系統需要儲存專案 agents 偏好，且專案內尚無 `.sessionhub/agents.json`
- **THEN** 系統不在專案內建立任何檔案，改寫入 `%APPDATA%\SessionHub\project-agents\<專案路徑雜湊>.json`

#### Scenario: 關閉但專案內已有既有檔案時仍寫回專案內

- **WHEN** `allowCreateProjectConfigDir` 為關閉，但專案內已存在 `.sessionhub/agents.json`
- **THEN** 系統仍讀取並寫回該既有檔案，不因開關關閉而改寫到 APPDATA
- **AND** 不會刪除或搬移該既有檔案

#### Scenario: 開關切換不遷移既有偏好

- **WHEN** 使用者切換 `allowCreateProjectConfigDir` 的開關狀態
- **THEN** 系統不自動搬移或合併 APPDATA 與專案內兩處既有的偏好內容，兩者各自獨立保留
- **AND** 後續讀取依「專案內優先、否則 APPDATA」的優先序取用其一，不合併欄位

### Requirement: 專案級 agents 偏好持久化

系統 SHALL 持久化每個專案的 agents 偏好：記住的衝突選擇（conflictChoice）、掃描忽略路徑（ignoredPaths）、啟用的同步目標（enabledTargets）。讀取時 SHALL 優先採用專案內 `.sessionhub/agents.json`（不論 `allowCreateProjectConfigDir` 狀態，只要檔案存在即讀取），不存在時回退至 APPDATA 位置，兩者皆不存在時使用預設值（衝突每次詢問、無忽略路徑、四個目標全啟用）。

#### Scenario: 讀取偏好的優先序

- **WHEN** 開啟專案的 Agents 分頁
- **THEN** 系統依序嘗試讀取 `<project>/.sessionhub/agents.json` → APPDATA fallback → 預設值
- **AND** 此優先序不受 `allowCreateProjectConfigDir` 開關狀態影響

#### Scenario: 偏好向後相容

- **WHEN** 偏好檔缺少部分欄位（舊版本寫入）
- **THEN** 系統以預設值補齊缺少欄位，不報錯

### Requirement: 全域 agents 來源根目錄可自訂

系統 SHALL 於全域設定提供 `agentsSourceRoot` 欄位（預設為空字串），讓使用者自訂全域範圍 Skills / Commands / AGENTS.md 的正本來源目錄，覆寫預設的 `~/.agents`。此設定僅影響全域範圍（`AgentsScope::Global`），專案範圍固定使用 `<project>/.agents`，不受影響。

#### Scenario: 設定頁顯示來源路徑欄位

- **WHEN** 使用者開啟設定頁的「Agents」區塊
- **THEN** 系統顯示「Agents 正本根目錄（全域範圍）」路徑輸入欄與「瀏覽」按鈕
- **AND** 欄位留空時顯示預設值（`%USERPROFILE%\.agents`）作為提示文字

#### Scenario: 設定自訂路徑後全域掃描改用該路徑

- **WHEN** 使用者將 `agentsSourceRoot` 設為 `D:\ching\AI tool setting\agents` 並儲存
- **THEN** 全域範圍的 Skills / Commands 矩陣改以 `D:\ching\AI tool setting\agents\skills`（及其 `command` 子目錄）為來源端進行掃描與同步
- **AND** 全域 AGENTS.md 掃描改以該目錄為根

#### Scenario: 欄位留空時維持原有預設行為

- **WHEN** `agentsSourceRoot` 為空字串或僅含空白
- **THEN** 系統 fallback 使用 `%USERPROFILE%\.agents` 作為全域來源根目錄，行為與升級前一致

#### Scenario: 專案範圍不受此設定影響

- **WHEN** 使用者已設定 `agentsSourceRoot`，並開啟某專案的 Agents 分頁
- **THEN** 該專案範圍仍固定使用 `<project>/.agents` 作為來源根目錄，不套用全域自訂路徑

### Requirement: 主題色系改由設定頁管理

系統 SHALL 將主題／色系切換集中於設定頁管理，避免 Sidebar 在展開與收折狀態出現不同位置或重複控制。

#### Scenario: Sidebar 不重複顯示主題切換

- **WHEN** 使用者切換 Sidebar 展開或收折狀態
- **THEN** 系統不應因不同狀態重複渲染多組主題切換控制

#### Scenario: 設定頁顯示主題切換

- **WHEN** 使用者開啟設定頁
- **THEN** 系統於設定頁顯示單一主題／色系切換控制
- **AND** 其行為與原先 Sidebar 主題切換一致
