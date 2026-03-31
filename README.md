# SessionHub

> 一個 Windows 桌面應用程式，讓你輕鬆管理所有 GitHub Copilot CLI 的 Session。

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)
![Stack](https://img.shields.io/badge/stack-Tauri%202%20%2B%20React%20%2B%20Rust-orange)

---

## 為什麼需要這個工具？

使用 GitHub Copilot CLI 一段時間後，`~/.copilot/session-state/` 目錄下會累積大量 session 資料夾。這些 session 分散在不同專案之間，既沒有統一的檢視介面，也沒有方法快速回到某個 session 繼續工作。

**SessionHub 解決了以下問題：**

- 無法一覽所有 session 及其對應專案
- 不知道哪些 session 有進行中的 `plan.md` 計畫
- 每次要重新開啟 session 都需要手動複製指令
- Session 越來越多卻沒有封存或刪除的管理方式

---

## 主要功能

| 功能                    | 說明                                                                   |
| ----------------------- | ---------------------------------------------------------------------- |
| **Session 列表**        | 掃描並列出所有 Copilot session，顯示專案路徑、摘要、建立時間、更新時間 |
| **依專案分頁**          | 每個專案（cwd）開啟獨立分頁，方便跨專案切換                            |
| **搜尋與篩選**          | 關鍵字搜尋、標籤多選篩選、排序（時間／名稱）                           |
| **Dashboard**           | 統計總覽：session 數量、近期活動、專案分佈                             |
| **一鍵開啟終端**        | 直接以指定終端（pwsh/PowerShell）在 session 的工作目錄開啟             |
| **複製 Session 指令**   | 一鍵複製重新進入該 session 的 Copilot 指令                             |
| **封存／刪除**          | 封存不再需要的 session，或永久刪除                                     |
| **Plan 編輯器**         | 在 app 內查看、編輯 `plan.md`，支援 Markdown 預覽                      |
| **外部編輯器開啟 Plan** | 以 VSCode 或自訂編輯器開啟 `plan.md`                                   |
| **即時更新**            | 監聽 session-state 目錄，新增或修改時自動刷新列表                      |
| **備註與標籤**          | 為每個 session 加入個人備註與標籤，存於本地資料庫                      |
| **多語系**              | 支援繁體中文介面                                                       |

---

## 系統需求

- **作業系統**：Windows 10 / 11（x64）
- **GitHub Copilot CLI** 已安裝，session-state 路徑為 `~/.copilot/session-state/`（可自訂）
- **終端機**：PowerShell 7（pwsh）或 Windows PowerShell

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
  msi/    → SessionHub_0.1.0_x64_en-US.msi
  nsis/   → SessionHub_0.1.0_x64-setup.exe
```

---

## 首次設定

1. 開啟應用程式後，前往左側 Sidebar 的「**設定**」
2. 確認 **Copilot 路徑**（預設 `~/.copilot`）是否正確
3. 設定**終端機路徑**（點擊「自動偵測」讓 app 自動找到 pwsh/PowerShell）
4. 選填**外部編輯器路徑**（用於開啟 plan.md，點擊「偵測 VSCode」自動填入）
5. 儲存設定，回到主頁即可看到所有 session

---

## 使用說明

### 查看所有 Session

- 主頁（Dashboard）顯示統計資訊與最近活動
- 點擊左側 Sidebar 的「**Sessions**」進入完整列表
- 使用頂部搜尋列過濾，或點選標籤篩選

### 開啟 Session

選擇任一 session，點擊：

- 🖥 **開啟終端**：在該專案目錄開啟終端機
- 📋 **複製指令**：複製 `gh copilot session <id>` 指令，貼到任何終端執行

### 管理 Plan

Session 卡片上若有 📄 圖示，代表該 session 包含 `plan.md`：

- 點擊 session 展開 **Plan 編輯器**，可直接在 app 內查看與編輯
- 點擊「**外部開啟**」以 VSCode 或指定編輯器開啟

### 封存與刪除

- **封存**：將 session 移至封存區，不再顯示於主列表（可在專案頁面切換「顯示封存」查看）
- **刪除**：永久移除 session 資料夾（不可復原）

---

## 資料存放位置

| 資料                 | 路徑                                  |
| -------------------- | ------------------------------------- |
| 應用程式設定         | `%APPDATA%\SessionHub\settings.json`  |
| 備註與標籤（SQLite） | `%APPDATA%\SessionHub\metadata.db`    |
| Copilot Sessions     | `~/.copilot/session-state/`（可自訂） |

---

## 技術棧

- **前端**：React 18 + TypeScript + Vite + React Query
- **後端**：Rust + Tauri 2
- **資料庫**：SQLite（rusqlite，bundled）
- **即時監控**：notify crate（filesystem watcher）
- **UI**：純 CSS（無 CSS 框架）

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
