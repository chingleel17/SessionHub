## Context

目前 `PlansSpecsView.tsx` 使用手風琴（accordion）模式：spec/change 項目展開後，內容預覽出現在清單項目正下方。當 OpenSpec 有 40+ specs 與多個 changes 時，使用者必須捲動整個清單才能看到文件內容，且選取不同文件需來回捲動。

畫面截圖顯示兩個問題：
1. 展開 change 後顯示 "Failed to read file"，錯誤訊息埋在清單中難以識別
2. 查看 spec 內容時需要捲動到底，閱讀效率低

參考 VSCode 的 Explorer 面板設計：左側固定寬度的樹狀導覽，右側固定的編輯/預覽區。

## Goals / Non-Goals

**Goals:**
- 將 PlansSpecsView 改為左右雙面板佈局（左：Explorer 樹 / 右：內容檢視）
- 左側面板：多層可收折樹狀結構（根節點 → 群組 → 文件），支援鍵盤導覽感知（滑鼠 hover 高亮）
- 右側面板：固定 Markdown 純文字顯示，文件內容佔滿可用空間且獨立捲動
- 左側寬度可調整（拖曳分隔線）或折疊（點擊 toggle 按鈕）
- 選取狀態持久（切換樹節點時右側立即更新，左側保持捲動位置）
- Props 介面不變（`PlansSpecsView` 對外 API 維持），不需改 App.tsx

**Non-Goals:**
- Markdown 語法高亮渲染（保持純文字）
- 文件編輯功能
- Rust backend 變更
- 鍵盤方向鍵導覽樹狀結構

## Decisions

### 決策 1：使用純 CSS flexbox 雙面板，不引入第三方 resizable 套件

**選項 A（採用）**：CSS `display: flex` + `--explorer-width` CSS 變數 + JS drag handler
- 優點：無新依賴、程式碼可控、符合專案「純 CSS，無 CSS 框架」慣例
- 缺點：需自行實作 drag resize，約 30 行 JS

**選項 B（未採用）**：引入 `react-resizable-panels`
- 優點：功能完整
- 缺點：增加依賴，與現有 CSS 架構不一致

### 決策 2：左側折疊後以 icon 欄位方式顯示（最小寬度 32px）

折疊時不完全消失，保留窄欄讓使用者可點擊展開，類似 VSCode activity bar 收折行為。

### 決策 3：樹狀節點結構（TreeNode 型別）

統一所有資料源（Sisyphus、OpenSpec Changes、Specs）為 `TreeNode` 陣列，讓 `ExplorerTree` 元件與資料源解耦：

```typescript
type TreeNode = {
  id: string;           // 唯一識別（路徑或 key）
  label: string;        // 顯示名稱
  icon?: string;        // 可選圖示（emoji 或文字符號）
  badge?: string;       // 可選 badge（如進度 "3/5"）
  children?: TreeNode[]; // 有 children = 群組節點；無 = 葉節點（可選取文件）
  selectable?: boolean; // 是否可觸發右側內容載入
  filePath?: string;    // 葉節點的實際檔案路徑
};
```

### 決策 4：保留 `ChangeItem` 的 specs 子項目展開

Change 節點下可顯示 `proposal.md`、`design.md`、`tasks.md`（以及 delta specs）作為子節點，使用者可直接選取，右側顯示對應內容。

### 決策 5：CSS 獨立成 `plans-specs-explorer.css`

為避免影響現有 CSS，新增獨立樣式檔，舊 CSS class 可保留（backward compatible）。

## Risks / Trade-offs

| 風險 | 緩解措施 |
|------|----------|
| 拖曳 resize 在低解析度螢幕重疊 | 設定最小寬度 160px（左）/ 200px（右），防止面板擠壓至不可用 |
| 樹狀節點過多導致左側過長 | 根節點預設折疊（除 Sisyphus Plans 與 Active Changes 外）|
| 舊版 spec 顯示問題（"Failed to read file"）| 右側面板統一以醒目錯誤狀態顯示（紅色 banner），而非埋在清單中 |
