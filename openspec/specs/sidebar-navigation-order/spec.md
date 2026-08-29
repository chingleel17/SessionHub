## Purpose

讓使用者依工作優先順序排列 Sidebar 的釘選專案，並在應用程式重新啟動或專案清單更新後維持可預期且可靠的主要導覽順序。

## Requirements

### Requirement: 釘選專案可拖曳排序

Sidebar SHALL 允許使用者以拖曳方式重新排列目前可見的釘選專案；排序操作 SHALL 僅改變釘選專案彼此的順序，不得改變 Dashboard、全域 Agents、設定或已開啟未釘選專案的位置。Windows Tauri 主視窗 SHALL 保留 WebView 的 HTML drag-and-drop 事件，不得由未使用的原生檔案拖放機制攔截 Sidebar 的拖曳流程。

#### Scenario: 將釘選專案移至另一個釘選專案前方

- **WHEN** 使用者拖曳一個釘選專案並放到另一個釘選專案的插入位置
- **THEN** Sidebar 立即依放置位置更新釘選專案順序
- **AND** 被拖曳專案保持釘選狀態
- **AND** 當前作用中的 view 不因排序而切換

#### Scenario: 無有效位移時不變更順序

- **WHEN** 使用者將釘選專案放回原位置、放在自身，或在有效 drop target 外結束拖曳
- **THEN** 釘選專案順序維持不變
- **AND** 系統不執行不必要的設定儲存

#### Scenario: 收折狀態仍可排序

- **WHEN** Sidebar 處於收折狀態且使用者以釘選專案 icon 執行拖曳排序
- **THEN** 系統 SHALL 提供與展開狀態相同的排序能力
- **AND** tooltip 或 accessible name 仍可辨識被拖曳的專案

#### Scenario: Windows 桌面版啟動拖曳

- **WHEN** 使用者在 Windows Tauri 主視窗按住釘選或已開啟專案並移動指標
- **THEN** WebView SHALL 觸發 Sidebar 的 HTML drag-and-drop 流程
- **AND** Tauri 原生檔案拖放設定不得使專案拖曳完全無法啟動

### Requirement: 釘選順序持久化

系統 SHALL 將成功變更的釘選順序保存至應用程式設定，並在後續載入時以保存的順序呈現釘選專案。

#### Scenario: 重新啟動後保留順序

- **WHEN** 使用者完成有效的釘選專案排序且設定儲存成功
- **THEN** `pinnedProjects` SHALL 以新的順序保存
- **AND** 應用程式重新啟動後 SHALL 依該順序顯示釘選專案

#### Scenario: 儲存失敗

- **WHEN** 使用者完成排序但設定無法保存
- **THEN** 系統 SHALL 回復排序前的釘選順序
- **AND** 以既有錯誤提示機制告知使用者排序未保存

### Requirement: 排序保留暫時不可見的釘選資料

重新排列目前可見的釘選專案時，系統 SHALL 保留設定中暫時無法對應到現有專案的釘選 key，且 SHALL 維持這些不可見 key 彼此的相對順序。

#### Scenario: 設定包含暫時不可用專案

- **WHEN** `pinnedProjects` 同時包含目前可見與暫時無法對應的專案，且使用者重新排列可見專案
- **THEN** 系統僅重排可見專案在既有有序清單中所占的槽位
- **AND** 暫時不可見的釘選 key 不會因本次排序遭到刪除或改變彼此順序

#### Scenario: 暫時不可用專案恢復

- **WHEN** 先前無法對應的釘選專案再次出現在專案資料中
- **THEN** Sidebar SHALL 依保存後的 `pinnedProjects` 順序顯示該專案

### Requirement: 拖曳排序提供明確回饋

Sidebar SHALL 在釘選排序期間顯示被拖曳項目與預計插入位置，回饋樣式 SHALL 在淺色與深色主題可辨識，並遵循 reduced-motion 偏好。回饋 SHALL 限於項目層級，不得替整個釘選區增加外框或大面積背景。

#### Scenario: 拖曳經過有效目標

- **WHEN** 使用者拖曳釘選專案經過另一個釘選專案
- **THEN** 被拖曳項目 SHALL 呈現拖曳狀態
- **AND** 預計插入位置 SHALL 顯示不造成版面跳動的指示

#### Scenario: 結束拖曳後清除回饋

- **WHEN** 使用者完成或取消拖曳
- **THEN** 所有拖曳中、drop target 與插入位置視覺狀態 SHALL 立即清除

#### Scenario: 拖入釘選區時維持安靜版面

- **WHEN** 使用者將未釘選的已開啟專案拖入釘選區，或在釘選項目之間排序
- **THEN** Sidebar SHALL 僅以被拖曳項目狀態與目標項目的插入位置提示提供回饋
- **AND** 釘選區不得顯示整區虛線或實線外框
- **AND** 拖曳回饋不得超出 Sidebar 可用寬度或遭捲動容器裁切

### Requirement: 釘選與已開啟專案拖曳行為互不干擾

系統 SHALL 區分釘選專案排序與已開啟未釘選專案的既有拖曳行為，避免同一拖曳事件同時觸發排序與釘選動作。

#### Scenario: 排序釘選專案

- **WHEN** 拖曳來源是已釘選專案
- **THEN** 放置於釘選區僅執行釘選順序調整
- **AND** 不執行取消釘選或重複釘選

#### Scenario: 將已開啟專案拖入釘選區

- **WHEN** 拖曳來源是未釘選的已開啟專案且放置於釘選區
- **THEN** 系統維持既有將該專案加入釘選清單的行為
- **AND** 新釘選專案加入既有釘選順序末端
