## ADDED Requirements

### Requirement: 讀取所有 session

系統 SHALL 掃描設定的 Copilot 根目錄下 `session-state/` 子目錄，讀取每個子目錄內的 `workspace.yaml`，並回傳結構化 session 清單。

#### Scenario: 正常讀取 session 列表

- **WHEN** 使用者開啟應用程式且 `session-state/` 目錄存在
- **THEN** 系統顯示所有解析成功的 session，每筆包含：id、summary（若存在）、cwd、created_at、updated_at、summary_count

### Requirement: Session 包含 provider 識別資訊

每個 session SHALL 攜帶 `provider` 欄位，識別其來源為 Copilot 或 OpenCode。

#### Scenario: 掃描多 provider session

- **WHEN** copilotRoot 和 opencodeRoot 均已設定
- **THEN** 系統分別掃描兩個 provider 的 session 目錄
- **AND** 回傳清單將兩者合並，依最新修改時間降序排序

#### Scenario: Session 卡片顯示 provider 標籤

- **WHEN** 顯示 OpenCode 的 session
- **THEN** session 卡片顯示小型 provider 標籤（如 `OpenCode`）
- **AND** Copilot session 不顯示額外標籤（為預設 provider）

### Requirement: Session 大量屬性附加

系統 SHALL 在 SessionInfo 中附加 has_plan、has_events、parse_error 等與 UI 相關屬性。

#### Scenario: Session 屬性完整

- **WHEN** 系統解析 session 目錄
- **THEN** SessionInfo SHALL 包含以下欄位：
  - `has_plan: bool` — session 目錄下存在 plan.md
  - `has_events: bool` — session 目錄下存在 events.jsonl
  - `parse_error: Option<String>` — workspace.yaml 解析失敗的錯誤訊息

### Requirement: 筐選多維度過濾

Session 清單 SHALL 支援多維度等過濾條件的組合。

#### Scenario: 過濾空 session

- **WHEN** 使用者啟用「隱藏空 session」
- **THEN** summary 為空且 summary_count 為 0 的 session SHALL 被隱藏

#### Scenario: 過濾特定 provider

- **WHEN** 使用者選擇 provider 等過濾
- **THEN** 僅顯示符合 provider 的 session

#### Scenario: 多條件同時過濾

- **WHEN** 多個過濾條件同時啟用
- **THEN** 系統以 AND 邏輯對它們進行組合過濾

### Requirement: Session 卡片顯示統計 badge 行

Session 卡片 SHALL 在底部顯示緊湊的統計 badge。

#### Scenario: Stats 已載入

- **WHEN** session 卡片轉現且統計已載入
- **THEN** 卡片底部 badge 顯示：互動次數、輸出 token（K 格式）、展有時長

#### Scenario: Stats 載入中

- **WHEN** stats 尚未取得
- **THEN** badge 區域顯示骨架狀態，不产生版面跳動

#### Scenario: 無統計資料

- **WHEN** session 無 events.jsonl（所有數字為 0）
- **THEN** badge 行隱藏
