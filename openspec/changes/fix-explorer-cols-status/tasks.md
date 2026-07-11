## 1. buildTree 匯出群組 id 常數

- [x] 1.1 在 `src/utils/buildTree.ts` 匯出 `ARCHIVED_CHANGES_GROUP_ID = "openspec:archived-changes"` 常數，取代目前散落的字串字面值
- [x] 1.2 確認 `buildOpenSpecTree` 中 `openspec:archived-changes` 節點的 `id` 改用此常數賦值

## 2. resolveChangeAction 區分 Archived 群組

- [x] 2.1 在 `src/components/PlansSpecsView.tsx` 調整 `resolveChangeAction` 函式簽章，新增 `isArchived: boolean` 參數
- [x] 2.2 當 `isArchived` 為 true 且 `progress.done >= progress.total && progress.total > 0` 時，回傳新的「已封存」action（`label: "已封存"`, `tone: "done"`, 不含可複製的 slash command，例如 `command: ""` 或以獨立欄位標示不可複製）
- [x] 2.3 調整 Cols 模式渲染端可複製按鈕（約第 670-683 行），當 action 無 command 時不渲染複製按鈕
- [x] 2.4 在 `renderColumnsPanel` 呼叫 `resolveChangeAction(entryNode, ...)` 處（約第 663 行），依當前 `groupNode.id === ARCHIVED_CHANGES_GROUP_ID` 傳入正確的 `isArchived` 值

## 3. Specs 節點不套用 change 動作判斷

- [x] 3.1 在 `renderColumnsPanel` 迭代 `entryNodes` 渲染動作徽章區塊前（約第 662-686 行），新增判斷：當 `entryNode.icon === "spec"` 時跳過 `resolveChangeAction` 呼叫，不渲染動作徽章與複製按鈕
- [x] 3.2 確認 Specs 節點在 Cols 模式第二欄仍正常顯示名稱與可點擊選取行為，只是不出現「待 propose」等徽章

## 4. 驗證

- [x] 4.1 執行 `npm run build`（或專案既有的 typecheck 腳本）確認型別正確
- [x] 4.2 啟動應用程式，切換到 Cols 模式，確認 Archived Changes 群組內已完成的 change 顯示「已封存」而非「可封存」，且無可複製按鈕
- [x] 4.3 確認 Active Changes 群組內的「待 propose / 可 apply / 進行中 / 可封存」判斷與行為未受影響
- [x] 4.4 確認 Specs 群組內的規格項目不再顯示「待 propose」或其他 change 動作徽章
- [x] 4.5 確認 List 與 Tree 模式行為未受影響（兩者不呼叫 `resolveChangeAction`）
