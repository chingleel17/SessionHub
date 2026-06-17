## Context

各 provider（claude / codex / copilot）的 hook 目前以「每事件雙腳本」維護：`.ps1`（Windows 主軌）+ `.sh`（其餘平台 / fallback）。兩份語意必須手動同步，Copilot 6 事件即 12 份檔案，三 provider 合計三十餘份。`.sh` 軌依賴 Git Bash 專屬的 `jq`（JSON 解析）、`bc`（退避 sleep 計算）與 GNU `date -d`（unix ms 轉 ISO，BSD/macOS date 語法不同會走 fallback）。Rust 端 `provider/*.rs` 以 `include_str!` 嵌入腳本並寫至 provider 原生目錄，設定檔同時塞 `powershell` 與 `command`（或 `command` / `commandWindows`）兩欄。

開發者使用 AI agent（Copilot CLI、Codex CLI 本身即 Node 生態）幾乎必有 Node.js，故可改以單一 Node.js CLI 為主軌，消除雙軌主邏輯與外部依賴。

## Goals / Non-Goals

**Goals:**
- 將 hook 主邏輯收斂為單一 `node <script.js>` 主軌，每事件主邏輯只維護一份。
- 用 Node 原生 `JSON.parse`、`Date.prototype.toISOString()` 消除 `jq` / `bc` / `date -d` 依賴（主軌）。
- 保留 `.sh` 作為無 node 環境的 fallback。
- 沿用既有 `include_str!` + 寫檔部署機制，零新增打包流程。

**Non-Goals:**
- 不引入 Tauri sidecar 或預編譯 binary（`pkg` / `bun --compile` / Node SEA）。
- 不改變 bridge events.jsonl 的 record 格式（version 4 結構維持）。
- 不重寫 sh fallback 邏輯，僅維持其現有行為。
- 不移除 `jq-dependency-check`（sh fallback 仍需 jq）。

## Decisions

### 決策一：以 `node <script.js>` 為主軌，不用 sidecar
hook 是 provider CLI 在 SessionHub **未執行時**也要能獨立 spawn 的程序，Tauri sidecar 的生命週期管理、stdin/stdout 橋接、權限全用不到，只會引入 `externalBin` 設定與 target-triple 命名（`hook-x86_64-pc-windows-msvc.exe`）的複雜度。`.js` 沿用現有 `include_str!` + 寫至 provider 目錄機制即可被 `node` 直接執行。

- 替代方案：預編譯 binary → CI 需產 win/mac/linux 三平台、app 體積增加，收益不抵成本。
- 替代方案：Tauri sidecar → 對「app 未跑時的獨立 hook」場景無價值，徒增複雜度。

### 決策二：保留 sh 作 fallback，移除 ps1
主軌統一 `node hook.js` 後，Windows 也走 node（裝 Copilot/Codex CLI 者必有 node），ps1 失去存在意義，移除以降低維護軌數（ps1+sh → js+sh）。sh 保留作為無 node 時的退路，行為與依賴維持現狀。

- 替代方案：三軌（js + ps1 + sh）→ 維護成本最高，與簡化初衷相違。
- 替代方案：只留 js、移除 sh → 無任何退路，與「保留 fallback」決策相違，風險過高。

### 決策三：共用 `modules/record-event.js`
比照現有 `record-event.sh`，將 payload 讀取、欄位擷取、bridge record 組裝、retry 退避集中於單一 Node 模組，各事件入口腳本 `require` 之，避免邏輯重複。

### 決策四：升 `HOOK_SCRIPT_VERSION` 觸發重新安裝
既有安裝的 hook 設定指向舊的 ps1/sh 主軌；升版號使 `install_hook_scripts` 於下次安裝/更新時改寫設定檔為 node 主軌並寫出 `.js`、清除 `.ps1`。

## Risks / Trade-offs

- [使用者無 node 或版本過舊，主軌 `node hook.js` 靜默失敗] → 設定檔保留 sh `command` 欄位作 fallback；provider 自身的 command/powershell 雙欄機制在無 node 時可退回 sh。
- [移除 ps1 為 BREAKING，既有 Windows 安裝會改走 node] → 升版號觸發自動改寫；安裝流程覆寫舊設定，使用者重新安裝即生效。
- [sh fallback 仍依賴 jq，未完全消除外部依賴] → 屬刻意取捨：主軌已無依賴，sh 僅為退路，`jq-dependency-check` 的提示機制保留。
- [provider 設定檔欄位語意（powershell/command/commandWindows）各 provider 不一致] → 逐 provider 確認其 hook schema 後再改，避免一次全改踩雷；實作順序為 Copilot 先行驗證再套用 codex/claude。

## Migration Plan

1. 先實作 Copilot：新增 `*.js` + `record-event.js`，Rust 改接、移除 ps1，升版號，端到端驗證 hook 觸發寫入 events.jsonl 正確。
2. 驗證通過後，以同模式套用至 codex、claude。
3. 各 provider 重新安裝整合，確認設定檔主軌為 node、fallback 為 sh、無殘留 ps1 欄位。
4. Rollback：還原 `include_str!` 目標與 `render_*_hook_command`、保留 ps1 檔案即可回復舊雙軌。
