## Purpose

定義 Sidebar 收折、對齊與垂直空間分配行為，確保不同視窗高度與導覽項目數量下皆維持一致且可操作的版面。

## Requirements

### Requirement: 側欄收折展開平滑過渡

側欄收折與展開 SHALL 以平滑動畫過渡（寬度變化約 150–250ms），文字內容以淡出/淡入方式隱藏與顯示，不得瞬間跳變；在 `prefers-reduced-motion` 環境下 SHALL 停用動畫並立即切換。

#### Scenario: 使用者收折側欄

- **WHEN** 使用者點擊收折按鈕
- **THEN** 側欄寬度以平滑動畫縮小至收折寬度
- **AND** 導覽文字與面板內容淡出，不產生內容擠壓變形

#### Scenario: 使用者展開側欄

- **WHEN** 使用者於收折狀態點擊展開按鈕
- **THEN** 側欄寬度以平滑動畫恢復，文字內容淡入

#### Scenario: 減少動態偏好

- **WHEN** 作業系統啟用 reduce motion
- **THEN** 收折/展開立即切換，不播放過渡動畫

### Requirement: 收折按鈕位置固定

收折／展開按鈕 SHALL 整合於品牌列靠 Sidebar 邊界的尾端位置，並在展開與收折兩種狀態下保持可見、可操作及對齊品牌區；按鈕不得獨占 Dashboard 上方的額外列。

#### Scenario: 展開狀態顯示控制

- **WHEN** Sidebar 處於展開狀態
- **THEN** 收折按鈕顯示於品牌名稱尾端、靠近 Sidebar 邊界
- **AND** Dashboard 導覽項目前不保留僅供此按鈕使用的空白列

#### Scenario: 切換至收折狀態

- **WHEN** 使用者點擊收折按鈕且 Sidebar 完成收折
- **THEN** 展開按鈕仍顯示於品牌 icon 旁或其靠 Sidebar 邊界的尾端區域
- **AND** 按鈕不遮蔽品牌 icon、workspace 內容或 Sidebar 導覽項目

#### Scenario: 切換收折狀態時按鈕不位移

- **WHEN** 使用者連續切換收折與展開
- **THEN** 控制項 SHALL 隨 Sidebar 邊界平滑移動並維持一致的品牌列垂直位置
- **AND** 每個狀態下的點擊目標 SHALL 保持完整可用

### Requirement: 收折控制圖示反映目標狀態

收折／展開按鈕 SHALL 依 Sidebar 當前狀態顯示方向或語意不同的圖示，使圖示清楚表達下一次啟用將執行收折或展開；按鈕的 accessible name 與 title SHALL 使用對應的在地化文案。

#### Scenario: 展開狀態顯示收折圖示

- **WHEN** Sidebar 處於展開狀態
- **THEN** 按鈕顯示指向收折方向的圖示
- **AND** accessible name 與 title 表達「收折側欄」

#### Scenario: 收折狀態顯示展開圖示

- **WHEN** Sidebar 處於收折狀態
- **THEN** 按鈕顯示指向展開方向的圖示
- **AND** accessible name 與 title 表達「展開側欄」

#### Scenario: 以鍵盤切換側欄

- **WHEN** 鍵盤焦點位於收折／展開按鈕且使用者啟用按鈕
- **THEN** Sidebar SHALL 切換狀態
- **AND** 焦點 SHALL 保留在同一控制項上以便連續操作

### Requirement: 收折狀態 icon 對齊一致

收折狀態下，品牌 app icon、一般導覽 icon、釘選專案首字母 icon 與已開啟專案首字母 icon SHALL 使用相同的 Sidebar 水平置中基準；收折與展開切換時，icon SHALL 不產生可見的水平跳位。專案名稱等已隱藏文字 SHALL 完全退出水平 flex 配額，不得因透明但仍可伸展的文字容器將 icon 推離中心；pin 徽章與關閉按鈕等角落附屬控制 SHALL 以不參與主 icon 排版尺寸的方式定位。

#### Scenario: 收折後 icon 對齊

- **WHEN** 側欄處於收折狀態
- **THEN** 品牌 icon、一般導覽 icon 及所有專案首字母 icon 在同一垂直軸線上置中對齊
- **AND** active 背景區塊 SHALL 以同一軸線置中，不因專案名稱、pin 徽章或關閉按鈕偏移

#### Scenario: 釘選徽章不影響置中

- **WHEN** 收折狀態的專案首字母 icon 顯示 pin 徽章
- **THEN** 首字母 icon 與 active 背景 SHALL 維持相對 Sidebar 的水平置中
- **AND** pin 徽章 SHALL 僅疊加於 icon 角落，不納入主 icon 的置中寬度計算

#### Scenario: 切換時 icon 不跳位

- **WHEN** 使用者收折或展開側欄
- **THEN** 導覽 icon 的水平位置平滑過渡或維持不變，無瞬間跳動

### Requirement: 收折展開共用單一版面結構

側欄收折與展開 SHALL 共用同一套 DOM 與版面結構（導覽、釘選、已開啟項目、footer 均使用相同元件），收折僅隱藏文字內容，不得切換為另一套元件版型；收折過程中任何元素 SHALL 不產生水平漂移（先向右再向左收回等軌跡）。

#### Scenario: 收折過程無水平漂移

- **WHEN** 使用者收折或展開側欄
- **THEN** 所有導覽項目、專案項目與 footer 元素僅文字淡出/淡入，起始水平位置維持不變

#### Scenario: 釘選項目兩態一致

- **WHEN** 側欄在展開與收折間切換
- **THEN** 釘選專案項目在兩態皆以首字母 icon（含 pin 徽章）呈現，收折後仍可辨識個別專案

### Requirement: 即時狀態指示點位置固定

底部即時掃描狀態指示點 SHALL 在收折與展開兩種狀態下維持可見且位置不變（對齊側欄 icon 軸）；收折時僅文字標籤與刷新按鈕淡出，指示點 SHALL 不得被壓縮或移出可視範圍。

#### Scenario: 收折後指示點仍可見

- **WHEN** 使用者收折側欄
- **THEN** 狀態指示點停留在原位置且保持原尺寸，僅文字與刷新按鈕淡出

### Requirement: Sidebar 依視窗高度保留底部區域

Sidebar SHALL 受應用程式實際 viewport 高度約束；品牌區與 footer SHALL 保持可見且不得被主要導覽清單壓縮或推離視窗。當專案項目超過中段可用高度時，只有主要導覽區 SHALL 垂直捲動，footer 仍固定於 Sidebar 底部。

#### Scenario: 視窗高度不足以顯示全部專案

- **WHEN** 應用程式不是全螢幕，且釘選與已開啟專案清單高度超過品牌區與 footer 之間的可用空間
- **THEN** 主要導覽區 SHALL 在剩餘高度內提供垂直捲動
- **AND** Agents、設定、版本與即時狀態 footer SHALL 保持完整顯示於視窗底部

#### Scenario: 視窗高度動態變更

- **WHEN** 使用者調整應用程式視窗高度
- **THEN** Sidebar SHALL 依新的 viewport 高度重新分配主要導覽區的可用高度
- **AND** 品牌區與 footer SHALL 不重疊、不被裁切，且不要求切換至全螢幕才能看到

### Requirement: 收折項目不被邊界裁切

收折狀態下，導覽與專案項目的背景區塊 SHALL 完整顯示於側欄可視範圍內，不得被側欄邊框或 overflow 裁切。

#### Scenario: 收折後項目完整顯示

- **WHEN** 側欄處於收折狀態且某專案項目為 active
- **THEN** 該項目的高亮背景與圓角完整可見，右緣不被裁切

### Requirement: 導覽分隔線兩態一致

Dashboard 與釘選專案區之間的分隔線 SHALL 在展開與收折兩種狀態下皆顯示。

#### Scenario: 展開狀態顯示分隔線

- **WHEN** 側欄處於展開狀態且存在釘選專案
- **THEN** Dashboard 與釘選專案區之間顯示分隔線，與收折狀態一致
