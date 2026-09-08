## 1. Token 估算核心與資料契約

- [x] 1.1 新增以 OpenAI 公開「英文約 4 字元 = 1 token」為基準的 `mixed-text-v1` 本機估算純函式與內容指標型別，並以 Rust 單元測試驗證空字串、英文、繁體中文、中英 Markdown、emoji 與重複計算的確定性。
- [x] 1.2 將 source／target metrics 接入專案與全域 AGENTS.md 掃描回應，確保 UTF-8 讀取失敗只回傳 unavailable；以 `cargo test` 驗證同步狀態不受影響且不同內容保留各自指標。
- [x] 1.3 同步更新 TypeScript IPC 型別，執行 `bun run build` 驗證 Rust camelCase 欄位在前端使用時通過 strict type check。

## 2. 樹列與格式化顯示

- [x] 2.1 新增 locale-aware 字元格式與 `~850 tokens`／`~1.2k tokens` 精簡格式化 helper，並以可執行的單元測試或明確 fixture 驗證 0、999、1,000、1,240 與整千邊界。
- [x] 2.2 擴充 `TreeNode` 與 `ExplorerTree` 的可選 trailing metadata，讓長檔名先截斷、尾端指標不縮小，並確認未設定 metadata 的既有 OpenSpec／Sisyphus tree DOM 與版面維持不變。
- [x] 2.3 在 `buildAgentsMdTree` 依實際 source／target file path 綁定 metrics，驗證一致單列、僅有單檔、diff 展開雙檔與 unavailable 四種 fixture 的顯示結果。

## 3. AGENTS.md 內容標頭與操作區

- [x] 3.1 讓 `ContentViewer` 支援可選 header metadata，並在選取指示檔時顯示完整字元數與 token 估算；以 `bun run build` 驗證其他呼叫端不需提供新 prop。
- [x] 3.2 將編輯／預覽與儲存移至 AGENTS.md 右上 action 區並移除獨立 action row，驗證未選檔時隱藏、選檔後可編輯、預覽與安全儲存，且 refresh／外部開啟／檔案總管／同步仍正常。
- [x] 3.3 依 i18n 與 Minimal UI token 補齊文案和樣式，手動驗證 light／dark、專案／全域 scope、sidebar 展開／收合及窄視窗皆無水平捲動且 token 位於檔案列最右側。

## 4. 整合驗證

- [x] 4.1 執行 `cargo fmt --check` 與 `cargo test`，修正所有 Rust 格式或測試失敗。
- [x] 4.2 執行 `bun run lint` 與 `bun run build`，確認前端 lint、TypeScript 與 production build 全數通過。
- [x] 4.3 以含繁體中文、英文、Markdown 與程式碼的實際 AGENTS.md 完成桌面 smoke test，確認掃描、選取、估算顯示、編輯儲存與儲存後指標更新符合 specs。
- [x] 4.4 使用 OpenAI Tokenizer 對固定代表性 corpus 建立帶日期及模型／encoding 註記的 calibration fixture，記錄 `mixed-text-v1` 對英文、繁體中文、Markdown 與程式碼樣本的誤差，但不將跨模型精確相等設為測試門檻。

## 5. Skills 與 Commands 擴充

- [x] 5.1 在 `SkillEntry` 與 `CommandEntry` 回傳實際預覽檔的可選內容指標，並以 Rust 測試驗證一般與 provider-aware 掃描。
- [x] 5.2 在 Skills／Commands 清單最右側顯示共用精簡 token 格式，開啟預覽後顯示共用完整內容指標。
- [x] 5.3 執行 Rust 測試、前端 lint 與 production build，並驗證無可讀預覽檔時既有功能不受影響。
