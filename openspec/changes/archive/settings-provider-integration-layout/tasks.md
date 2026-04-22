## 1. SettingsView 結構調整

- [x] 1.1 將 `SettingsView` 的 provider integration 區塊從一般設定表單中拆出，改為獨立卡片或等效寬版區塊
- [x] 1.2 保留既有 provider integration 操作與資料顯示內容，僅調整區塊層級與 JSX 結構

## 2. 設定頁寬版排版

- [x] 2.1 調整 `App.css` 的 settings layout，讓一般設定與 provider integration 在桌面寬畫面下使用不對稱版面
- [x] 2.2 調整 provider integration card 內部排版，明確分離 header、actions、metadata grid 與 error block

## 3. 響應式回退與驗證

- [x] 3.1 補上較窄視窗下的堆疊式布局規則，確保 provider integration 內容仍可完整閱讀與操作
- [x] 3.2 執行 `bun run build` 驗證前端型別與編譯
- [x] 3.3 手動驗證：桌面寬畫面下 provider integration 不再擠在窄卡片中，窄畫面下可正常回退

## 4. Provider 路徑折疊與複製

- [x] 4.1 provider integration card 的 configPath / bridgePath 預設折疊，不常駐佔版面
- [x] 4.2 summary 列顯示欄位名稱 + 複製按鈕，路徑不存在時按鈕 disabled
- [x] 4.3 新增 `settings.integrations.actions.copy` i18n key（zh-TW / en-US）
- [x] 4.4 新增對應 CSS（`.provider-path-summary`、`.provider-path-label`、`.provider-path-copy`）

## 5. Bridge JSONL 解析容錯

- [x] 5.1 新增 `coerce_json_string()` 輔助函式，將 array 欄位 join 為 string
- [x] 5.2 `read_last_bridge_record` 改用 `serde_json::Value` 手動提取欄位，對所有 Optional 欄位套用 coerce
- [x] 5.3 修正 "invalid type: sequence, expected a string" 解析失敗根因
- [x] 5.4 `cargo test bridge` 通過（3/3）

## 6. Bridge 失敗 Fallback

- [x] 6.1 `create_provider_bridge_watcher` 事件處理失敗時，仍呼叫 `emit_provider_refresh` fallback
- [ ] 6.2 手動驗證：bridge jsonl 格式異常時，session 列表仍正常更新
