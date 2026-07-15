# Tasks: Integration Card Collapse

## 1. 元件邏輯（SettingsView.tsx）

- [ ] 1.1 新增 `expandedProviders` useState（Record<string, boolean>，預設空物件）與 toggle 函式；展開判定為 `expandedProviders[provider] ?? Boolean(lastError)`（一般卡片預設收起、錯誤卡片預設展開）
- [ ] 1.2 卡片標題列改為可點擊切換：整列 onClick、`aria-expanded`、title 使用新增的 expand/collapse locale key
- [ ] 1.3 收起狀態：badges 後方加入最後事件時間摘要（formatDateTime，無事件時顯示 noEvent 文案）、右側僅顯示 chevron；隱藏操作按鈕與內容區
- [ ] 1.4 展開狀態：維持現有操作按鈕與 provider-integration-grid / lastError 區塊；操作按鈕容器 stopPropagation 避免觸發收折

## 2. 樣式（App.css）

- [ ] 2.1 新增收折相關 class（摘要時間文字、chevron 與旋轉過渡動畫），遵循 sessionhub-minimal-ui 設計 token
- [ ] 2.2 確認錯誤卡片被手動收起時 `provider-integration-card--error` 錯誤樣式仍清楚可辨

## 3. 文案（locales）

- [ ] 3.1 zh-TW.ts / en-US.ts 新增 `settings.integrations.actions.expand`、`settings.integrations.actions.collapse` 文案

## 4. 驗證

- [ ] 4.1 手動驗證：一般卡片預設收起、錯誤卡片預設展開、點擊切換、各卡片獨立、操作按鈕不觸發收折、無版本/無事件的顯示、錯誤卡片手動收起後可辨識
- [ ] 4.2 執行 `openspec validate --change integration-card-collapse` 確認 spec 格式正確
