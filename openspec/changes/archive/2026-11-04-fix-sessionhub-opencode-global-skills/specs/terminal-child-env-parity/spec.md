# terminal-child-env-parity

## ADDED Requirements

### Requirement: OpenCode skills 來源比對

系統 SHALL 提供可重複執行的驗證程序，對照同一工作目錄中手動終端與 SessionHub shell 終端執行 OpenCode skill 診斷時的結果；若啟用 herdr，SHALL 另記錄 herdr pane 的結果與 server 啟動狀態。比對 SHALL 以 skill 名稱與來源位置集合為準，不得僅以總數或 SessionHub 自身的資源列示判定是否一致。

#### Scenario: 相同條件下列示一致

- **WHEN** 在同一專案目錄下、同一 OpenCode 版本與相同有效設定，手動終端及 SessionHub shell 終端分別成功執行 `opencode debug skill`
- **THEN** 驗證程序記錄 exit code、CLI 版本、筆數及 `(name, location)` 集合，並比對兩側結果
- **AND** 若兩側來源集合相同且含預期全域來源，回報目前無法重現載入差異，不要求更動子程序啟動實作

#### Scenario: 數量相同但來源不同

- **WHEN** 兩側診斷結果筆數相同，但 skill 名稱或來源路徑不同
- **THEN** 驗證程序回報差集，不得宣稱通過

#### Scenario: 診斷指令失敗

- **WHEN** 任一側的 OpenCode 診斷指令回傳非零 exit code 或無法提供可比對的清單
- **THEN** 驗證結果標記為無法判定，先檢查設定與版本，不將零筆或空結果當成缺漏來源的證據

#### Scenario: herdr 使用既有服務

- **WHEN** 由 SessionHub 啟動的 herdr pane 使用已存在的 server／client
- **THEN** 驗證程序記錄服務啟動來源與冷／暖啟狀態，並單獨比對 pane 結果
- **AND** 不假設該 pane 的環境區塊與本次 SessionHub 母程序相同

### Requirement: shell 子程序環境繼承界線

SessionHub SHALL 保留 shell 新程序對母程序環境的既有繼承行為，且 SHALL NOT 為修正 skills 問題強制設定全域 skills 路徑、清除使用者的 OpenCode 停用旗標或覆寫 home 相關環境變數。手動終端與 SessionHub 母程序的環境可能原本不同；只有可重現且已證實由 SessionHub 啟動流程造成的差異，才需要調整啟動實作。

#### Scenario: 使用者設定停用旗標

- **WHEN** SessionHub 程序環境中已設定 `OPENCODE_DISABLE_EXTERNAL_SKILLS`、`OPENCODE_DISABLE_CLAUDE_CODE_SKILLS` 或 `OPENCODE_PURE`
- **THEN** SessionHub 啟動的 shell 子程序保留其既有值，不繞過使用者設定

#### Scenario: 使用者未設定停用旗標

- **WHEN** SessionHub 程序環境中未設定上述旗標
- **THEN** SessionHub 不為修正 skills 問題主動注入上述旗標（包含空字串）

#### Scenario: 保留既有 MSYS 合併

- **WHEN** SessionHub 在 Windows 建立 shell 終端且母程序已有 MSYS 選項
- **THEN** 子程序保留既有選項並合併 stackdump 緩解設定，不以新值取代

#### Scenario: 不硬編 home 或 skills 路徑

- **WHEN** SessionHub 建立 shell 子程序
- **THEN** 系統不為技能載入注入或覆寫 `HOME`、`USERPROFILE`、`HOMEDRIVE`、`HOMEPATH`、`XDG_*` 或任何固定的 skills 位置

### Requirement: 依證據處理可重現的載入差異

在來源集合確有差異時，系統 SHALL 先辨識差異是由 SessionHub shell 啟動、herdr 既有服務、OpenCode 設定、CLI 行為或連結遍歷造成；僅在確認 SessionHub 啟動流程是原因時修正對應路徑，並以原重現步驟驗證。

#### Scenario: 差異來自 SessionHub 啟動流程

- **WHEN** 同條件比對可重現來源缺漏且已排除 CLI 設定與既有服務污染，並證明缺漏由 SessionHub 啟動流程造成
- **THEN** 系統修正已定位的啟動路徑，附上回歸測試，且保留使用者既有設定與原有命令／工作目錄／console flags

#### Scenario: 連結無法遍歷

- **WHEN** 可重現差異與由 SessionHub 管理的 skills symlink／junction 遍歷失敗有因果關係
- **THEN** 診斷程序記錄失敗路徑及錯誤，不以空集合代替成功載入，並針對已證實的責任邊界修正

#### Scenario: 差異屬於外部來源

- **WHEN** 缺漏源自使用者設定、CLI 行為或既有服務，而非 SessionHub 啟動流程
- **THEN** 記錄原因與重現條件，不對 SessionHub 子程序環境做無根據的修改
