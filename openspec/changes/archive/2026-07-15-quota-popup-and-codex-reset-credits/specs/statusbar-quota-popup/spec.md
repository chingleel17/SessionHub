## ADDED Requirements

### Requirement: 點擊狀態列 quota 區域開啟彈出面板

系統 SHALL 讓狀態列右側的 quota 區域可點擊：點擊時在狀態列上方彈出一個錨定於右下的浮動額度面板；再次點擊該區域 SHALL 關閉面板（toggle）。

#### Scenario: 點擊開啟面板

- **WHEN** 面板為關閉狀態且使用者點擊狀態列 quota 區域
- **THEN** 額度面板顯示於狀態列上方、靠視窗右側，z-index 高於主內容區域
- **AND** hover tooltip 行為不受影響

#### Scenario: 再次點擊關閉面板

- **WHEN** 面板為開啟狀態且使用者再次點擊狀態列 quota 區域
- **THEN** 面板關閉

### Requirement: 彈出面板重用 QuotaOverview 顯示完整額度資訊

彈出面板 SHALL 直接渲染與 Dashboard 相同的 QuotaOverview 元件（含 provider tabs、「全部」tab、用量條、重置倒數、重置額度區塊與刷新按鈕），資料與刷新回呼由 App 經 StatusBar props 下傳，元件本身不得直接呼叫 Tauri IPC。面板 tab 選取 SHALL 以獨立的 localStorage key 記憶，不與 Dashboard 互相覆寫。

#### Scenario: 面板顯示與 Dashboard 一致的內容

- **WHEN** 面板開啟且 quota snapshots 已載入
- **THEN** 面板顯示與 Dashboard QuotaOverview 相同的 provider tabs 與面板內容
- **AND** 點擊刷新按鈕觸發與 Dashboard 相同的 refresh 流程

#### Scenario: 面板與 Dashboard 各自記憶 tab 選取

- **WHEN** 使用者在彈出面板切換至「全部」tab，之後開啟 Dashboard
- **THEN** Dashboard 的 QuotaOverview 維持自己先前的 tab 選取，不被面板影響

#### Scenario: 內容過長時面板可捲動

- **WHEN** 面板內容高度超過可用視窗高度（例如「全部」tab 列出多個 provider）
- **THEN** 面板以 max-height 限制並在內部垂直捲動，不遮蓋狀態列也不溢出視窗

### Requirement: 點擊面板外或按 Escape 關閉面板

面板開啟時，系統 SHALL 在使用者點擊面板與 quota 區域以外的任意位置、或按下 Escape 鍵時關閉面板。

#### Scenario: 點擊面板外關閉

- **WHEN** 面板開啟且使用者點擊主內容區域
- **THEN** 面板關閉，且該次點擊不觸發面板內的任何操作

#### Scenario: Escape 關閉

- **WHEN** 面板開啟且使用者按下 Escape
- **THEN** 面板關閉
