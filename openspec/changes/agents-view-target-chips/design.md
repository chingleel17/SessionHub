## Context

`add-mcp-config-management` 已實作 `AgentsConfigView`，其中 Skills / Commands 清單（`renderListGroup`）目前對每一列僅計算一個聚合徽章：統計 `prefs.enabledTargets` 中「非 in-sync/linked/canonical」的 target 數，顯示「已同步」或「需同步 N」。設計稿 `Agents.dc.html`（來源：claude.ai/design 專案 sessionhub）改為每列直接列出四個平台的個別狀態晶片。

後端 `SkillEntry.targets` / `CommandEntry.targets` 已提供 `TargetStatus[]`，其中每筆含 `targetId` 與 `SyncStatus`；`resolveTargetStatuses(tab, data, entry)` 已負責補上 skills 的虛擬 `agents` canonical 欄位。所需資料齊備，本變更為純呈現層調整。

設計稿中列的結構為：名稱（190px 等寬字型）＋描述（彈性、省略號）＋四個固定寬度晶片（每個 66px、含 5px 色點與平台名）。全域群組標頭另有一列 sync-note（說明文字＋三色圖例＋同步/重新整理按鈕）。

## Goals / Non-Goals

**Goals:**

- 每列以固定順序 claude / codex / opencode / copilot 呈現四個逐平台狀態晶片，狀態→顏色對應與設計稿一致。
- 全域清單標頭內嵌 sync-note 圖例與同步入口。
- 文案全走 i18n；配色沿用既有 `agents-status-pill` 色調精神（柔和、無粗黑膠囊）。

**Non-Goals:**

- 不改後端掃描、Tauri commands 或型別。
- 不改同步 modal 內的矩陣呈現（既有 `矩陣欄位對齊與精簡狀態呈現` 需求不動）。
- 不新增跨平台同步行為。

## Decisions

**D1 — 晶片狀態對應（三態）。** 新增 `chipStateFromStatus(status, enabled)` 純函式：`enabled === false` → `未安裝`；否則依 `SyncStatus`：`in-sync`/`linked`/`canonical` → `已載入`，`differs`/`target-missing`/`link-broken` → `需同步`，`source-missing`/`not-in-source` → `未安裝`。固定平台順序常數 `["claude","codex","opencode","copilot"]`。

**D2 — 資料來源。** 沿用既有 `resolveTargetStatuses(tab, data, entry)` 取得該列所有 target 狀態，再以四平台順序逐一查表；查無對應 target 者視為 `未安裝`。`enabled` 由 `prefs.enabledTargets.includes(targetId)` 決定。

**D3 — 取代聚合徽章。** 移除 `renderListGroup` 內原本計算 `outOfSyncCount` / `badgeLabel` 並渲染單一 `agents-status-pill` 的區塊，改渲染四個晶片容器。列的點擊仍為 `openPreview(entry)`（進入詳情頁），晶片本身不攔截點擊（避免與整列點擊衝突）。

**D4 — 全域 sync-note 標頭。** 目前 skills 已有 `agents-skills-compat-note`；擴充為含三色圖例 span 與同步/重新整理按鈕的版面（commands 頁沿用同結構或以既有 `agents-list-actions-row` 承載圖例）。圖例點顏色與晶片色點一致。同步按鈕沿用既有 `setSyncModalTab(tab)`，不新增邏輯。

**D5 — 樣式。** 於 `App.css` 新增 `.agents-target-chip`（inline-flex、色點、平台名、柔和底色/邊框、固定寬度）與三個修飾類 `--loaded` / `--needsSync` / `--notInstalled`，以及 sync-note 圖例 `.agents-sync-legend` 樣式。沿用既有色票（綠 `#16a34a`、藍 `#2563eb`、灰 `#d1d5db`）以與設計稿一致。

**D6 — i18n 鍵。** 新增 `agents.chip.loaded` / `agents.chip.needsSync` / `agents.chip.notInstalled`、`agents.chip.tooltip`（帶 `{platform}` `{state}` 參數）、`agents.legend.*` 與（若尚缺）`agents.skills.compatNote` 對應 commands 版說明；zh-TW 與 en-US 同步新增。

## Risks / Trade-offs

- **視覺密度提高**：每列四個晶片較單一徽章佔空間；以固定 66px 寬與省略號描述控制，窄視窗時晶片容器 `overflow:hidden` 截斷（沿用設計稿）。
- **狀態語意一致性**：`enabled === false` 一律顯示「未安裝」可能與底層實際已存在檔案的情況不符，但與設計稿固定四欄語意一致，且避免使用者對未啟用 target 誤解為待同步；此取捨於 spec 明列。
- 低風險：無檔案寫入、無後端變更，回歸僅需視覺驗證四種狀態組合與全域圖例。
