## ADDED Requirements

### Requirement: Skills provider 集合由啟用平台決定

Skills 掃描與介面 provider 欄位 SHALL 以 `AppSettings.enabledProviders` 為唯一來源，保持設定順序並去除重複值。停用 provider SHALL 不解析任何 Skills root、不啟動 CLI、不建立狀態欄位；`enabledTargets` 只作同步目標偏好，MUST NOT 決定 provider 是否出現在主清單。

#### Scenario: 僅啟用 OpenCode 與 Codex
- **WHEN** `enabledProviders` 僅包含 `opencode` 與 `codex`
- **THEN** Skills 掃描只解析 OpenCode 與 Codex roots，清單也只顯示這兩個 provider
- **AND** 不存取 Claude、Copilot 或 Antigravity Skills 目錄

#### Scenario: 設定停用 provider
- **WHEN** 使用者在設定頁停用 Copilot
- **THEN** Copilot Skills query 資料失效，重新掃描後 Copilot 欄位消失

### Requirement: Skills 依 provider 相容 roots 掃描

系統 SHALL 為每個啟用 provider 掃描其 project/global scope 實際支援的 Skills roots，而非只掃 `<scope>/.agents/skills` 或將同步目標當作完整 discovery。第一版 SHALL 支援：Claude `.claude/skills`；Codex `.codex/skills` 與 `.agents/skills`；OpenCode `.opencode/skill`、`.opencode/skills`、`.claude/skills`、`.agents/skills`；Copilot `.github/skills`、`.agents/skills`、`.claude/skills`；Antigravity/Gemini `.gemini/skills`、`.agents/skills`，並使用對應 user roots。不同 provider 的同名 skill SHALL 分別判定；同一 provider 多個 root 的同名 skill SHALL 保留所有 locations，除非 provider CLI 能確認，系統 MUST NOT 猜測 effective path 或遮蔽關係。

#### Scenario: OpenCode 同時使用 native 與 agents skills
- **WHEN** 專案同時存在 `.opencode/skills/open-native/SKILL.md` 與 `.agents/skills/shared/SKILL.md`
- **THEN** OpenCode discovery 同時包含 `open-native` 與 `shared`

#### Scenario: Claude 不假設 agents 相容
- **WHEN** skill 只存在 `.agents/skills/shared/SKILL.md`，且 Claude 已啟用
- **THEN** 系統不得僅因該檔案存在便將 Claude 標示為可見或已載入

#### Scenario: Copilot 與 Gemini 使用 agents alias
- **WHEN** skill 存在 `.agents/skills/shared/SKILL.md`，且 Copilot 與 Antigravity 已啟用
- **THEN** 兩個 provider 的 root discovery 均包含該 skill

#### Scenario: 同名 skill 保留所有來源
- **WHEN** OpenCode 的 `.opencode/skills/review` 與 `.agents/skills/review` 同時存在
- **THEN** root discovery 在同一個 OpenCode `review` 項目保留兩個 locations
- **AND** 只有 OpenCode CLI 查詢成功時才標示 effective path

### Requirement: OpenCode Skills 實際可用狀態由 CLI 查詢決定

Skills 掃描 SHALL 只對 OpenCode 使用已驗證的 JSON CLI 查詢有效技能清單，並以指定 scope 的工作目錄執行：project scope 使用專案根目錄取得 effective 合併結果；global scope 使用不含專案設定的中立工作目錄。OpenCode 結果 SHALL 與 root discovery 及同步雜湊狀態保持獨立，MUST NOT 再由檔案存在或同步狀態推導技能已載入或未安裝。Claude、Codex、Copilot、Antigravity Skills SHALL 不執行 CLI 狀態檢查，只保留 root discovery、預覽與同步資料。

#### Scenario: CLI 確認技能可用
- **WHEN** OpenCode CLI 在指定 scope 回傳某個 skill
- **THEN** 該 skill 的 OpenCode effective 狀態為 `available`，資料來源為 `cli`
- **AND** 無論其同步狀態為何，介面均不得以同步狀態覆寫此實際狀態

#### Scenario: 專案查詢包含有效合併結果
- **WHEN** 某 skill 僅由專案層設定提供，且系統以該專案根目錄查詢 provider
- **THEN** project scope 結果包含該 skill
- **AND** global scope 的中立目錄查詢不將它列為全域 skill

#### Scenario: OpenCode CLI 未回傳掃描到的技能
- **WHEN** 檔案掃描找到某個 skill，但已成功的 OpenCode CLI 查詢未將它列入有效清單
- **THEN** 系統不得將其 OpenCode 狀態標示為 `available` 或「未安裝」
- **AND** 仍可保留該檔案項目與同步狀態供使用者管理

### Requirement: 未支援的 Skills provider 跳過 CLI 檢查

Claude、Codex、Copilot、Antigravity Skills 因沒有已驗證的穩定機器介面，系統 SHALL 跳過其 CLI 狀態檢查，但仍 SHALL 執行 provider root discovery；不得顯示 `available`、`unknown`、`error`、「已載入」、「未安裝」或「未驗證」CLI badge。OpenCode CLI 不存在、逾時或 JSON 無法解析時，系統 SHALL 保留 provider root discovery 與同步資料，只在群組層顯示非阻擋提示，不為各項目建立推測 CLI 狀態。

#### Scenario: 未支援 provider 不啟動 CLI
- **WHEN** 使用者開啟 Skills 頁籤
- **THEN** 系統不為 Claude、Codex、Copilot、Antigravity 啟動技能列舉指令
- **AND** 介面只顯示這些 provider 的 root discovery 與同步資訊

#### Scenario: CLI 查詢逾時
- **WHEN** OpenCode 列舉指令超過系統設定的查詢逾時
- **THEN** 系統終止該查詢並在群組層顯示無法更新 CLI 狀態
- **AND** 不讓 Agents 頁持續停留在載入中
