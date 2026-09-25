## MODIFIED Requirements

### Requirement: TrendChart 折線趨勢圖元件

系統 SHALL 提供可隨容器調整大小的趨勢圖，同一座標軸只呈現同單位序列，支援精確數值提示與日期鑽取。

#### Scenario: 正常渲染折線
- **WHEN** 收到兩個以上時間桶
- **THEN** X 軸顯示時間，Y 軸顯示目前所選單位，序列以主題色、圖例文字區分
- **AND** hover 或鍵盤巡覽顯示日期、精確數值、單位與完整性

#### Scenario: 單一資料點
- **WHEN** 只有一個資料點
- **THEN** 顯示可聚焦單點與數值，不繪製虛構趨勢線

#### Scenario: 空資料
- **WHEN** 沒有可用事件
- **THEN** 依 coverage 區分無用量、尚在索引、未支援或錯誤，不顯示誤導零值曲線

#### Scenario: 無障礙支援
- **WHEN** 使用者以鍵盤或輔助技術操作
- **THEN** 可辨識圖表標題、巡覽資料點與啟動鑽取，並取得文字數值替代

### Requirement: 圖表顏色跟隨應用主題

圖表 SHALL 使用 CSS 主題 tokens 控制線條、填色、文字、tooltip 與焦點樣式，不以顏色作唯一資訊提示。

#### Scenario: 深色主題下圖表清晰可辨
- **WHEN** 切換深淺主題
- **THEN** 趨勢與排行保持可辨識對比，無硬編碼白底或不可見線條，圖例保留文字與符號

### Requirement: 可選折線顯示切換

趨勢圖 SHALL 提供 Token、估算 USD、點數的單位切換，以及所選單位內的序列顯示控制；Token 可選 total、input 或 output，並依總量、provider、model 分組，total 另提供互斥 input/output 類型拆分。

#### Scenario: 切換指標顯示
- **WHEN** 使用者切換單位或顯示序列
- **THEN** Y 軸與 tooltip 同步更新，至少保留一條可用序列
- **AND** 不將點數、美元與 Token 畫在同一軸

#### Scenario: Token 子集合與缺少事件成本
- **WHEN** 檢視 input/output 拆分或來源只有 session 點數彙總
- **THEN** cache/reasoning 以子集合明細呈現、不重複堆疊；無事件點數時顯示該趨勢不可得與原因

## REMOVED Requirements

### Requirement: PieChart 圓餅分布圖元件

**Reason**: 分析頁改用可排序且容易比較長尾項目的橫條排行。

**Migration**: Dashboard 移至精簡摘要，全域與專案分布採橫條排行；若其他介面仍使用舊元件，保留其實作而非強制刪除。

## ADDED Requirements

### Requirement: 可鑽取的專案與模型排行

分析 SHALL 提供按目前指標降冪排列的專案與模型橫條排行，顯示精確值、單位及可用值佔比，支援完整清單捲動與篩選鑽取。

#### Scenario: 長尾與單一項目
- **WHEN** 資料有許多項目或僅一個非零項目
- **THEN** 所有項目均可查閱，不遺失長尾；單一項目顯示已知值的 100%
- **AND** 部分涵蓋時佔比標示只代表已知資料

#### Scenario: 全零或未知
- **WHEN** 所有值為零或無已知值
- **THEN** 顯示相符空狀態或缺漏原因，不產生 NaN 百分比
