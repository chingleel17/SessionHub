## Context

`App.tsx` 的即時事件訂閱集中在一個 `useEffect`（依賴陣列 `[activePlanSession, activeProject, queryClient, refreshProjectPlansSpecs, settingsQuery.data, t]`），內含：

1. `copilot-sessions-updated` / `opencode-sessions-updated` / `codex-sessions-updated` / `claude-sessions-updated` — 4 個幾乎相同的 `onSessionsRefresh` handler（已共用同一函式，無重複問題）
2. `copilot-session-targeted` / `claude-session-targeted` — 2 個邏輯完全相同的 handler（**重複**，僅 `cwd` 查詢與 `setQueriesData` 更新，事件名不同）
3. `copilot-activity-hint` / `claude-activity-hint` / `opencode-activity-hint` — 3 個高度相似的 handler，都做「找 session → 算 status/detail → `setQueriesData` 更新 `activity_statuses`」，但欄位來源與 fallback 規則不完全相同（claude 版本有 `hintStatus` 優先於前端推算的邏輯，copilot 版本沒有；opencode 版本更簡化）
4. `plan-file-changed`、`project-files-changed`、`quota-snapshots-updated`、`navigate-main-view` — 各自獨立、無重複

另外 `App.tsx:508-513` 還有一個獨立的 `provider-bridge-event-logged` 監聽（bridge event log 緩衝），與上述主要事件訂閱 `useEffect` 是分開的兩塊，本次不動它（範圍已經足夠大，避免混雜不相關重構）。

**與既有 spec 的關係**：`openspec/specs/mcp-config-management/spec.md:206` 規定「前端 IPC 呼叫 SHALL 集中於 `App.tsx`」，該條文的上下文是規範 `McpConfigView` 這類**展示元件**不可自帶 IPC、避免呼叫下放到元件樹葉節點。本 change 抽出的 `useSessionRealtimeEvents()` 是 custom hook 而非展示元件，僅被 `App()` 呼叫、不被其他元件引用，屬於「App 層」而非「元件層」的程式碼組織方式調整。經與使用者確認，此舉視為符合該條文精神（IPC 呼叫仍統一由 App 層發起與管理，只是拆到同層級的 hook 檔案），不構成對該 spec 的行為變更，故不需要 delta。若未來需要更嚴格地遵守字面規定，可重新評估是否需要修改 spec 文字或改回不拆檔案。

## Goals / Non-Goals

**Goals:**
- 消除 `copilot-session-targeted` / `claude-session-targeted` 的重複程式碼
- 消除三個 `*-activity-hint` handler 中重複的 `setQueriesData` 更新 pattern，同時保留各 provider 既有的欄位語意差異（不可為了去重而抹平 claude 版本 `hintStatus` 優先邏輯等既有行為）
- 把整個事件訂閱邏輯從 `App.tsx` 搬到獨立 hook 檔案，降低 `App.tsx` 行數與職責
- 保持事件監聽時機、cleanup 時機、`mounted` guard 行為完全不變

**Non-Goals:**
- 不改變後端事件發送邏輯或事件 payload 結構
- 不處理 `provider-bridge-event-logged` 監聽（獨立區塊，留待後續視需要再處理）
- 不處理 embedded app（`EmbeddedQuotaOverlayApp` / `EmbeddedTrayPanelApp`）的抽離，那是 `extract-embedded-apps-and-settings-hook` change 的範圍
- 不新增任何新的即時事件類型

## Decisions

### D1：activity-hint 統一為單一「更新函式」+ 各事件各自算 status/detail 後呼叫它
不強行把三個 handler 合併成同一個 `listen()` 訂閱（事件名稱不同、部分欄位語意不同），而是抽出：
```ts
function applyActivityStatusUpdate(
  queryClient: QueryClient,
  sessionId: string,
  update: { status: SessionActivityStatus["status"]; detail?: SessionActivityStatus["detail"]; lastActivityAt?: string | null },
): void
```
三個 handler 各自保留自己的「如何從 event.payload 算出 status/detail」邏輯（因為這部分本來就不同），但都改呼叫同一個 `applyActivityStatusUpdate` 做 `setQueriesData`。

替代方案（放棄）：把三個 provider 的 hint 邏輯完全合併成一個泛用 handler，用 provider 參數 switch。放棄原因：copilot／claude／opencode 三者的 payload 欄位與 fallback 規則本來就不同（claude 有 `hintStatus` 優先、copilot 全靠前端推算），硬合併會讓單一函式充滿 `if (provider === ...)` 分支，可讀性反而更差；D1 只去重「真正重複」的更新邏輯。

### D2：session-targeted 合併為單一參數化函式
`copilot-session-targeted` 與 `claude-session-targeted` 邏輯完全相同，抽出：
```ts
function createSessionTargetedHandler(
  queryClient: QueryClient,
  copilotRoot: string | undefined,
  onSynced: () => void,
): (event: Event<SessionTargetedPayload>) => Promise<void>
```
兩個 `listen()` 呼叫各自傳入事件名稱，但共用同一個 handler 建構函式。

### D3：整體搬到 `src/hooks/useSessionRealtimeEvents.ts`
新增 custom hook，簽章大致為：
```ts
function useSessionRealtimeEvents(params: {
  activePlanSession: SessionInfo | null;
  activeProject: ProjectGroup | null;
  copilotRoot: string | undefined;
  refreshProjectPlansSpecs: (dir: string) => Promise<void>;
  onRealtimeStatusChange: (status: "active") => void;
  showToast: (msg: string) => void;
  setActiveView: (view: string) => void;
}): void
```
hook 內部持有 `sessionsDataRef` 的等效機制（可由呼叫端傳入 ref，或 hook 自行透過 queryClient 讀取最新 sessions cache，實作時二擇一並在 design 落地時記錄理由）。`App.tsx` 呼叫此 hook 取代原本整個 `useEffect` 區塊。

替代方案（放棄）：拆成 4-5 個更細的 hook（每類事件一個 hook）。放棄原因：這些事件共享同一組 cleanup 生命週期與 `mounted` guard，拆太細會需要重複建立多組 `useEffect` 生命週期樣板，維護成本不降反升；單一 hook 內用具名內部函式分段已足夠可讀。

## Risks / Trade-offs

- **[風險] 抽出 hook 過程中不慎改變某個 provider 的 status/detail 計算規則** → 緩解：tasks.md 要求逐一比對重構前後三個 activity-hint handler 的計算邏輯（非只看最終效果）
- **[風險] `sessionsDataRef` 的 stale-closure 保護機制（原本用 ref 而非 state）若在搬移時被誤改為直接依賴 `sessionsQuery.data`，會重新引入 stale closure bug** → 緩解：design D3 明確保留 ref 機制，tasks.md 要求驗證此點
- **[風險] hook 依賴陣列與原本 `useEffect` 不一致，導致 cleanup 時機改變（例如少了某個 dep 造成 stale closure，或多了某個 dep 造成事件重複訂閱/取消訂閱過於頻繁）** → 緩解：依賴陣列原樣搬移，tasks.md 要求人工核對

## Migration Plan

無資料遷移。純前端程式碼重構，一般 PR 流程套用。可透過還原單一 commit rollback。
