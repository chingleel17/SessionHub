## ADDED Requirements

### Requirement: 互動元件 hover 與 active 視覺回饋

所有共用互動元件（Button 各 variant、IconButton、Select、checkbox）SHALL 具備可辨識的 hover 與 active 視覺回饋，且狀態切換 SHALL 具備平滑過渡動畫；在 `prefers-reduced-motion` 環境下 SHALL 停用過渡與位移動畫。

#### Scenario: hover 任一 Button variant

- **WHEN** 使用者將指標停留於 primary、secondary、ghost 或 danger 任一 variant 的按鈕
- **THEN** 按鈕呈現與預設狀態可明顯區分的背景或邊框變化，並以短過渡動畫（約 120–200ms）切換

#### Scenario: 按下按鈕的 active 回饋

- **WHEN** 使用者按下（mousedown / 鍵盤觸發）任一共用按鈕
- **THEN** 按鈕呈現 active 回饋（背景加深或輕微下沉）

#### Scenario: hover select 與 checkbox

- **WHEN** 使用者將指標停留於共用 Select 或 checkbox
- **THEN** 控制項顯示 hover 邊框或背景變化，非僅游標形狀改變

#### Scenario: hover 趨勢圖圖例切換鈕

- **WHEN** 使用者將指標停留於趨勢圖下方的數據線圖例切換鈕
- **THEN** 切換鈕顯示 hover 背景變化（subtle 強調底），active（已選取）狀態以 subtle 底與邊框呈現，且按鈕內文字水平置中

#### Scenario: 減少動態偏好

- **WHEN** 作業系統啟用 reduce motion
- **THEN** 上述過渡與位移動畫停用，狀態變化立即套用

### Requirement: 深色底按鈕 hover 維持文字對比

primary 與 danger 等白色（淺色）文字按鈕的 hover 樣式 SHALL 使用同色系加深或提亮的背景，文字與背景 contrast ratio SHALL ≥ 4.5:1；不得改用與文字對比不足的淺色背景。

#### Scenario: hover primary 按鈕（如「儲存設定」）

- **WHEN** 使用者 hover primary 按鈕
- **THEN** 背景維持 primary 色系（加深或提亮），白色文字清晰可辨，contrast ratio ≥ 4.5:1

#### Scenario: hover danger 按鈕

- **WHEN** 使用者 hover danger 按鈕
- **THEN** 背景維持 error 色系變化，文字對比 ≥ 4.5:1

### Requirement: 既有按鈕遷移至共用按鈕體系

畫面中重複使用的既有按鈕樣式（如 `ghost-button` 等自訂 class）SHALL 遷移至共用 `ui-button` 體系，使 hover、focus-visible、disabled 規則全站一致。

#### Scenario: 舊樣式按鈕遷移後

- **WHEN** 任一原使用自訂按鈕 class 的畫面渲染
- **THEN** 按鈕套用共用 `ui-button` 對應 variant 的完整狀態樣式（含 hover 與 focus-visible）

### Requirement: 設定頁 quota 手動刷新為圖示按鈕

設定頁 provider quota 監控卡片的手動刷新操作 SHALL 以共用 IconButton（refresh 圖示）呈現，並提供 i18n tooltip 與 accessible name。

#### Scenario: 設定頁刷新操作呈現

- **WHEN** 使用者開啟設定頁且 quota 監控啟用
- **THEN** 監控卡片標題列顯示 refresh 圖示按鈕（非文字按鈕），hover 顯示「立即刷新」tooltip

#### Scenario: 點擊刷新圖示

- **WHEN** 使用者點擊 refresh 圖示按鈕
- **THEN** 觸發與原文字按鈕相同的 quota 手動刷新行為
