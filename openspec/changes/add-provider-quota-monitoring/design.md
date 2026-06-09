## Context

SessionHub 目前已經有三種與「即時狀態」相關的能力：session 掃描、provider bridge、analytics 聚合。但這些資料都還停留在 session 與 token 使用結果層，沒有一個統一的 quota subsystem 來回答「某個 provider 還剩多少額度、何時 reset、資料來源是否可信」。

你提到想參考 `opencode-quota` 的方式。從產品方向看，它的價值不在單一 UI，而在於把 provider-specific 的 quota 來源封裝成統一輸出，再決定要顯示在 sidebar、toast 或 status line。對 SessionHub 而言，我們也需要同樣的分層，只是 UI 面會落在 Dashboard、Settings 與 global status bar，而不是 OpenCode TUI。

這個需求還有一個額外維度：你希望既能「內建」使用，也保留像插件那樣擴充的方式。這代表設計上不能把 quota 讀取邏輯直接塞進單一 command，而要先定義 adapter / connector 介面，再由 SessionHub 內建實作幾個 provider，未來才有空間以外掛方式擴充。

## Goals / Non-Goals

**Goals:**

- 在 SessionHub 內提供統一的 provider quota snapshot，至少能表達 provider、狀態、來源、使用量、剩餘量與 reset 時間。
- 以內建 quota adapters 為主路徑，讓常見 provider 可直接被 SessionHub 查詢。
- 保留插件式 connector 擴充點，允許未來新增 provider-specific quota 來源，而不必重寫核心 UI。
- 提供背景 refresh、手動 refresh，以及 bridge 事件觸發的節流更新能力。
- 在 Dashboard、Settings 與 status bar 顯示 quota 摘要與診斷資訊。

**Non-Goals:**

- 本次不承諾一次支援所有 `opencode-quota` 已知 provider；首版可只支援少數高價值 provider。
- 本次不直接嵌入或依賴 `opencode-quota` 的 OpenCode plugin runtime。
- 本次不要求 quota 資料一定由 hook 直接提供；hook/bridge 只作為 refresh trigger，不作為唯一資料來源。
- 本次不實作複雜的多租戶雲端同步或伺服器端配額管理。

## Decisions

### 1. 採用「quota manager + provider adapter」的內建架構

SessionHub 後端新增 quota manager，統一調度多個 provider adapter。每個 adapter 回傳標準化 quota snapshot，而不是讓前端直接理解不同平台的原始 payload。

原因：這與現有 session/provider 的模組化方向一致，也能把 provider-specific 的 auth、API、fallback 估算收斂在後端。

替代方案：

- 前端直接各自呼叫不同 quota API。缺點是 auth 與錯誤處理會分散在 `App.tsx`。
- 直接依賴 `opencode-quota` 套件做全部資料來源。缺點是會被 OpenCode plugin lifecycle 與其內部模組邊界綁住。

### 2. 內建 adapter 為主，插件式 connector 為延伸模式

首版以內建 adapters 為主，並在 quota manager 中預留 connector registry。connector 可以是 SessionHub 內建 adapter，也可以是日後透過設定啟用的外部 connector。

原因：這能先滿足「SessionHub 裡直接看到 quota」的需求，同時不把未來擴充路封死。

替代方案：

- 一開始就完全插件化。缺點是交付太重，且沒有內建 provider 時產品不可用。
- 完全不考慮插件化。缺點是之後每新增 provider 都要改核心程式。

### 3. quota source 與 session platform 分開建模

SessionHub 的 session platform（copilot / opencode / codex）與 quota provider（openai / github-copilot / anthropic ...）不一定一一對應，因此 quota snapshot 應以 quota provider 為主鍵，而不是直接綁定 session platform。

原因：例如 OpenCode 可能對接多個模型來源，Codex 很可能對應到 OpenAI quota，而不是獨立一個「codex quota」。

替代方案：

- 把 quota 直接綁在目前的 provider 欄位。缺點是模型與平台語意會混淆。

### 4. 更新策略採「背景輪詢為主，bridge trigger 為輔」

quota refresh 主要由 app startup、固定輪詢、手動 refresh 驅動；收到 provider bridge 事件時，只作為觸發 quota refresh 的節流信號。

原因：quota 通常需要查本地 auth 或遠端 API，bridge event 無法可靠攜帶完整額度資訊；但它很適合提示「剛剛有 activity，值得 refresh」。

替代方案：

- 完全只靠背景輪詢。缺點是互動後資料更新感較弱。
- 完全只靠 hook。缺點是資料來源不足且容易漏更新。

### 5. quota snapshot 需要本地快取與最後刷新時間

後端應維持一份 quota snapshot cache，包含 `fetchedAt`、source、error/message 與可能的 resetAt。前端以這份快取渲染，避免每次進入畫面都直接打 quota source。

原因：部分 provider 查詢可能慢、需網路、甚至有速率限制；快取可以改善 UX 與穩定性。

替代方案：

- 每次畫面 render 都同步即時查詢。缺點是效能與失敗體驗都差。

## Risks / Trade-offs

- [不同 provider 的 quota 來源差異很大] -> 以標準 snapshot 模型封裝差異，未支援 provider 回傳 `unsupported` 或 `unknown`。
- [遠端 quota API 可能需要敏感 auth] -> 優先使用既有本地 auth / provider integration 狀態，避免在 UI 中暴露機密值。
- [背景輪詢過於頻繁] -> 提供設定化 refresh interval 與 app-side cache，並在 bridge trigger 加 debounce。
- [插件式 connector 範圍過大] -> 首版只定義擴充邊界，不要求完整外部 SDK 生態。
- [Dashboard 與 status bar 容易資訊過載] -> Dashboard 顯示詳細 overview，status bar 僅顯示精簡摘要。

## Migration Plan

1. 新增 quota manager、snapshot 型別與查詢 command。
2. 先接入少量內建 adapters 與本地快取。
3. 在 Settings 顯示 quota diagnostics 與 refresh 控制。
4. 在 Dashboard 與 global status bar 接入 quota snapshot 顯示。
5. 補 bridge-triggered refresh 與設定化 refresh interval。

## Open Questions

- 首版要優先支援哪些 quota providers：OpenAI、GitHub Copilot，還是也包含 OpenCode 相關來源？
- quota snapshot 快取要只放記憶體，還是同步落地到 SQLite / settings cache？
- 外部 connector 的載入型態要走本機命令、JSON bridge，還是直接沿用現有 provider integration 概念？
