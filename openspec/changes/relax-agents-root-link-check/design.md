## Context

全域範圍設定自訂 agents 正本位置（`agentsSourceRoot` ≠ `~/.agents`）時，`agents_config.rs` 的 `check_agents_root_link_against` 以「`~/.agents` 整層是否為指向正本的 symlink」為唯一判準：

- `~/.agents` 不存在 → `missing`（banner 提供「建立連結」）
- `~/.agents` 是實體目錄 → 一律 `conflict`（banner 顯示「已存在實體內容，無法自動連結」，要求手動合併）
- 是 symlink 但指向他處 → `not-linked`
- 是指向正本的 symlink → `linked`

實務上使用者常以逐項 symlink 方式維護 `~/.agents`（外層是實體目錄，內部 `instructions`、`agents`、`skills` 等子項目個別連到正本），或直接放實體檔案。這些佈局功能上完全等效或屬使用者自主選擇，現行檢查卻回報阻擋性 conflict，且「先手動合併再整層連結」的指引會破壞既有佈局。

## Goals / Non-Goals

**Goals:**

- `~/.agents` 為實體目錄時不再一律回報 conflict，改依內容做等效性判定
- 逐項 symlink 佈局（子項目均解析到正本對應路徑）視為已連結
- 實體內容與正本無關聯時給資訊性提示，不用錯誤語氣、不提供破壞性自動化
- 「建立連結」整層 symlink 自動化僅保留給 `missing` 情境

**Non-Goals:**

- 不提供「自動逐項建立子項目 symlink」的新自動化（維持 YAGNI，先只調整判定與呈現）
- 不改變 Skills/Commands 同步矩陣的掃描與同步邏輯
- 不改變 `agentsSourceRoot` 設定本身的解析行為
- 不處理專案範圍（`<project>/.agents`）

## Decisions

### D1: 狀態機重整 — `conflict` 拆分為 `partial` 與 `unlinked-physical`

新狀態集合：`linked` | `partial` | `unlinked-physical` | `not-linked` | `missing`。

- `linked`：`~/.agents` 整層是指向正本的 symlink，**或**為實體目錄且正本第一層每個子項目在 `~/.agents` 都有同名項目、且該項目為解析到正本對應路徑的 symlink（逐項等效）
- `partial`：實體目錄，正本第一層子項目部分有對應 symlink、部分缺漏或指向他處
- `unlinked-physical`：實體目錄，與正本無任何 symlink 對應（純實體內容，可能是使用者自主維護）
- `not-linked`：整層是 symlink 但指向他處（維持原語意）
- `missing`：`~/.agents` 不存在（維持原語意）

捨棄方案：沿用 `conflict` 但僅改文案——狀態名稱與實際語意不符（實體目錄不是「衝突」），且無法區分逐項連結與純實體兩種需要不同提示的情境。

等效性判定以「正本第一層子項目」為基準而非遞迴全樹：遞迴比對成本高且逐項 symlink 佈局只會發生在第一層；正本子項目在 `~/.agents` 對應到實體副本（非 symlink）時不計入等效（無法保證同步），該項視為缺漏。symlink 解析用 `fs::read_link` + 既有 `canonicalize_link_target` 工具，與現行 `linked` 判定一致。

### D2: banner 呈現 — 資訊性提示取代阻擋性錯誤

- `linked`（含逐項等效）：沿用既有已連結徽章
- `partial`：資訊性提示「部分子項目已連結至正本」，列出缺漏項目名稱，不提供自動修復按鈕（使用者自行決定如何補）
- `unlinked-physical`：資訊性提示「~/.agents 為實體目錄，未連結至自訂正本」，說明三個原生相容工具（codex/opencode/copilot）將讀取實體內容而非正本；不再出現「無法自動連結、請先手動合併」錯誤文案
- `missing`：維持「建立連結」按鈕（整層 symlink）
- `not-linked`：維持現行提示（指向他處屬異常，仍需使用者處理）

### D3: `link_agents_root` 防護維持

`link_agents_root_to` 僅在狀態為 `missing` 時執行整層 symlink 建立；`partial` / `unlinked-physical` / `not-linked` 呼叫時回傳明確錯誤訊息（前端不顯示按鈕，此為第二層防護）。與現行「不覆蓋、不搬移」原則一致。

## Risks / Trade-offs

- [逐項等效判定僅看第一層，深層差異偵測不到] → 屬可接受簡化；矩陣同步掃描本身仍會逐 skill 比對內容，等效判定只影響 banner 呈現
- [正本第一層項目很多時逐項 read_link 成本] → 第一層項目數量級為個位數到十位數，成本可忽略
- [舊前端／新後端狀態字串不相容（新增 `partial` 等值）] → 前後端同版發佈（Tauri 桌面 app 無分離部署），無相容性窗口
- [使用者原以 conflict 文案理解現況，升級後語意改變] → 新文案更準確描述實際狀態，屬修正而非破壞
