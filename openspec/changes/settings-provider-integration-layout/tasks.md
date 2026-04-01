## 1. SettingsView 結構調整

- [ ] 1.1 將 `SettingsView` 的 provider integration 區塊從一般設定表單中拆出，改為獨立卡片或等效寬版區塊
- [ ] 1.2 保留既有 provider integration 操作與資料顯示內容，僅調整區塊層級與 JSX 結構

## 2. 設定頁寬版排版

- [ ] 2.1 調整 `App.css` 的 settings layout，讓一般設定與 provider integration 在桌面寬畫面下使用不對稱版面
- [ ] 2.2 調整 provider integration card 內部排版，明確分離 header、actions、metadata grid 與 error block

## 3. 響應式回退與驗證

- [ ] 3.1 補上較窄視窗下的堆疊式布局規則，確保 provider integration 內容仍可完整閱讀與操作
- [ ] 3.2 執行 `bun run build` 驗證前端型別與編譯
- [ ] 3.3 手動驗證：桌面寬畫面下 provider integration 不再擠在窄卡片中，窄畫面下可正常回退
