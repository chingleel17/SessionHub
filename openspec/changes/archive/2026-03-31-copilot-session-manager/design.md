## Context

目前 GitHub Copilot CLI 將所有 session 資料存於 `~/.copilot/session-state/<sessionID>/workspace.yaml`，開發者無任何 GUI 工具可管理。本應用程式針對 Windows 桌面環境開發，提供視覺化 session 管理介面。

技術選型評估後選擇 **Tauri 2 + React（TypeScript）**：
- Rust 後端負責檔案系統操作（讀取/封存/刪除/watch），安全且效能佳
- React 前端提供靈活的 UI，方便實現 Chrome 分頁風格與多主題
- Tauri bundle 產生單一 `.exe` 安裝檔，分發簡便

## Goals / Non-Goals

**Goals:**
- 讀取並顯示所有 Copilot session，支援依專案分組與篩選
- 對 session 執行：封存、刪除、開啟終端、複製開啟指令
- 支援對 session 新增自訂備註（notes）與標籤（tags），儲存於本地 SQLite DB
- 支援查看、App 內編輯（CodeMirror）、或開啟外部編輯器查看 session 的 `plan.md`
- Filesystem watch 即時監聽 `session-state/` 目錄變更，自動更新 UI
- Chrome 風格分頁：Dashboard 主頁 + 各專案分頁
- 可設定 Copilot 根目錄、終端路徑、外部編輯器路徑
- 建立多語系架構，初版先提供繁體中文（`zh-TW`）
- 亮色系 UI 主題，預留主題切換擴充點
- `workspace.yaml` 中的 `summary` 欄位作為 session 標題（fallback 到 session ID 前 8 碼）

**Non-Goals:**
- 不修改 Copilot CLI 本身行為
- 不支援 macOS / Linux（此版本）
- 不實作雲端同步或遠端 session 管理
- 不解析 plan.md 以外的 session 內部檔案

## Decisions

### D1：技術堆疊選擇 Tauri 2 + React（TypeScript）

**理由**：Rust 後端可直接操作檔案系統且安全，React 生態豐富易於實作複雜 UI（分頁、篩選、主題），Tauri 打包體積遠小於 Electron。

### D2：分頁架構 — 路由式分頁

- `/` → Dashboard
- `/project/:encodedCwd` → 專案分頁

### D3：封存實作 — 移動目錄至 `~/.copilot/session-state-archive/`

封存 = 將 `session-state/<id>/` 移動至 `session-state-archive/<id>/`，不破壞原始資料。

### D4：設定儲存位置 — `%APPDATA%\SessionHub\`

- `settings.json`：Copilot 根目錄、終端路徑、外部編輯器路徑等設定
- `metadata.db`：SQLite，存 session 的備註與標籤

### D5：workspace.yaml 解析

使用 Rust `serde_yaml` 解析，`summary` 不存在時，UI 顯示 session ID 前 8 碼作為標題。

### D6：終端開啟指令格式

`<terminal_path> -NoExit -Command "cd '<cwd>'"`

### D7：資料層架構 — 掃描 + SQLite 混合模式

- **來源資料（唯讀）**：每次啟動/重新整理時掃描 `session-state/` 目錄，解析 `workspace.yaml`
- **使用者 metadata（可寫）**：`%APPDATA%\SessionHub\metadata.db`（SQLite）

### D8：即時更新 — `notify` crate（Filesystem Watch）

使用 Rust `notify` crate 監聽 `session-state/` 目錄事件，透過 Tauri event 推送到前端。無 Copilot CLI 官方 hook，純 OS 檔案事件。

### D9：plan.md 編輯器 — CodeMirror 6 + marked

- **App 內編輯**：CodeMirror 6 + `marked`
- **外部編輯器**：依序偵測 `code`（VSCode）→ 使用者自訂 → Windows 系統預設

### D10：多語系架構 — 內建 dictionary provider

- 前端使用輕量字典式 i18n provider，不在初版引入大型 i18n 框架
- 資源檔放於 `src\locales\zh-TW.ts`
- 元件一律透過 `t("key")` 取文案，不直接硬編碼 UI 字串
- 初版預設語系固定為 `zh-TW`，後續可擴充 `en-US` 等資源檔

## Risks / Trade-offs

- **[Risk] workspace.yaml schema 未來變更** → 解析時對所有欄位使用 `Option<T>`
- **[Risk] 大量 session（1000+）效能** → 前端虛擬化列表，Rust 端非同步讀取
- **[Risk] 刪除操作不可逆** → UI 顯示確認對話框
- **[Risk] Filesystem watch 事件風暴** → 前端 debounce 事件處理
- **[Risk] plan.md 外部與 App 內同時編輯衝突** → 偵測到外部修改立即提示重載
- **[Risk] 初期直接寫死中文文案導致後續難以擴充** → 初始化階段即導入 i18n key 結構

## Migration Plan

全新應用程式，無現有資料需遷移。

## Open Questions

- `gh copilot session resume <id>` 是否為有效指令？
- 封存後的 session 是否需要在 UI 中顯示？預設隱藏，提供切換。
