## Context

`SettingsView` 目前把一般設定欄位與 provider integration 區塊都放在同一張 `info-card` 中。雖然設定頁外層已有 `settings-layout` grid，但實際只渲染一張卡片，因此 provider integration 內容仍被限制在窄欄寬度內，導致狀態 badge、操作按鈕與多條路徑資訊密集換行。

這次變更不調整 provider integration 的資料模型或操作流程，只重新定義設定頁的資訊架構與響應式排版，讓現有內容在桌面寬畫面下更容易閱讀與操作，同時保留較小視窗下的堆疊式布局。

## Goals / Non-Goals

**Goals:**

- 將一般設定與 provider integration 管理拆成獨立卡片或區塊。
- 在桌面寬畫面下讓 provider integration 區塊使用主要內容寬度，而不是被限制在單一卡片窄欄。
- 讓單一 provider 卡片內的狀態、操作與 metadata 具備更穩定的版面分區。
- 保持現有操作能力與資料來源不變，只改善資訊呈現。

**Non-Goals:**

- 不新增新的 provider integration 狀態或操作按鈕。
- 不改動後端 bridge / plugin 偵測、安裝、更新或 watcher 邏輯。
- 不重做整個設定頁的所有欄位，只聚焦 provider integration 區塊與其周邊排版。

## Decisions

### D1：設定頁改為多卡片資訊架構

**決定**：把 provider integration 從一般設定表單中拆出，作為獨立的設定卡片；一般設定維持較緊湊的欄位編排，provider integration 則放在更寬的區域。

**理由**：

- provider integration 資訊密度高，與一般路徑設定混在一起會造成掃讀困難。
- 拆成獨立卡片後，可以單獨控制寬度、間距與欄位配置，不影響其他設定欄位。

**替代方案**：

- 保留單一卡片、只調大寬度：仍會讓一般設定欄位過寬，資訊層次也不夠清楚。

### D2：桌面寬畫面採不對稱版面

**決定**：在寬畫面下採兩欄或等效的不對稱 grid，讓一般設定停留在較窄欄，provider integration 區塊佔用較寬欄或整列寬度。

**理由**：

- 使用者的主要痛點是右側大量留白未被使用。
- provider integration 的路徑、錯誤訊息與多個按鈕需要更大的水平空間。

**替代方案**：

- 所有卡片都平均分欄：仍可能讓 provider integration 區塊偏窄，改善有限。

### D3：provider integration 卡片內部採區塊式排版

**決定**：每個 provider card 需清楚分成 header、actions、metadata grid、error block；metadata grid 在寬畫面下可使用多欄，窄畫面下自動堆疊。

**理由**：

- 現有資訊已足夠，只是需要更可預測的排版結構。
- 這種分區方式可避免 badge、按鈕與長路徑彼此擠壓。

## Risks / Trade-offs

- **[桌面與窄畫面切換時版面不一致]** → 以明確 breakpoint 定義寬版與堆疊版規則，避免中間尺寸出現半擠壓狀態。
- **[拆卡後視覺層次過重]** → 沿用現有 `info-card` 視覺系統，只調整間距與欄寬，不新增過多裝飾。
- **[CSS 變更影響其他設定欄位]** → 優先新增 provider integration 專用 class，避免覆寫通用 `field-group` / `info-card` 行為。

## Migration Plan

1. 調整 `SettingsView` 結構，將 provider integration 區塊從一般設定卡片抽出。
2. 新增或更新對應 CSS，定義設定頁寬版與堆疊版行為。
3. 以 `bun run build` 驗證型別與編譯。
4. 手動檢查桌面寬畫面與較窄視窗下的設定頁布局。

## Open Questions

- provider integration 區塊在桌面寬畫面下應採「右側寬欄」還是「下方全寬卡片」，可在實作時以內容密度最高的方案為準，但需滿足不再擠在窄欄中的要求。
