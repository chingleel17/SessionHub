# Design: 修正 SessionHub 終端中 OpenCode 全域 skills 無法載入

## Context

### 現象

同一台機器、同一專案目錄（`D:\ching\SourceCode\tool\session_hub`）、同一個 `opencode.exe`（1.18.4，`C:\nvm4w\nodejs\node_modules\opencode-ai\bin\opencode.exe`）：

| 啟動來源 | skills 數 |
|---|---|
| SessionHub 開啟的 PowerShell 終端 | 7（僅專案內） |
| 使用者自行開啟的終端 | 21（專案 + 全域） |

SessionHub 的程序鏈：`session-hub.exe` → `pwsh.exe -NoExit -Command "cd '<project>'"` → `opencode.exe`。

此問題曾被修正過又復發，代表既有 spec 未涵蓋真正的失效條件。

### OpenCode 1.18.4 的 skill discovery 實作（由 exe 反組譯字串取得）

```js
EA = function*(j, J, K, z, B, X, M, T){
  let $ = {matches:new Set, dirs:new Set}, Z = [];
  if(!B){                                    // B = disableExternalSkills
    if(!X) Z.push(".claude");                // X = disableClaudeCodeSkills
    Z.push(".agents");
    for(let G of Z){                                    // ── global 掃描
      let Y = path.join(z.home, G);
      if(!(yield* K.isDir(Y))) continue;                //    靜默跳過
      yield* u($, Y, "skills/**/SKILL.md", {dot:!0, scope:"global"});
    }
    let O = yield* K.up({targets:Z, start:M, stop:T})   // ── project 往上找
              .pipe(A.catch(()=>A.succeed([])));        //    整段吞掉錯誤
    for(let G of O) yield* u($, G, "skills/**/SKILL.md", {dot:!0, scope:"project"});
  }
  ...
}

// scoped scan 失敗時靜默回傳空集合：
u = function*(j, J, K, z){
  let B = yield* A.tryPromise({try:()=>ZA.scan(K,{cwd:J, absolute:!0, include:"file", symlink:!0, dot:z?.dot}), catch:(X)=>X})
    .pipe(A.catch((X)=>{
      if(!z?.scope) return A.die(X);
      return A.logError(`failed to scan ${z.scope} skills`, {dir:J, error:X}).pipe(A.as([]))
    }));
  ...
}
```

`home` 的定義（同一份反組譯）：

```js
An = { get home(){ return process.env.OPENCODE_TEST_HOME ?? os.homedir() }, data:U, config:Tn, ... }
```

三個關鍵性質：

1. **global 與 project 掃描共用同一個 `Z` 陣列、同一個 `if(!B)` 區塊**，但兩者是各自獨立的迴圈，會分別失效
2. **scoped scan 失敗會被 `logError` 後回傳空集合**，不會中止流程 — 失效是靜默的
3. `isDir` 為偽時 `continue`，同樣靜默

### Log 證據（`C:\Users\User\.local\share\opencode\log\opencode.log`）

還原各 run 實際掃到的 skill 根目錄：

| run | count | 掃到的根目錄 |
|---|---|---|
| `e47b44d5`（SessionHub） | 7 | 專案 3 個（`session_hub`、`.claude`、`.opencode`） |
| `e6a54e80`（手動） | 21 | `~/.agents`、`~/.claude` + 專案 3 個 |
| `a24b8993` | 21 | 上述 + `~/.config/opencode` |
| `dcad77ba` | 15 | **僅** `~/.agents`、`~/.claude`（專案完全沒掃） |

`dcad77ba` 是關鍵反例：它證明 global 段與 project 段**會各自獨立失效**。若失效原因是 `disableExternalSkills` 旗標，`dcad77ba` 不可能出現 15。

### 已排除的假設

- **停用旗標**：SessionHub 終端中 `OPENCODE_DISABLE_EXTERNAL_SKILLS`、`OPENCODE_DISABLE_CLAUDE_CODE_SKILLS`、`OPENCODE_PURE` 皆不存在；`Get-ChildItem Env:OPENCODE*` 無輸出。且與 `dcad77ba`=15 矛盾
- **config / 路徑解析**：`opencode debug paths` 在 SessionHub 終端回傳正確的 home/config/data/cache；global config 與專案 config 皆確認有載入
- **`MSYS` 注入**：在正常環境單獨設定 `MSYS=error_start:` 後重測，仍為 21
- **環境變數差異**：以 SHA-256 指紋比對（不含明文值）SessionHub 終端與一般終端，差異僅為 `EFC_10588_*`、`GOROOT`、`MSYS`（SessionHub 側）與 `CLAUDE_CODE_*`、`WT_*`、`GIT_*` 等工具噪音（另一側）；**未出現** `OPENCODE_TEST_HOME`、`HOME`、`XDG_*`

### 未排除的主要假設：symlink/junction 遍歷

已確認的檔案系統狀態：

- `~/.agents` 與 `~/.claude` 本身是**實體目錄**（非 symlink）
- 其下 `skills\` 的**每一個子項目都是 `Directory, ReparsePoint`**，目標指向 `D:\ching\AI tool setting\agents\skills\<name>`（**跨磁碟機**，C: → D:）
- 兩個磁碟皆為 Fixed NTFS，連結目標可解析（`Test-Path` 為真）
- 這正是 SessionHub 自身 `agents_config.rs`（`link_agents_root_internal`、`sync_agents_items_internal`、`skill_target_roots`）所管理的結構，與既有 spec `agents-skills-sync` 一致

OpenCode 的 scan 帶 `symlink:!0`，必須逐項解析這些 reparse point。**若跨磁碟 reparse point 的解析在 SessionHub 拉起的程序中失敗（權限、impersonation、或工作目錄所在磁碟不同），`u()` 會靜默回傳空集合，症狀完全吻合，且能解釋「修過又復發」**。

### 診斷工具的已知陷阱（下次調查務必先處理）

1. **global config 目前有致命錯誤**，會讓 `opencode` 直接 exit 1、回傳 0 skills：
   ```
   Error: Configuration is invalid at C:\Users\User\.config\opencode\agents\planbyproject.md
   ↳ Expected object | undefined, got [...] tools
   ```
   該檔為 2026-03-30 舊檔（`tools:` 寫成陣列，1.18.4 要求 object 或 undefined）。**這不是本 bug 的根因**（兩邊終端都會一樣爆），但它會擋住所有量測，必須先修
2. **常駐 OpenCode 程序**：CLI 透過 client 與 server 溝通。若 `debug skill` attach 到已暖機的 server，改任何環境都不會反映在讀數上。調查期間曾同時存在 3 個常駐 `opencode` 程序，導致早期 flip-test 讀數失真（先是恆為 21，後突然全為 0）

### 約束

- 僅影響 Windows
- 不得硬編 skills 路徑、不得無條件注入 `HOME`、不得清除使用者環境變數
- 使用者工作樹有既有未提交變更，不得覆蓋或還原

## Goals / Non-Goals

### Goals

- 以可重現的實驗釘死根因，再動手修正
- 讓 SessionHub 終端中 OpenCode 掃描到的 skill **根目錄集合**與手動終端一致
- 建立回歸防護，避免同一問題第三次復發
- 統一三個啟動入口的環境傳遞實作

### Non-Goals

- 不修改 OpenCode 本身
- 不變更 `agents_config.rs` 既有的 symlink 佈署策略（`agents-skills-sync` 已定義）
- 不修正 `planbyproject.md` 的 config 錯誤作為本變更的一部分（它是獨立問題，僅需先排除以利量測）
- 不改變任何啟動指令、參數、工作目錄或 console creation flags

## Decisions

### D1：先完成根因定位，再實作修正

**決定**：本變更的 tasks 分為兩階段。階段一為診斷，必須產出可重現的最小差異；階段二才依據階段一結論實作。

**理由**：此問題已修過一次又復發，強烈暗示上次是對症狀而非根因下藥。已排除的假設（旗標、路徑解析、`MSYS`、環境變數）涵蓋了所有「顯而易見」的方向，剩下的假設需要實測。

**替代方案**：直接注入 `HOME` 或硬編 skills 路徑 — 已被使用者明確排除，且無證據支持，會再次製造假性修復。

### D2：以「掃描到的根目錄集合」而非「skill 總數」作為判準

**決定**：所有驗證以 log 中還原的 skill 根目錄集合比對。

**理由**：`dcad77ba`=15 與 `e47b44d5`=7 都是「壞」的，但總數不同；而 21 也可能來自不同的根目錄組合（`a24b8993` 含 `~/.config/opencode`，`e6a54e80` 不含，兩者皆為 21）。總數是有損指標。

### D3：診斷環境必須無殘留常駐程序且 config 可載入

**決定**：每次量測前確認無 `opencode` 常駐程序，並確保 global config 無致命錯誤；量測一律以冷啟動子程序進行。

**理由**：見上文「診斷工具的已知陷阱」。此決策直接來自本次調查中讀數失真的實際教訓。

### D4：修正手段優先序

依階段一結論，按以下優先序選擇修正方式（愈前面愈不侵入）：

1. **若為環境區塊傳遞問題** → 修正 `configure_msys_stackdump_suppression` 或啟動處的環境組裝，使其完整繼承而非部分覆寫
2. **若為 symlink/junction 遍歷權限問題** → 檢視 SessionHub 程序的權限脈絡（是否以不同 integrity level 或 token 執行），調整子程序啟動方式；必要時檢討 `agents_config.rs` 是否應改用 directory symlink 而非跨磁碟 junction
3. **若為工作目錄／磁碟脈絡問題** → 調整子程序的 cwd 設定方式

**理由**：不預設答案，但預先定義決策樹，避免階段二臨時發散。

### D5：以共用 helper 統一三個啟動入口

**決定**：將環境組態抽為單一 helper，`open_terminal_internal`、`open_in_tool_internal`、`resume_session_in_terminal_internal` 一律呼叫它。

**理由**：目前三者各自呼叫 `configure_msys_stackdump_suppression`，容易在新增入口時遺漏 — 這正是既有 spec `terminal-launcher` 已識別的風險。將其擴充為完整環境組態的單一入口，可讓「遺漏」在編譯期或測試期被發現。

**替代方案**：各入口各自處理 — 已被既有 spec 否決。

## Risks / Trade-offs

- **[階段一無法重現根因]** → 診斷步驟包含逐步縮小的環境重建（從 SessionHub 終端逐項還原至手動終端），並保留 log 根目錄集合作為客觀判準；若仍無法重現，將問題範圍與已排除清單記錄於本文件供後續接手
- **[修正只對當前 symlink 佈局有效]** → 驗證需涵蓋跨磁碟 junction 與同磁碟 symlink 兩種情境，避免修正僅在使用者當前設定下成立
- **[調整環境傳遞可能影響其他 provider]** → 三個入口共用同一 helper，任何變更需以 Rust 測試覆蓋全部 provider 分支；並確認不改變既有 MSYS 合併語意（`merge_msys_options` 已有測試）
- **[權限相關修正可能需要提權]** → SHALL NOT 要求使用者以系統管理員執行 SessionHub 作為修正手段；若根因確為權限，優先改變佈署結構而非要求提權（與 `agents-skills-sync` 既有立場一致）
- **[config 錯誤干擾驗收]** → 驗收前先確認 `opencode debug skill` 在手動終端能正常回傳，否則任何比對都無意義

## Migration Plan

1. 修正干擾量測的 global config 錯誤（`planbyproject.md`），或暫時移開以進行診斷
2. 清除殘留的 OpenCode 常駐程序
3. 執行階段一診斷，記錄結論於本文件
4. 依 D4 決策樹實作修正
5. 執行 `cd src-tauri && cargo test` 與 `bun run build`
6. 以真實 SessionHub 終端驗收：根目錄集合須與手動終端一致

**Rollback**：本變更僅涉及子程序環境組裝與（可能的）連結佈署方式，無資料庫 schema 變更、無設定檔格式變更。回滾即還原相關 commit。

## Open Questions

1. **SessionHub 程序本身是如何被啟動的？**（開機自動啟動 / 手動點擊捷徑）未追蹤的 `openspec/changes/add-launch-on-startup/` 顯示此專案正在導入開機自動啟動。若 SessionHub 由 Task Scheduler 或 registry Run key 拉起，會繼承精簡的環境區塊與不同的權限脈絡 — 這能同時解釋「環境指紋看似無差異」（因為指紋是在 SessionHub 終端內取的，已是繼承後的結果）與「修過又復發」。**這是最高優先待答問題**
2. `os.homedir()` 在 SessionHub 拉起的程序中實際回傳什麼？（`opencode debug paths` 正確，但需確認 discovery 用的 `home` 與 `debug paths` 顯示的是否同源）
3. 跨磁碟 junction（C: → D:）在不同權限脈絡下的解析行為為何？是否為 `dcad77ba`=15（只有 global、沒有 project）與 `e47b44d5`=7（只有 project、沒有 global）兩種對稱失效的共同解釋？
4. `dcad77ba`=15 這筆 run 是從哪個終端執行的？釐清它能大幅縮小假設空間
