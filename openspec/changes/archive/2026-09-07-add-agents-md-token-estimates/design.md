## Context

`scan_agents_md` 目前在同一次走訪中為 AGENTS.md 與 CLAUDE.md 建立 SHA-256、mtime 與 byte size fingerprint，但不保留文字內容指標。前端將掃描資料轉成通用 `TreeNode`，`ExplorerTree` 目前只有 label 與狀態 badge；`AgentsConfigView` 的編輯按鈕則位於內容面板內的獨立 action row。

OpenAI 與 Anthropic 均提供官方遠端 token counting API，可回傳特定模型實際接受 request payload 時的精確輸入 token 數；但兩者都需要 API 認證與網路呼叫。OpenAI 官方另提供 Tokenizer 網頁工具，並建議純文字程式化計數使用 `tiktoken`；其公開英文粗估為「1 token 約 4 字元」或「1 token 約 0.75 個英文單字」。同一份官方說明也指出 token 數會因模型、編碼與語言不同，且純文字計數不含訊息結構、工具及檔案等 request framing。Anthropic 未提供可保證對應目前 Claude 模型的公開本機 tokenizer。因此第一版不能合理地宣稱支援 Claude Opus 或 GPT-5.6 的精確本機計數。

參考：
- OpenAI Tokenizer：`https://platform.openai.com/tokenizer`
- OpenAI「瞭解並計算 Token」：`https://help.openai.com/zh-hant/articles/4936856-understanding-and-counting-tokens`
- OpenAI tiktoken：`https://github.com/openai/tiktoken`
- OpenAI Counting tokens：`https://developers.openai.com/api/docs/guides/token-counting`
- Anthropic Count Message tokens：`https://platform.claude.com/docs/en/api/messages/count_tokens`

## Goals / Non-Goals

**Goals:**
- 以不含機密外傳風險、無網路延遲的方式，在掃描階段一次產生可比較的指示檔規模指標。
- 將估算算法隔離並版本化，未來可用校正後 profile 或精確 tokenizer 替換。
- 讓通用樹元件支援靠右 metadata，但不影響其他既有樹狀畫面。
- 減少內容面板頂部多餘的操作列，維持窄視窗與深色主題可讀性。

**Non-Goals:**
- 不提供帳務、context window 保證或供應商精確 token 數。
- 不在第一版呼叫 OpenAI／Anthropic token counting API，也不管理其 API key。
- 不在第一版估算 system prompt、tool schemas、skills 自動載入內容或訊息 framing 等額外脈絡。
- 不將指標擴充到 MCP、OpenSpec 或其他非 Agents 設定頁面的檔案。

## Decisions

### 1. 後端使用版本化的 mixed-text 本機估算器

新增獨立的純函式模組，輸入 `&str`，輸出 `character_count`、`estimated_tokens` 與固定 estimator 版本識別。第一版 `mixed-text-v1` 將文字分為 CJK 類字元與其餘非空白片段：CJK 類字元以每字約一 token 計，其他連續片段依約四個 Unicode scalar values 一 token 向上取整。後者直接採用 OpenAI 公開的英文粗估基準；前者是因應非英文 token 密度通常較高的保守啟發式，並非 OpenAI 或 Anthropic 公布公式。算法只使用整數運算，空內容回傳零，結果具確定性。

選擇此方案是因為指示檔常混合繁體中文、英文、Markdown 與程式碼，單純 `bytes / 4` 會明顯低估 CJK，而將所有字元視為一 token 又會高估英文。OpenAI Tokenizer 可用來建立代表性測試樣本與觀察誤差，但不作為應用程式執行階段服務。替代方案包括直接整合 `tiktoken` 或相容套件；這可改善特定 OpenAI encoding 的純文字計數，卻無法代表 Claude、未必能對應尚未公開的模型 encoding，且增加原生套件與模型映射維護。遠端 counting API 最精確，但需要金鑰、網路並會將指示內容送出，不適合預設清單掃描。

實作時以固定的中英混合 AGENTS.md corpus 建立 regression fixtures，至少涵蓋英文段落、繁體中文、Markdown 標題／清單、程式碼與路徑。英文及混合樣本先以 OpenAI Tokenizer 取得參考結果並記錄當時選用的 encoding／模型；fixture 用於量測與記錄誤差，不設定成跨供應商的精確相等斷言。Claude 僅能在有獨立測試憑證的開發環境透過 Count Message tokens API 做非必要比較，不得成為 build 或測試必要條件。

### 2. 指標附加在各清單實際預覽的文字檔

新增 `FileContentMetrics`，由 `AgentsMdEntry.sourceMetrics` 與 `targetMetrics` 個別選擇性攜帶，`SkillEntry.metrics` 對應該列實際預覽的 `SKILL.md`，`CommandEntry.metrics` 對應該列實際預覽的 `.md`、`.prompt.md` 或 `.toml`。不修改共用 `FileFingerprint`，因為 fingerprint 同時用於目錄同步與非預覽用途，直接加入內容欄位會迫使大量非文字路徑與建構點承擔無意義資料。

掃描既有 fingerprint 後，只對存在且實際可預覽的指示檔讀取 UTF-8 文字並計算指標。讀取或解碼失敗回傳 `None`，不改變既有 `exists`、hash、discovery 或同步狀態。這會為每個指示檔多一次循序讀取；檔案通常很小，先保持實作簡單，若量測發現瓶頸再將 hashing 與文字統計合併為單次串流。

### 3. TreeNode 新增通用 trailing metadata

在 `TreeNode` 增加可選的 `trailingMeta` 顯示字串，`ExplorerTree` 於 badge 後渲染為不縮小的次要文字並推至列尾。`buildAgentsMdTree` 依實際 `filePath` 配對 source 或 target metrics；內容不同而拆成兩個 child 時，各 child 使用自己的估算值。其他 tree builder 不設定此欄位，因此 UI 保持不變。

token 精簡格式由前端純函式統一處理：小於 1,000 顯示整數；其餘除以 1,000、四捨五入至一位小數並移除 `.0`。完整字元數使用目前 locale 的 `Intl.NumberFormat`。樹列可用 title 或可及性標籤提供完整字元與未縮寫 token 數。

### 4. 檔案 metrics 與操作共用內容標頭／頂層 action slot

`ContentViewer` 增加可選 header metadata slot 或字串 prop，使檔案路徑與內容指標維持同一列；未傳入時不影響其他使用位置。`AgentsConfigView` 移除 `.agents-content-actions` 占用的獨立列，將編輯／預覽與儲存按鈕加入現有 `mdActions`，並以已選取檔案為條件渲染。外部開啟、檔案總管與同步按鈕沿用既有 handler；未選取時可保留 refresh，但檔案專屬操作不渲染。

儲存成功後沿用上層 refresh handler 更新掃描資料，避免在前端另寫一份估算算法而造成數字分歧。視覺使用現有 icon、button 與色彩 token，不新增卡片或高對比邊框。

### 5. Skills 與 Commands 沿用同一 formatter 與預覽 header

Skills／Commands 的 provider-aware 掃描可能將同名資源合併成一列；該列的 metrics 必須與點擊後實際開啟的 preview path 來自同一個 entry，不加總不同 provider 的重複內容。清單在既有 provider chips 後方渲染不可縮小的 token 文字，使其成為最右側資訊；檔名與描述優先截斷。預覽 modal 將同一份 metrics 傳入 `ContentViewer.headerMeta`，因此清單與內容標頭不會採用不同估算來源。

## Risks / Trade-offs

- [估算值可能與任一實際模型差距明顯] → 所有顯示固定加 `~`，tooltip／說明使用「估算」語意，且不以模型名稱宣稱精確度。
- [OpenAI Tokenizer 的模型／encoding 選項日後改變] → calibration fixture 記錄取得日期與選項，只作誤差觀察；產品算法的確定性測試不依賴網站即時結果。
- [CJK 與 emoji 的 Unicode scalar count 不等同使用者感知 grapheme] → 規格明確定義為 Unicode 字元統計；測試涵蓋 surrogate 之外的 Rust `char` 行為，避免誤用 UTF-8 byte size。
- [掃描多一次檔案增加 I/O] → 僅處理已找到的兩種指示檔，錯誤時降級而不阻斷；保留未來合併串流讀取的空間。
- [共用 ExplorerTree 排版回歸] → `trailingMeta` 完全可選，加入窄寬、長檔名與既有 badge 的元件測試／視覺驗證。
- [儲存瞬間顯示舊 metrics] → 儲存完成後觸發既有掃描 refresh；在新結果回來前不於前端猜測數字。

## Migration Plan

此變更只為 Tauri 掃描回應增加欄位，不涉及持久化資料或設定 migration。前後端於同一桌面版本一起發佈；若需回復，可移除選擇性 metrics 欄位與 UI metadata，既有 fingerprint、同步與檔案讀寫流程不受影響。
