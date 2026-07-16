## 1. 後端：連結狀態判定重整

- [x] 1.1 `agents_config.rs`：`AgentsRootLinkStatus` enum 移除 `Conflict`，新增 `Partial` 與 `UnlinkedPhysical`（serde kebab-case：`partial` / `unlinked-physical`），`Partial` 附帶未對應項目名稱清單供前端顯示
- [x] 1.2 `check_agents_root_link_against`：`~/.agents` 為實體目錄時改走逐項等效判定——列舉正本第一層子項目，逐一以 `symlink_metadata` + `read_link` + `canonicalize_link_target` 檢查 `~/.agents` 同名項目是否解析到正本對應路徑；全部對應 → `Linked`，部分對應 → `Partial`（含缺漏清單），零對應 → `UnlinkedPhysical`
- [x] 1.3 `link_agents_root_to`：僅 `Missing` 狀態執行整層 symlink 建立；`Partial` / `UnlinkedPhysical` / `NotLinked` 回傳明確錯誤訊息，不覆蓋、不搬移
- [x] 1.4 更新與新增單元測試：整層 symlink → linked、逐項全對應 → linked、部分對應 → partial（驗證缺漏清單）、純實體 → unlinked-physical、實體副本（非 symlink）不計入等效、missing 建立連結成功、非 missing 呼叫建立連結被拒

## 2. 前端：型別與 banner 呈現

- [x] 2.1 `src/types/index.ts`：`AgentsRootLinkStatus` 改為 `"linked" | "partial" | "unlinked-physical" | "not-linked" | "missing"`，並依後端回傳結構補上 `partial` 缺漏清單欄位
- [x] 2.2 `AgentsConfigView.tsx`：banner 分支改寫——`partial` 顯示資訊性提示與未對應項目名稱；`unlinked-physical` 顯示資訊性提示（實體目錄、原生工具讀實體內容）；兩者皆不顯示「建立連結」按鈕；`missing` 維持按鈕；移除 conflict 分支
- [x] 2.3 `zh-TW.ts` / `en-US.ts`：移除 `agents.rootLink.conflict.*`，新增 `partial` 與 `unlinked-physical` 對應文案 keys（資訊性語氣）

## 3. 驗證

- [x] 3.1 `cargo test`（agents_config 相關測試）與 `tsc --noEmit` 全數通過
- [x] 3.2 實機走查四情境：整層 symlink、逐項 symlink（link.bat 佈局）、純實體目錄、`~/.agents` 不存在 → banner 呈現與「建立連結」可用性符合 spec
