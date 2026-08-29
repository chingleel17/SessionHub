## Purpose

讓使用者不必先從 Dashboard 尋找專案，即可由 Sidebar 的低干擾入口查看目前尚未出現在導覽中的已偵測專案，並選擇開啟或釘選。

## Requirements

### Requirement: 專案分隔線提供單一新增入口

Sidebar SHALL 在 Dashboard 下方的主要專案分隔線提供單一 `+ New` 入口；入口 SHALL 使用與分隔線相近的低對比色彩及與分支標籤相同層級的文字尺寸，不得另占一列或在每個專案區段重複顯示。

#### Scenario: 展開狀態顯示新增入口

- **WHEN** Sidebar 處於展開狀態
- **THEN** Dashboard 下方的分隔線尾端 SHALL 顯示低對比 `+ New` 按鈕
- **AND** 按鈕與分隔線共用同一列，不增加明顯垂直高度

#### Scenario: 同時存在釘選與已開啟區段

- **WHEN** Sidebar 同時顯示釘選專案與已開啟未釘選專案，因而出現多條區段分隔線
- **THEN** 專案選擇入口 SHALL 只出現在 Dashboard 下方的第一條主要分隔線
- **AND** 已開啟區段原有的全部關閉操作 SHALL 維持不變

#### Scenario: 收折狀態顯示入口

- **WHEN** Sidebar 處於收折狀態
- **THEN** 入口 SHALL 以緊湊 `+` icon 保持可用
- **AND** accessible name 與 tooltip SHALL 使用在地化的新增專案文案

### Requirement: Modal 僅列出可加入的已偵測專案

啟用新增入口後，系統 SHALL 開啟 modal，列出目前已偵測且同時不在釘選清單與已開啟清單中的專案。每筆候選 SHALL 顯示專案名稱，並在資料存在時顯示分支與路徑。

#### Scenario: 顯示候選專案

- **WHEN** 已偵測專案中存在尚未釘選且尚未開啟的項目
- **THEN** modal SHALL 列出所有符合條件的候選專案
- **AND** 不得列出任何已釘選或已開啟的專案

#### Scenario: 沒有候選專案

- **WHEN** 所有已偵測專案都已釘選或開啟，或目前沒有已偵測專案
- **THEN** modal SHALL 顯示在地化空狀態
- **AND** 不顯示無效的專案 action

#### Scenario: 候選項目超出可視高度

- **WHEN** 候選專案數量超出 modal 可視高度
- **THEN** 清單區 SHALL 可在 modal 內捲動
- **AND** dialog 不得超出應用程式可視邊界

### Requirement: 候選專案可選擇開啟或釘選

每個候選專案 SHALL 提供「開啟」與「釘選」操作，分別加入已開啟區段或釘選區段；任一操作成功後 SHALL 關閉 modal，且不得產生重複項目。

#### Scenario: 開啟候選專案

- **WHEN** 使用者對候選專案選擇「開啟」
- **THEN** 專案 SHALL 加入已開啟未釘選清單
- **AND** active view SHALL 切換至該專案
- **AND** modal SHALL 關閉

#### Scenario: 釘選候選專案

- **WHEN** 使用者對候選專案選擇「釘選」
- **THEN** 專案 SHALL 加入釘選順序末端並保存設定
- **AND** modal SHALL 關閉
- **AND** 專案不得同時重複顯示於未釘選清單

#### Scenario: 關閉 modal 不變更 Sidebar

- **WHEN** 使用者按下關閉按鈕、Escape 或啟用 backdrop 關閉行為
- **THEN** modal SHALL 關閉
- **AND** 釘選、已開啟及 active view 狀態 SHALL 維持不變

### Requirement: 專案選擇 modal 維持可及性與視覺一致性

專案選擇 modal SHALL 使用既有 SessionHub dialog 視覺語言、design token 與在地化文案，並提供 dialog semantics、初始焦點、鍵盤操作及關閉後焦點回復。

#### Scenario: 以鍵盤開啟與關閉

- **WHEN** 使用者以鍵盤啟用 `+ New` 並於 modal 內操作
- **THEN** 焦點 SHALL 移入 dialog 的可操作內容
- **AND** Escape 關閉後焦點 SHALL 回到原新增入口

#### Scenario: 雙主題顯示

- **WHEN** 應用程式使用淺色或深色主題
- **THEN** modal、候選列、次要資訊、hover、focus 與 action SHALL 保持可辨識對比
