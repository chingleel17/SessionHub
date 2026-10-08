## Purpose

為多個 coding provider 建立可追溯的用量資料基礎，保留實際事件時間、來源識別、計量單位及完整性，讓歷史分析能區分精確事件與 session 彙總，避免重複計算、錯誤跨日歸因或將缺資料視為零。

## ADDED Requirements

### Requirement: 用量事件依來源身分去重

系統 SHALL 以 provider、session 與來源事件識別區分用量，依事件發生時間記錄，並將串流修訂更新至同一筆用量。

#### Scenario: 重複掃描與串流修訂
- **WHEN** 同一訊息被重複掃描或收到更新後 usage
- **THEN** 重複掃描不增加總量，修訂取代同一訊息先前 usage
- **AND** 不因 session 最後更新時間改變其事件日期

#### Scenario: 跨 provider 相同 session ID
- **WHEN** 兩個 provider 有相同 session ID
- **THEN** 用量與 metadata 各自歸屬正確 provider，不互相覆寫或串接

### Requirement: Token 與成本有明確語意

系統 SHALL 區分未知與零，並以不重複計算的方式正規化 input、output、cache 與 reasoning；估算 USD 與 Copilot 點數 MUST 分開。

#### Scenario: Cache 與 reasoning 子集合
- **WHEN** 事件提供含 cache 的 input 及含 reasoning 的 output
- **THEN** total 等於 input 加 output，不再次加上 cache 或 reasoning
- **AND** 缺任一必要總量欄位時 total 顯示未知，保留已知細項

#### Scenario: 無價格或不同成本單位
- **WHEN** 查詢包含只有點數、只有 USD 估值或無價格的事件
- **THEN** 分開回傳點數與估算 USD，未知成本標示缺漏且不當成零或免費

### Requirement: Session 彙總不得偽裝成事件用量

系統 SHALL 將只有 session 粒度的用量與事件用量分開，保留來源與可得時間範圍，不做未有證據的跨日分攤。

#### Scenario: Copilot shutdown 與事件 output 並存
- **WHEN** 同一 session 同時具有 assistant output 與 shutdown totals
- **THEN** 事件趨勢只使用事件 output，shutdown totals 在獨立 session 彙總區顯示
- **AND** 兩者不相加，彙總不得標示為所選期間消耗

### Requirement: Provider 能力與資料缺漏可觀察

系統 SHALL 按來源與指標揭露 complete、partial、pending、unsupported 或 error，支援 Claude 與 OpenCode 可驗證的事件用量、Copilot 部分事件及獨立彙總，Codex 僅使用可驗證格式，Antigravity 本版顯示 token 分析未支援。

#### Scenario: Codex 累計資料缺少可靠基準
- **WHEN** 累計 counter 首筆無基準、倒退或無法確認事件時間與計量範圍
- **THEN** 不將累計總量直接當作本次消耗，不產生負值
- **AND** 受影響區段標記 partial；完全未知格式標記 unsupported，不以 quota 百分比補值

#### Scenario: 部分 provider 尚未完成索引
- **WHEN** 一部分資料已可分析，另一部分仍 pending 或 unsupported
- **THEN** 已知資料可顯示，完整性提示列出未涵蓋來源，摘要不得聲稱完整總量

### Requirement: 增量更新可恢復且保持一致

系統 SHALL 在背景索引變更來源，保留可恢復進度，並以完整 revision 發布每個 session 的更新；歷史查詢不得同步重新掃描所有原始檔。

#### Scenario: 更新中斷或來源半筆資料
- **WHEN** 索引中斷、解析失敗或來源正寫入不完整紀錄
- **THEN** 保留上次完整結果並標記 pending/error，重試不重複計量，其他 session 仍可處理

#### Scenario: 改寫與刪除來源
- **WHEN** 可讀來源的完整掃描確認檔案縮短、改寫或刪除
- **THEN** 更新或移除對應衍生用量，不留下舊資料重複加總
- **AND** 根目錄暫時失聯不視為全部 session 已刪除

#### Scenario: 切換來源或重建索引
- **WHEN** 設定來源根目錄改變或解析版本升級
- **THEN** 目前查詢不混入舊根目錄資料，新的索引可恢復補算
- **AND** 不修改原始 session、備註或標籤
