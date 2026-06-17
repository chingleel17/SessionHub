# node-hook-scripts Specification

## Purpose
定義 Node.js CLI hook 入口腳本規格，作為各 provider hook 執行的主軌，取代 PowerShell 腳本，以 `node <script.js>` 方式被 provider 呼叫。

## Requirements

### Requirement: Node.js CLI hook 入口腳本

每個 provider（claude / codex / copilot）的 `hooks/<provider>/` 下 SHALL 提供 `.js` 入口腳本，與既有事件一一對應，作為 hook 執行的主軌，以 `node <script.js>` 方式被 provider 呼叫。

每支 `.js` 入口腳本 SHALL：
- 接受 `--bridge-path` 與 `--provider` 參數
- 從 stdin 讀取 JSON payload
- 呼叫共用模組的 bridge 寫入函式，將事件追加至 bridge events.jsonl
- 在 payload 為空或缺少 bridge-path 時以 exit 0 安全結束，不阻斷 provider 工作流程

#### Scenario: session 起始事件正確寫入

- **WHEN** session 起始的 `.js` 腳本以有效 JSON payload 透過 stdin 呼叫，並提供合法 `--bridge-path`
- **THEN** bridge 檔案中新增一行 JSON，包含對應的 `eventType`、正確的 `sessionId` 與 `cwd`

#### Scenario: 缺少 bridge-path 時安全退出

- **WHEN** `.js` 腳本未取得 `--bridge-path`
- **THEN** 腳本以 exit 0 結束，不寫入任何檔案、不報錯阻斷流程

#### Scenario: 空 payload 時安全退出

- **WHEN** stdin 傳入空白或無內容
- **THEN** 腳本以 exit 0 結束，不寫入 bridge 檔案

### Requirement: 共用 record-event.js 模組

`hooks/<provider>/modules/` 下 SHALL 提供 `record-event.js` 共用模組，集中 payload 讀取、欄位擷取、bridge record 組裝與寫入重試邏輯，供各事件入口腳本引用，避免邏輯重複。

模組 SHALL：
- 提供讀取 stdin payload 並解析為物件的函式（使用 Node 原生 `JSON.parse`）
- 提供由 payload 擷取 `sessionId`、`cwd`、`transcriptPath` 等欄位的函式（容許多個候選鍵名）
- 提供組裝並以 append 方式寫入 bridge record 的函式，record 維持既有 version 4 欄位結構
- 以原生 `Date.prototype.toISOString()` 產生 timestamp，不依賴外部工具

#### Scenario: 模組可被 require 引入

- **WHEN** 入口腳本執行 `require('./modules/record-event.js')`
- **THEN** bridge 寫入函式可被呼叫，不報錯

#### Scenario: record 維持 version 4 格式

- **WHEN** 模組組裝 bridge record
- **THEN** record 包含 `version=4`、`provider`、`eventType`、`timestamp`、`sessionId`、`cwd`、`sourcePath` 等既有欄位

#### Scenario: 不依賴 jq / bc / date 外部工具

- **WHEN** 在僅有 Node.js、無 `jq` 的環境執行 `.js` 主軌
- **THEN** payload 解析與 timestamp 產生皆正常，事件成功寫入 bridge 檔案

### Requirement: Node hook 腳本以嵌入方式部署

Rust provider 模組 SHALL 以 `include_str!` 嵌入 `.js` 入口腳本與 `record-event.js` 模組，並於安裝 / 更新整合時一併寫出至 provider 原生 hook scripts 目錄；`HOOK_SCRIPT_VERSION` SHALL 升版以觸發既有安裝重新寫出。

#### Scenario: 安裝後 js 腳本存在於正確路徑

- **WHEN** 呼叫 provider 的 hook scripts 安裝流程
- **THEN** hook scripts 目錄下包含各事件對應的 `.js` 檔案與 `modules/record-event.js`

#### Scenario: 升版後既有安裝被重新寫出

- **WHEN** 既有安裝的 hook script 版本低於內建 `HOOK_SCRIPT_VERSION`
- **THEN** 下次安裝 / 更新時重新寫出 `.js` 檔案並更新設定檔主軌指向 node
