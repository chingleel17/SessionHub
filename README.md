# SessionHub

> 一個 Windows 桌面應用程式，統一管理 GitHub Copilot CLI、OpenCode、Codex、Claude Code、Antigravity（Google Gemini）等多家 AI coding 工具的 Session。

![Version](https://img.shields.io/badge/version-0.1.6-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![Stack](https://img.shields.io/badge/stack-Tauri%202%20%2B%20React%2019%20%2B%20Rust-orange)

---

## 為什麼需要這個工具？

使用多種 AI coding CLI/IDE（GitHub Copilot CLI、OpenCode、Codex、Claude Code、Antigravity）一段時間後，各家工具會在各自的資料目錄下累積大量 session 資料。這些 session 分散在不同工具與不同專案之間，既沒有統一的檢視介面，也很難掌握每家工具的用量配額還剩多少。

**SessionHub 解決了以下問題：**

- 無法一覽所有工具、所有專案的 session
- 不知道各家 provider 的用量配額（quota）還剩多少、何時重置
- 不知道哪些 session 有進行中的 `plan.md` 計畫或 OpenSpec 變更
- 每次要重新開啟 session 都需要手動查指令、切終端
- Session 越來越多卻沒有統一的封存、標籤、統計方式
- 各專案的 AGENTS.md / CLAUDE.md / Skills / MCP 設定要手動比對是否同步

---

## 支援的 Provider

| Provider | 說明 |
| --- | --- |
| **GitHub Copilot CLI** | 掃描 `~/.copilot/session-state/`，支援 session 讀取、hook 整合 |
| **OpenCode** | 掃描 session/message/part 目錄，支援統計指標與 hook 整合 |
| **Codex** | 掃描 session 資料，支援 hook 整合與信任狀態偵測（codex trust） |
| **Claude Code** | 掃描 `~/.claude/`，支援 hook 整合、用量配額監控 |
| **Antigravity（Google Gemini）** | 掃描 `~/.gemini/` brain roots，透過本機 language server RPC 取得用量配額 |

各 provider 皆可在設定頁個別啟用/停用，並各自設定資料根目錄。

---

## 主要功能

| 功能 | 說明 |
| --- | --- |
| **多 Provider Session 掃描** | 同時掃描並列出五種 provider 的 session，顯示專案路徑、Git 分支、摘要、建立/更新時間 |
| **依專案分頁** | 每個專案（cwd）開啟獨立分頁，方便跨專案、跨 provider 切換 |
| **搜尋與篩選** | 關鍵字搜尋、標籤多選篩選、專案釘選、排序（時間／名稱） |
| **Dashboard** | 統計總覽、看板（Kanban）檢視 session 進行狀態（進行中/等待回應/閒置/已完成）、活動圖表 |
| **Session 統計分析** | Token 用量、花費、互動次數等分析圖表，可依週/月切換 |
| **用量配額監控（Quota）** | 狀態列圓環顯示即時用量百分比（變色警示），Dashboard 依模型群組分組顯示各 provider 配額視窗與重置時間 |
| **Hook 整合** | 一鍵安裝／偵測／解除安裝各 provider 的事件 hook（bridge），支援 targeted 局部刷新與原生通知 |
| **通知** | Agent 需要介入時、session 結束時跳出系統通知 |
| **系統匣（Tray）** | 可最小化至系統匣背景執行 |
| **MCP 設定管理** | GUI 新增/編輯/刪除/啟停用各平台的 MCP Server 設定（HTTP/SSE、npx、本機執行檔、自訂 JSON） |
| **Agents 設定管理** | 管理 AGENTS.md／CLAUDE.md 雙檔源、Skills、Commands，顯示各平台載入/同步狀態並可一鍵同步 |
| **Plans & Specs 瀏覽** | 瀏覽專案內 OpenSpec 變更（active/archived）、規格與 `.sisyphus` 計畫/筆記/證據，支援 Tree/List/Cols 檢視模式 |
| **一鍵開啟終端** | 直接以指定終端（pwsh/PowerShell）在 session 的工作目錄開啟 |
| **複製 Session 指令** | 一鍵複製依 provider 對應的重新進入指令 |
| **封存／刪除** | 封存不再需要的 session，或永久刪除 |
| **Plan 編輯器** | 在 app 內查看、編輯 `plan.md`，支援 Markdown 預覽；亦可用外部編輯器開啟 |
| **即時更新** | 監聽各 provider 的 session 目錄與 hook 事件，新增或修改時自動刷新列表 |
| **備註與標籤** | 為每個 session 加入個人備註與標籤，存於本地資料庫 |
| **多語系** | 支援繁體中文、English 介面切換 |

---

## 系統需求

- **作業系統**：Windows 10 / 11（x64）
- **終端機**：PowerShell 7（pwsh）或 Windows PowerShell
- 至少安裝並使用過以下其中一種工具，才有 session 可供管理：
  - GitHub Copilot CLI
  - OpenCode
  - Codex CLI
  - Claude Code
  - Antigravity（Google Gemini）

---

## 安裝方式

### 方式一：使用安裝檔（建議）

1. 前往 [Releases](../../releases) 下載最新版本
2. 執行 `.msi` 或 `-setup.exe` 安裝檔
3. 依照安裝精靈完成安裝
4. 從開始選單或桌面捷徑開啟 **SessionHub**

### 方式二：從原始碼建置

**前置需求：**

- [Rust](https://rustup.rs/)（stable toolchain）
- [Node.js](https://nodejs.org/) 22+
- [Bun](https://bun.sh/)

```bash
# 安裝相依套件
bun install

# 開發模式（熱重載）
bun run tauri dev

# 建置安裝檔
bun run tauri build
```

建置完成後，安裝檔位於：

```
src-tauri/target/release/bundle/
  msi/    → SessionHub_0.1.6_x64_en-US.msi
  nsis/   → SessionHub_0.1.6_x64-setup.exe
```

---

## 首次設定

1. 開啟應用程式後，前往左側 Sidebar 的「**設定**」
2. 在「Provider」區塊啟用需要的工具，並確認各自的資料根目錄（例如 Copilot 根目錄、Claude Code 資料目錄、Antigravity 根目錄等）是否正確
3. 設定**終端機路徑**（點擊「自動偵測」讓 app 自動找到 pwsh/PowerShell）
4. 選填**外部編輯器路徑**（用於開啟 plan.md，點擊「偵測 VSCode」自動填入）
5. 依需求開啟用量配額監控、通知、系統匣最小化等選項
6. 儲存設定，回到主頁即可看到所有 session

---

## 使用說明

### 查看所有 Session

- 主頁（Dashboard）顯示統計資訊、看板檢視與最近活動
- 點擊左側 Sidebar 的專案分頁進入完整列表
- 使用頂部搜尋列過濾，或點選標籤篩選

### 開啟 Session

選擇任一 session，點擊：

- **開啟終端**：在該專案目錄開啟終端機
- **複製指令**：複製對應 provider 的 session 重新進入指令，貼到任何終端執行

### 管理 Plan 與 OpenSpec

- Session 卡片上若含 `plan.md`，可展開 **Plan 編輯器** 直接在 app 內查看與編輯，或以外部編輯器開啟
- 「Plans & Specs」頁籤可瀏覽專案的 OpenSpec 變更與規格、`.sisyphus` 計畫資料

### 管理 Agents 設定

- 「Agents」頁籤可檢視/編輯 AGENTS.md、CLAUDE.md、Skills、Commands、MCP 設定
- 顯示各平台的載入/同步狀態（一致／需同步／未安裝），可一鍵套用同步

### 監控用量配額

- 狀態列圓環會即時顯示目前 provider 的用量百分比並依用量變色
- Dashboard 的用量面板依模型群組分組顯示各配額視窗、剩餘額度與重置時間

### 封存與刪除

- **封存**：將 session 移至封存區，不再顯示於主列表（可在專案頁面切換「顯示封存」查看）
- **刪除**：永久移除 session 資料夾（不可復原）

---

## 資料存放位置

| 資料 | 路徑 |
| --- | --- |
| 應用程式設定 | `%APPDATA%\SessionHub\settings.json` |
| 備註與標籤／快取（SQLite） | `%APPDATA%\SessionHub\metadata.db` |
| Hook 腳本 | `%APPDATA%\SessionHub\.claude\hooks\`（及各 provider 對應目錄） |
| GitHub Copilot CLI Sessions | `~/.copilot/session-state/`（可自訂） |
| OpenCode Sessions | `<root>/session|message|part/`（可自訂） |
| Codex Sessions | Codex 資料目錄（可自訂） |
| Claude Code Sessions | `~/.claude/`（可自訂） |
| Antigravity Sessions | `~/.gemini/`（brain roots + `agyhub_summaries_proto.pb`，可自訂） |

---

## 技術棧

- **前端**：React 19 + TypeScript + Vite + React Query
- **後端**：Rust + Tauri 2
- **資料庫**：SQLite（rusqlite，bundled）
- **即時監控**：notify crate（filesystem watcher）+ 各 provider hook bridge
- **UI**：純 CSS（無 CSS 框架），統一設計 token（圓角/邊框/陰影/玻璃/動畫）

---

## 開發

```bash
# 執行 Rust 單元測試
cd src-tauri && cargo test

# 前端型別檢查
bun run build
```

---

## License

MIT
