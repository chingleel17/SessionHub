## 1. 前置整理

- [ ] 1.1 將時長格式化(`Xm` / `XhYm` / `Xh`)抽為共用 util(或複用 SessionStatsBadge 既有實作),SessionStatsBadge 與 SessionStatsPanel 共用
- [ ] 1.2 於 `src/locales/*.ts` 增修 label key:輸入/輸出 tokens 欄位標題、合計、「+N 更多」展開/收合文案(zh-TW 與其他語系檔同步)

## 2. SessionStatsPanel 重寫

- [ ] 2.1 重寫 `SessionStatsPanel.tsx` 為垂直三分區版面:摘要 stat row(output/input tokens、互動、工具調用、reasoning、時長,條件顯示規則不變)+ live 提示
- [ ] 2.2 實作 per-model 明細表:`modelMetrics` 非空時渲染(模型/輸入/輸出/成本),依 `requestsCost` 降冪,多模型附合計列,成本全 0 時隱藏成本欄;移除 `provider === "copilot"` 分支
- [ ] 2.3 實作 `modelMetrics` 為空時的退回:顯示 `modelsUsed` 逗號串,兩者皆空顯示無資料文案
- [ ] 2.4 實作工具調用 top 5 + 「+N」展開/收合(local state),移除固定高度捲動框

## 3. 樣式

- [ ] 3.1 重寫 `App.css` 的 `.stats-panel*` 區塊:移除 panel 與內部分區的 border/background/radius,改透明 + 分區 hairline;摘要 stat row 與明細表樣式一律使用設計 token
- [ ] 3.2 light / dark 雙主題實測:確認無露白、hairline 與文字對比正常、無多餘 scrollbar

## 4. 驗證

- [ ] 4.1 `npm run build`(tsc + vite)通過,無型別錯誤
- [ ] 4.2 實際開啟 app 驗證:copilot session(有成本)、claude session(per-model tokens)、opencode/codex session(退回 modelsUsed 名單)三種情境顯示正確
- [ ] 4.3 執行 `openspec validate --change redesign-session-stats-panel`
