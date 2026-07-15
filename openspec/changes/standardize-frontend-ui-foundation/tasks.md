## 1. 前端品質基礎

- [ ] 1.1 確認 Oxlint 與 Bun 相容後，以套件管理指令新增 Oxlint，並建立符合現有 TypeScript/React 專案的設定檔。
- [ ] 1.2 在 `package.json` 新增 `lint` script，執行 lint 並修正本 change 觸及檔案的違規。
- [ ] 1.3 在 CI 的 Windows 前端 job 加入不可忽略的 `bun run lint` check，保留既有 build 與 Rust checks。

## 2. 圖示系統

- [ ] 2.1 確認 Lucide React 與 Bun 相容後，以套件管理指令新增 `lucide-react`。
- [ ] 2.2 建立集中圖示出口，提供既有操作名稱的 Lucide mapping 與統一預設尺寸/stroke 規則。
- [ ] 2.3 建立 ProviderIcon 或等效的統一 provider 識別元件，移除操作 UI 中的 Emoji 作為唯一 provider 識別。
- [ ] 2.4 遷移 Sidebar、SessionCard、Settings、Agents/MCP、Dialogs、Tray/Overlay 等高頻畫面的操作與導覽 icon，移除被取代的 inline SVG 與自製 path。
- [ ] 2.5 保留並確認 charts、quota rings、品牌 logo 與其他資料視覺化 SVG 未被誤替換。

## 3. 共用互動元件

- [ ] 3.1 在 `src/components/ui/` 建立 Button，支援 primary、secondary、ghost、danger、disabled 與 loading 狀態。
- [ ] 3.2 建立具 accessible name、tooltip、focus-visible 與 disabled 行為的 IconButton。
- [ ] 3.3 建立共用 Select 樣式/元件，統一原生 select 的高度、圓角、border、focus-visible 與 disabled 狀態。
- [ ] 3.4 整理現有 DropdownMenu 為共用 dropdown contract，確認 Escape、外部點擊關閉、焦點回復與 ARIA 行為一致。
- [ ] 3.5 將共用元件樣式接入既有 light/dark token，補足必要的 hover、focus、disabled 與 reduced-motion 規則。

## 4. 畫面遷移與文件

- [ ] 4.1 將 SessionCard 的操作按鈕遷移為共用 IconButton，維持既有 i18n tooltip 與 hover 顯示行為。
- [ ] 4.2 將重複的文字按鈕與選單觸發按鈕遷移為 Button、Select 或 Dropdown，避免新增重複 CSS。
- [ ] 4.3 更新 `src/components/AGENTS.md`、`src/AGENTS.md` 與 `CONTRIBUTING.md`，記錄圖示、primitive、inline SVG 例外與 lint 規則。

## 5. 驗證

- [ ] 5.1 執行 `bun run lint` 與 `bun run build`。
- [ ] 5.2 以亮色與暗色主題手動驗證 Button、IconButton、Select/Dropdown 的 hover、focus、disabled、keyboard 與窄寬度顯示。
- [ ] 5.3 執行既有 Rust 測試，確認純前端變更未造成 Tauri 專案回歸。
- [ ] 5.4 執行 `openspec validate standardize-frontend-ui-foundation --strict`，修正所有規格驗證問題。
