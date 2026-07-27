# terminal-child-env-parity

## ADDED Requirements

### Requirement: 子程序環境區塊完整性

SessionHub 於 Windows 啟動任何終端或 AI coding CLI 子程序時，SHALL 將自身程序的完整環境區塊傳遞給子程序，除本規格明確允許的注入項目外，SHALL NOT 移除、清空或覆寫任何既有環境變數。

允許的注入項目限於系統為緩解已知平台缺陷而必要者（例如 MSYS stackdump 緩解），且每一項注入 SHALL 保留使用者既有值的語意（採合併而非取代）。

#### Scenario: 使用者環境變數完整傳遞

- **WHEN** SessionHub 啟動終端或 AI coding CLI 子程序
- **THEN** 子程序環境區塊包含 SessionHub 程序環境區塊中的每一個變數名稱
- **AND** 除允許注入項目外，各變數值與 SessionHub 程序中的值相同

#### Scenario: 注入項目採合併語意

- **WHEN** 系統需注入某個環境變數，且該變數在 SessionHub 程序環境中已存在使用者設定的值
- **THEN** 系統合併既有值與所需選項，SHALL NOT 直接以新值取代既有值

#### Scenario: 不得注入 home 相關變數

- **WHEN** SessionHub 啟動子程序
- **THEN** 系統 SHALL NOT 為解決工具路徑解析問題而注入或覆寫 `HOME`、`USERPROFILE`、`HOMEDRIVE`、`HOMEPATH` 或任何 `XDG_*` 變數

### Requirement: 保留使用者明確設定的工具停用旗標

SessionHub SHALL NOT 為修正 skill 載入問題而清除、覆寫或忽略使用者明確設定的 AI coding CLI 停用旗標。

適用範圍包含但不限於 `OPENCODE_DISABLE_EXTERNAL_SKILLS`、`OPENCODE_DISABLE_CLAUDE_CODE_SKILLS`、`OPENCODE_PURE`。

#### Scenario: 使用者已設定停用旗標

- **WHEN** 使用者在系統或使用者層級設定了 OpenCode 停用旗標，並從 SessionHub 開啟終端
- **THEN** 子程序中該旗標的值與使用者設定相同
- **AND** 系統不因此變更任何啟動行為以繞過該旗標

#### Scenario: 使用者未設定停用旗標

- **WHEN** 使用者未設定任何 OpenCode 停用旗標
- **THEN** 系統 SHALL NOT 主動設定這些旗標為任何值（包含設為空字串）

### Requirement: 連結目錄的子程序可視性

SessionHub 以 symlink 或 junction 管理 `~/.agents` 與 `~/.claude` 之下的 skills 時，SHALL 確保由 SessionHub 啟動的子程序能夠解析並遍歷這些連結，其結果與使用者手動開啟的終端一致。

#### Scenario: 子程序可解析全域 skills 連結

- **WHEN** `~/.agents\skills` 或 `~/.claude\skills` 之下存在 SessionHub 建立的 symlink/junction，且使用者從 SessionHub 開啟終端
- **THEN** 子程序遍歷該路徑所得到的項目集合，與使用者手動開啟終端遍歷所得的集合相同

#### Scenario: 連結無法解析時可被診斷

- **WHEN** 子程序因權限或連結型別而無法解析上述連結
- **THEN** 系統以可觀測的方式記錄此失敗，SHALL NOT 靜默地回報為空集合

### Requirement: 啟動入口一致性

所有啟動子程序的入口（一般終端、AI coding CLI 啟動、session resume）SHALL 共用同一套環境傳遞實作，任一入口 SHALL NOT 具備與其他入口不同的環境處理邏輯。

#### Scenario: 三個入口行為一致

- **WHEN** 分別經由 `open_terminal`、`open_in_tool`、`resume_session_in_terminal` 啟動子程序，且目標工作目錄相同
- **THEN** 三者建立的子程序具有相同的環境區塊內容

#### Scenario: 新增啟動入口沿用共用實作

- **WHEN** 系統新增任何啟動終端或 CLI 子程序的入口
- **THEN** 該入口沿用共用的環境傳遞實作，不自行組裝環境區塊

### Requirement: 全域 skill 載入回歸驗證

系統 SHALL 提供可重複執行的驗證程序，用以確認從 SessionHub 啟動的終端中，OpenCode 掃描到的 skill 根目錄集合與使用者手動終端一致。

驗證 SHALL 以「掃描到的 skill 根目錄集合」為判準，SHALL NOT 僅以 skill 總數為判準，因為總數相同不代表來源相同。

#### Scenario: 根目錄集合一致

- **WHEN** 在相同專案目錄下，分別於 SessionHub 終端與手動終端執行 OpenCode 的 skill 診斷指令
- **THEN** 兩者掃描到的 skill 根目錄集合相同，且皆包含 `~/.agents`、`~/.claude` 與專案內的 skill 目錄

#### Scenario: 驗證程序不受殘留程序影響

- **WHEN** 執行驗證程序
- **THEN** 驗證前確認無殘留的 OpenCode 常駐程序，避免讀數來自已暖機的既有程序而非當次環境
