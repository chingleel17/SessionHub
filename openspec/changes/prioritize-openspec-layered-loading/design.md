# Design

## Context

見 `proposal.md`。現有 OpenSpec scanner 回傳完整 `OpenSpecData`，專案檢視以單一 query 等待整個目錄掃描；外部檔案 watcher 與 tasks 樂觀寫入依賴專案路徑的 query key。UI 仍須支援沒有 OpenSpec 目錄與舊版不帶參數的 IPC 呼叫。

## Goals / Non-Goals

**Goals:** 優先顯示 active 資料、維持完整掃描契約、區段結果不跨專案、讀取失敗保留成功區段，以及重整時保留使用者操作狀態。

**Non-Goals:** 不改變 OpenSpec 磁碟格式、不建立持久化索引、不調整其他資源掃描或全站路由。

## Decisions

1. **保留完整回傳型別，增添可選區段。** `OpenSpecScanSection` 依 active/specs/archived/all 控制 scanner 只走需要的目錄；舊呼叫走 all。命令以 blocking pool 執行同步掃描，避免阻塞非同步 IPC。相較新增三個專用命令，沿用單一相容的 API 與既有 metadata 解析。
2. **依序啟用三個 project-scoped query。** key 為 `["project_specs", projectDir, section]`，只在前一層成功且不在重新擷取時啟用下一層；各層獨立 stale/cache 狀態，合併成既有 `OpenSpecData` 傳入純展示元件。前一層失敗時停止後續階段並顯示錯誤；先前成功的快取資料仍可顯示。相較並行掃描，這確保 active 優先取得 I/O；代價是完整結果稍晚。
3. **重整先標記所有層過期，再重掃 active。** 相依 query 逐層重新擷取；tasks 樂觀更新只作用於 tasks 所在的 active 或 archived key，寫入失敗回滾。watcher 的短暫抑制避免剛寫入的進度被競爭中的掃描覆蓋。
4. **Explorer 僅初始化新來源及群組狀態。** 後續層加入時不重設已選文件與收折；重掃短暫找不到文件時等候後續層。以 project path 作元件 key 隔離不同專案的畫面狀態；保留每專案既有檢視模式偏好。背景狀態與失敗訊息用既有 i18n 與設計 token 顯示。

## Risks / Trade-offs

- [三次 IPC 增加完整顯示時間] → active 優先且各層可操作，藉大型 fixture 比對分層合併與完整掃描結果。
- [重整後舊區段與新 active 短暫共存] → 先標記各層過期、依相依查詢重掃；暫時缺席不立即清除文件選取。
- [區段失敗造成後續查詢未啟動] → 顯示錯誤與重整途徑，不把未啟動層當作持續載入。

## Migration Plan

僅新增可選 IPC 參數，不需資料遷移；回退前端至原本完整掃描呼叫仍由後端相容分支支援，整體回退則一併回退前後端修改。此 change 保持未封存供驗證與後續依專案流程處理。
