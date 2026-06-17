## 1. 後端：OpenSpecChange 新增 createdAt

- [x] 1.1 在 `src-tauri/src/types.rs` 的 `OpenSpecChange` struct 新增 `created_at: Option<String>` 欄位（`#[serde(rename_all = "camelCase")]` 已套用，前端收到 `createdAt`）
- [x] 1.2 在 `src-tauri/src/openspec_scan.rs` 的 `scan_openspec_change` 中，讀取 change 目錄下的 `.openspec.yaml`，解析 `created:` 行取出日期字串，填入 `created_at`；無檔案或無此欄位時填 `None`
- [x] 1.3 在 `src/types/index.ts` 的 `OpenSpecChange` 型別新增 `createdAt?: string | null`

## 2. CSS：List 與 Cols 名稱加粗

- [x] 2.1 在 `src/App.css` 的 `.explorer-list-row-name` 加入 `font-weight: 600`
- [x] 2.2 在 `src/App.css` 的 `.explorer-cols-entry-name` 加入 `font-weight: 600`

## 3. 前端：排序 state 與邏輯

- [x] 3.1 在 `PlansSpecsView.tsx` 定義 `type SortField = "progress" | "name" | "createdAt"` 與 `type SortDir = "asc" | "desc"`
- [x] 3.2 新增 `sortField` / `sortDir` state，以 `localStorage` 初始化（key: `explorer-sort:<projectCwd>`），預設 `{ field: "name", dir: "asc" }`
- [x] 3.3 實作 `sortChanges(changes: OpenSpecChange[]): OpenSpecChange[]` 函式：
  - `name`：`localeCompare` 字母序
  - `progress`：`done/total` 比值，`total===0` 視為 `-1`
  - `createdAt`：字串比較（`YYYY-MM-DD` 格式可直接比較），`null` 維持相對原序
- [x] 3.4 在 `buildOpenSpecTree` 呼叫前，對 `openspecData.activeChanges` 與 `archivedChanges` 各自套用 `sortChanges`

## 4. 前端：排序切換器 UI

- [x] 4.1 在 `PlansSpecsView.tsx` 的標頭區（view 切換器旁）新增排序切換器，三個按鈕：進度（`t("plansSpecs.explorer.sort.progress")`）、名稱（`t("plansSpecs.explorer.sort.name")`）、時間（`t("plansSpecs.explorer.sort.createdAt")`）
- [x] 4.2 每個按鈕顯示欄位標籤 + 方向圖示：選中欄位升冪顯示 `↑`，降冪顯示 `↓`；未選中顯示 `⇅`
- [x] 4.3 點擊邏輯：點擊已選中欄位切換 dir；點擊未選中欄位設為該欄位 `asc`；更新後寫入 localStorage
- [x] 4.4 在 `src/locales/zh-TW.ts`（或對應的 i18n 檔案）新增三個排序標籤翻譯 key

## 5. CSS：排序切換器樣式

- [x] 5.1 在 `src/App.css` 新增 `.explorer-sort-switcher`（排版：flex row，gap 2px）與 `.explorer-sort-btn` / `.explorer-sort-btn--active` 樣式，風格與 `.explorer-view-btn` 一致

## 6. 驗證

- [x] 6.1 TypeScript 型別檢查通過（`tsc --noEmit`）
- [x] 6.2 測試 List 模式 change 名稱加粗顯示
- [x] 6.3 測試 Cols 模式 change 名稱加粗顯示
- [x] 6.4 測試三種排序切換（名稱 / 進度 / 時間），升降冪切換正確
- [x] 6.5 測試排序設定在切換頁面後恢復
- [x] 6.6 驗證 `createdAt` 在後端正確填入（有 yaml 與無 yaml 兩種情境）
