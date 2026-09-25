## MODIFIED Requirements

### Requirement: 提供時序聚合統計查詢

系統 SHALL 提供統一分析查詢，接受起訖本地日期、時區、day/week/month 粒度、provider、可選 cwd／模型、封存範圍與指標，依事件發生時間聚合；摘要、趨勢、專案／模型排行及明細 MUST 共用條件與 revision。

#### Scenario: 依日分組查詢單一專案
- **WHEN** 指定 cwd 與日期範圍依日查詢
- **THEN** 只聚合該 canonical cwd、指定來源及範圍內事件，日標籤為 YYYY-MM-DD
- **AND** 跨日 session 的用量分別歸到各事件日期，不以 session updated_at 歸因

#### Scenario: 依周分組查詢所有專案
- **WHEN** cwd 為 null 且依週查詢
- **THEN** 聚合所有符合其他條件的專案，週從週一開始，標籤為 ISO 週年 YYYY-WNN

#### Scenario: session_stats 快取未完整覆蓋
- **WHEN** 部分來源尚未完成用量事件索引或僅提供部分指標
- **THEN** 報表回傳已知事件值及指標級 missing counts，並列出 pending／unsupported／error 涵蓋狀態
- **AND** 不使用缺少的 session_stats 快取或 session updated_at 推算事件用量

#### Scenario: 依月分組查詢
- **WHEN** 分組為 month
- **THEN** 月標籤為 YYYY-MM，首尾桶只計查詢區間內事件

#### Scenario: 本地日期邊界
- **WHEN** 查詢包含本地午夜或日光節約時間變更
- **THEN** 範圍為該時區開始日午夜至結束日次日午夜的半開區間，不以固定 UTC offset 或字串截斷判定

#### Scenario: 無法取得支援時區
- **WHEN** 系統時區無法辨識為受支援 IANA identifier
- **THEN** 顯示錯誤而非靜默變更日期意義；使用者明確選擇 UTC 後才以 UTC 查詢並標示

#### Scenario: 區間內無資料
- **WHEN** 索引完整且指定範圍沒有符合事件
- **THEN** 回傳空狀態與 coverage 而非錯誤；有事件的報表中完整且無事件的桶補零
- **AND** pending、unsupported 或 error 不被當成已確認零用量

#### Scenario: 部分資料尚未完成
- **WHEN** 部分來源尚未索引或僅能提供部分指標
- **THEN** 回傳已知值、指標級完整性與未涵蓋來源，不宣稱已取得完整總量

#### Scenario: 篩選封存與停用來源
- **WHEN** 使用預設條件查詢
- **THEN** 僅納入已啟用 provider、未封存 session；明確 includeArchived 才包含封存
- **AND** 空 provider 選取回傳空結果，不擴大成全部來源

### Requirement: AnalyticsDataPoint 型別定義

分析回應 SHALL 以 camelCase 提供型別化報表，包含 summary、previousSummary、series、projectRanking、modelRanking、coverage、sessionSummaryOnly、revision 與 generatedAt；資料點有 label、可得 input/output/total tokens、去重 usage 訊息數及 distinct session 數，成本獨立為估算 USD 與點數。

#### Scenario: 前端收到完整欄位
- **WHEN** 查詢成功
- **THEN** 每個指標可識別已知值與缺漏狀態，不使用混合單位 costPoints
- **AND** cache 與 reasoning 子集合不再次計入 total

#### Scenario: 部分事件只知道 output
- **WHEN** 部分事件只有 output 而無 input
- **THEN** total 僅加總 input/output 都已知的事件並標記缺漏，input 與 output 單獨指標各自加總已知事件
- **AND** total 的 input/output 堆疊使用相同完整事件集合；完全無可計量事件時 knownValue 為 null 而非零

#### Scenario: 跨桶 Session 數
- **WHEN** 同一 session 在多日都有事件
- **THEN** 摘要 session 數只計一次，各日各計一次，不將日 session 數相加當摘要

#### Scenario: 只有全期彙總的資料
- **WHEN** session 有彙總但無對應指標事件
- **THEN** 彙總放在獨立區域，依可得活動範圍與所選區間重疊篩選，標示全 session 總量
- **AND** 不加入事件摘要或趨勢；無有效時間範圍者回報未定位數量

### Requirement: 查詢參數驗證

系統 SHALL 驗證分析與明細查詢的日期、時區、分組、指標與分頁條件，拒絕無效值而非默默擴大範圍。

#### Scenario: 無效日期格式
- **WHEN** 起訖日期格式不符 YYYY-MM-DD 或日期不存在
- **THEN** 回傳 invalid date format 錯誤

#### Scenario: startDate 晚於 endDate
- **WHEN** startDate 晚於 endDate
- **THEN** 回傳 startDate must be before endDate 錯誤；相同日期允許查詢整個本地日

#### Scenario: 無效 groupBy 值
- **WHEN** groupBy 不在 day、week、month
- **THEN** 回傳 invalid groupBy value 錯誤

#### Scenario: 無效時區或分頁
- **WHEN** 時區無法解析、指標不受支援、page 小於 1 或 pageSize 不在 1 到 200
- **THEN** 回傳可辨識的驗證錯誤，不執行未限制查詢

## ADDED Requirements

### Requirement: 分頁明細可與報表對帳

系統 SHALL 提供相同條件的分頁 session 明細，預設每頁 50 筆並依事件時間降冪穩定排序，以 provider/session 識別解決同值排序，且檢查報表 revision。

#### Scenario: 明細與排行加總
- **WHEN** 在同一 revision 查閱所有頁面的可加總指標
- **THEN** 明細加總、完整排行加總與摘要一致，session 全期彙總不混入

#### Scenario: 舊 revision 查詢下一頁
- **WHEN** 事件索引已更新而明細請求仍引用舊 revision
- **THEN** 回報 stale-revision，讓前端重取一致報表，而非混合不同版本資料

### Requirement: 指標涵蓋與成本精度可解釋

分析 SHALL 逐指標回報已知與缺欄位事件數、各狀態 session 數及 provider 能力，不將其推算成實際用量涵蓋百分比；成本保持固定精度與估價來源。

#### Scenario: 未解析與無時間資料
- **WHEN** scope 內包含 pending、unsupported、error 或無可定位時間的資料
- **THEN** 顯示無法判定期間涵蓋的來源，不能因目前已知事件完整就把整體標為 complete
- **AND** 排行佔比只以相同指標已知值總和為分母，不表示完整用量涵蓋率

#### Scenario: 部分成本與混合估價版本
- **WHEN** 有已知成本、未知價格或不同估價版本的事件
- **THEN** 以逐事件十進位小數 6 位 half-up 的固定精度加總已知成本，分別回報未知與版本混合資訊
- **AND** 不以顯示到 2 位的金額重新加總，不默默按最新價格重估歷史

#### Scenario: Revision 包含範圍異動
- **WHEN** 用量、來源設定、cwd 或封存狀態改變而影響報表
- **THEN** 報表全域 revision 隨原子交易更新，摘要及排行在同一讀取快照取得
