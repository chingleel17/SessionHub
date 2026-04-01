# PROJECT KNOWLEDGE BASE

**App:** SessionHub — Windows 桌面應用程式，管理 GitHub Copilot CLI Sessions  
**Stack:** React 19 + TypeScript + Vite (frontend) · Rust + Tauri 2 (backend) · SQLite (rusqlite)  
**Generated:** 2026-03-31 · Branch: main

---

## STRUCTURE

```
copilot-session-manager/
├── src/              # React/TS 前端
│   ├── App.tsx       # 根元件（所有狀態與 invoke 呼叫集中於此）
│   ├── components/   # 純顯示元件，接收 props 驅動
│   ├── types/        # 共用 TypeScript 型別（index.ts）
│   ├── i18n/         # I18nProvider + 繁中翻譯
│   └── styles/       # 純 CSS（無 CSS 框架）
├── src-tauri/src/    # Rust 後端（lib.rs 單一大檔，802 行）
│   ├── lib.rs        # 所有 Tauri commands、業務邏輯、SQLite 操作、FS watcher
│   └── main.rs       # Tauri 入口，呼叫 session_hub_lib::run()
├── openspec/         # OpenAPI 規格文件
├── .opencode/        # OpenCode 工具設定（獨立 package.json）
├── index.html        # Vite HTML 入口
└── vite.config.ts    # Vite 設定
```

---

## WHERE TO LOOK

| 任務 | 位置 |
|------|------|
| 新增/修改 Tauri command | `src-tauri/src/lib.rs` → 找 `#[tauri::command]`，並在 `invoke_handler![]` 登記 |
| 前端呼叫 backend | `src/App.tsx` → `invoke<T>("command_name", {...})` |
| 前端狀態/查詢 | `src/App.tsx`（React Query + useState，無 Redux/Zustand） |
| 型別定義 | `src/types/index.ts`（frontend）；`src-tauri/src/lib.rs` 頂部 struct（backend） |
| 翻譯文字 | `src/i18n/` + `src/locales/` |
| UI 元件 | `src/components/`（每個 View/Dialog 對應一個 .tsx 檔） |
| 應用程式資料路徑 | `%APPDATA%\SessionHub\` → settings.json, metadata.db |
| Session 資料來源 | `~/.copilot/session-state/<session-id>/workspace.yaml` |

---

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `run()` | fn | lib.rs:697 | Tauri builder、插件登記、invoke_handler 入口 |
| `get_sessions` | command | lib.rs | 掃描 session-state 目錄，回傳 `Vec<SessionInfo>` |
| `save_settings` / `get_settings` | commands | lib.rs | 讀寫 `%APPDATA%\SessionHub\settings.json` |
| `upsert_session_meta` | command | lib.rs | SQLite 備註 + 標籤 upsert |
| `open_terminal` | command | lib.rs | 依終端機類型（pwsh/cmd/bash）開啟終端至 session 工作目錄 |
| `WatcherState` | struct | lib.rs:62 | Mutex 包裝 FS watcher（sessions + plan 各一） |
| `SessionInfo` | struct | lib.rs:28 | Rust 端序列化；前端對映 `src/types/index.ts` |
| `App` | component | src/App.tsx:58 | 所有業務邏輯、mutations、event listeners 在此 |
| `buildProjectGroups` | fn | src/App.tsx:26 | sessions → 依 cwd 分組成 `ProjectGroup[]` |

---

## CONVENTIONS

- **前後端型別映射**：Rust `#[serde(rename_all = "camelCase")]` → TS `camelCase`。新增欄位兩端都必須同步。
- **activeView 路由**：`"dashboard"` | `"settings"` | `{projectKey}`（Project Tab）。Plan 編輯器不在頂層路由，改為 ProjectView 內的子 Tab，以 session ID 為 key 識別。
- **React Query key 結構**：`["sessions", copilotRoot, showArchived]`（含參數，settings 更新會 invalidate）。
- **CSS**：純 CSS class，命名用 BEM-like 如 `tab-item`, `tab-item-project`, `workspace-header`。無 Tailwind/CSS Modules。
- **翻譯**：全部用 `t("key")` 取得，禁止 hardcode 中文字串在 JSX。
- **錯誤回報**：Rust commands 一律回傳 `Result<T, String>`；前端 catch 後呼叫 `showToast`。

---

## ANTI-PATTERNS（此專案禁止）

- 在子元件內直接呼叫 `invoke()` — 違反架構慣例，所有 IPC 集中在 App.tsx
- `as any` / `@ts-ignore` — TypeScript strict mode 啟用
- 在 Rust 中 `unwrap()` 而不處理錯誤（production code）
- Hardcode Windows 路徑分隔符 `\`（用 `PathBuf` 組合）
- main.rs 中加業務邏輯（只呼叫 `run()`）
- 在 `#[tauri::command]` 函式內寫邏輯（應委派給 `_internal` 輔助函式，方便測試）

---

## TAURI IPC PATTERN

```
frontend invoke("command_name", {snakeCaseArgs}) 
  → #[tauri::command] fn command_name(snake_case_args) → Result<T, String>
    → calls command_name_internal() for testability
```

每個 command 對應一個 `_internal` fn，讓 `#[cfg(test)]` 可直接測試。

---

## COMMANDS

```bash
# 開發（熱重載）
bun run tauri dev

# 前端型別檢查 + build
bun run build

# Rust 單元測試
cd src-tauri && cargo test

# 建置安裝檔（msi / nsis）
bun run tauri build
# 輸出：src-tauri/target/release/bundle/
```

---

## NOTES

- Windows ONLY：`USERPROFILE` 與 `APPDATA` 環境變數必定存在；`CREATE_NEW_CONSOLE` flag 防止 release build 彈出主控台視窗。
- `COPILOT_SESSION_MANAGER_APPDATA_OVERRIDE` 環境變數可覆蓋 AppData 路徑（測試用）。
- `src-tauri/target/` 是 Rust 編譯輸出，勿提交，大小可達數 GB。
- `.opencode/` 有獨立 `package.json`（`@opencode-ai/plugin`），非主專案相依。
