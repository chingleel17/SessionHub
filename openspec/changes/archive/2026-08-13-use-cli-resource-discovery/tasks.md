## 1. Discovery 型別與安全執行器

- [x] 1.1 新增 provider resource registry 與 camelCase discovery 型別，涵蓋 enabled provider order、kind、scope、discovered locations、可選 effective path、CLI source/state、provider diagnostic 及 MCP editable 狀態
- [x] 1.2 實作白名單 executable/argv 的唯讀 child-process runner，包含指定工作目錄、中立 global probe 目錄、輸出容量限制、ANSI 清理、錯誤截斷、逾時 kill 與機密欄位排除
- [x] 1.3 為 runner 加入單元測試，驗證成功、找不到 CLI、非零 exit、逾時終止、輸出截斷及 diagnostic 不洩漏敏感值

## 2. Provider-aware root 掃描

- [x] 2.1 將 `AppSettings.enabledProviders` 傳入 Skills、Commands、MCP 掃描，依設定順序去重；停用與未知 provider 不執行 root/CLI/config I/O
- [x] 2.2 建立 Skills root registry，涵蓋 Claude `.claude`、Codex `.codex/.agents`、OpenCode `.opencode/.claude/.agents`、Copilot `.github/.claude/.agents`、Antigravity `.gemini/.agents` 的 project/global roots
- [x] 2.3 建立 Commands root registry，涵蓋 Claude commands、Codex prompts、OpenCode singular/plural command、Copilot GitHub/legacy prompts、Gemini commands 及 global roots
- [x] 2.4 依 provider 支援 `.md`、`.prompt.md`、`.toml` 與 `SKILL.md` 格式；同一 provider 同名資源合併所有 locations，只有 CLI 證實時標示 effective path
- [x] 2.5 以 temporary directory 測試每個 provider 的 project/global root、`.agents/skills` 相容矩陣、停用 provider 零 I/O、格式差異與同名多來源

## 3. Provider CLI adapters

- [x] 3.1 實作 OpenCode `debug skill` 與 `debug config` JSON adapters，並以 fixture 測試正常、缺欄位及格式損壞
- [x] 3.2 實作 Copilot `mcp list --json` 與 Codex `mcp list --json` adapters，保留可取得的 effective source/enabled 資料並加入 fixture 測試
- [x] 3.3 建立固定能力矩陣，只註冊 OpenCode Skills/Commands 與 Copilot/Codex MCP；驗證停用或其他 provider 組合不啟動 CLI、不解析文字或無效 JSON
- [x] 3.4 驗證 project CWD 與 global 中立 CWD 查詢不混用，且單一 adapter 失敗不影響 root discovery、設定資料與其他 provider 結果

## 4. 後端掃描與 IPC 整合

- [x] 4.1 重構 Skills 掃描以合併 enabled provider roots、同步正本與 OpenCode CLI effective 結果，納入 CLI-only 項目且不把 root 存在翻譯成 loaded
- [x] 4.2 重構 Commands 掃描以合併 enabled provider roots、同步正本與 OpenCode resolved config；失敗時保留 root/sync 資料且不建立 per-item unknown 狀態
- [x] 4.3 MCP 只列出 enabledProviders 與既有 adapter 的交集，只為 Copilot/Codex 合併 CLI effective/configured 狀態，OpenCode/Claude 維持設定管理
- [x] 4.4 維持既有 Tauri commands 的薄包裝與 spawn_blocking 架構，更新 Rust/TypeScript 型別及 query keys，使 kind、scope、enabled provider 集合隔離
- [x] 4.5 新增後端整合測試，涵蓋 provider roots 聯集、停用 provider、OpenCode CLI-only/未列入、adapter 局部錯誤、MCP adapter 交集及同步狀態不被覆寫

## 5. Agents 資源狀態介面

- [x] 5.1 移除固定 `CHIP_PLATFORMS`、`chipStateFromStatus` 及 enabledTargets/sync status 推導 loaded/notInstalled 的邏輯，依後端 enabled provider order 渲染
- [x] 5.2 Skills/Commands 顯示各 provider 的 root visibility/effective path；只為 OpenCode 加上 CLI effective 狀態，同步摘要保持獨立
- [x] 5.3 MCP 只顯示啟用且有 adapter 的 provider；Copilot/Codex 顯示 CLI effective/configured，並停用 CLI-only 不可安全寫入來源的操作
- [x] 5.4 將 root scan/CLI query 改為頁籤首次進入、enabledProviders 變更、手動重新整理及成功變更後觸發，保留前次資料並禁止背景輪詢
- [x] 5.5 補齊 zh-TW、en-US 文案與前端元件測試，確認 provider 欄位隨設定變動、未支援組合無 CLI badge，且不再出現不準確的「已載入／未安裝」

## 6. 閱讀器可讀性

- [x] 6.1 使用既有主題 surface token，讓 Skills/Commands preview modal、header 與 ContentViewer 內容區完整不透明，不更動通用 dialog glass 規則
- [x] 6.2 以長篇 Markdown 在 light/dark 主題與窄視窗檢查底層文字不穿透、文字對比、橫向 code 捲動及 modal 內部垂直捲動

## 7. 驗證

- [x] 7.1 執行 `cargo test`，修正 resource registry/discovery、agents_config 與 mcp_config 測試失敗
- [x] 7.2 執行 `bun run lint` 與 `bun run build`，修正 TypeScript、React 與 CSS 品質問題
- [x] 7.3 逐組切換 enabledProviders，手動驗證五家 project/global root 掃描、OpenCode Skills/Commands、Copilot/Codex MCP 與停用 provider 零 I/O
- [x] 7.4 使用瀏覽器自動化完成 light/dark、desktop/mobile 閱讀器與動態 provider 狀態清單回歸，確認 console 無新增錯誤
