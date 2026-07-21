# 貢獻指南

感謝你協助改善 SessionHub。Issue 與 Pull Request 請以繁體中文或英文撰寫，並提供足夠資訊讓其他人重現與驗證。

## 開始前

- 先搜尋既有的 Issue 與 Pull Request，避免重複回報。
- Bug、功能建議與安全漏洞請使用對應的模板或流程。
- 不要提交 API key、Access Token、session 內容、個人資料或 `.env`。
- 大型功能請先開 Issue 討論範圍，再開始實作。

## 開發環境

- Windows 10／11 x64
- Node.js 22+
- Bun
- Rust stable toolchain
- PowerShell 7 或 Windows PowerShell

```bash
bun install
bun run lint
bun run build
cd src-tauri
cargo test
```

## 程式碼規範

- 遵循既有 TypeScript、React 與 Rust 結構，不在子元件直接呼叫 Tauri `invoke()`。
- Rust command 應將業務邏輯委派給可測試的 internal function。
- 不使用 `unwrap()` 處理 production error path。
- 前後端新增欄位時同步更新型別與序列化名稱。
- 使用既有 i18n，不在 JSX 直接硬編寫使用者介面文字。
- 一般操作與導覽圖示使用 `src/components/Icons.tsx` 集中的 Lucide 映射；圖表、quota ring 與品牌資產才可保留專用 SVG。
- 優先使用 `src/components/ui/` 的 Button、IconButton、Select 與 `DropdownMenu`。無可見文字的操作必須使用 i18n accessible name。
- `bun run lint` 是必要的前端品質門檻，lint error 不得合併。

## Pull Request

1. 從 `main` 建立分支，命名為 `feature/...`、`fix/...` 或 `docs/...`。
2. 保持變更聚焦，避免把無關重構混入功能或修正。
3. 更新必要的測試、文件、CHANGELOG 或截圖。
4. 執行 `bun run lint`、`bun run build`、`cargo fmt -- --check`、`cargo clippy --all-targets --all-features` 與 `cargo test`。
5. 填寫 Pull Request 模板，說明測試結果與可能的破壞性變更。

CI 的前端 lint、前端建置、Rust 測試、依賴漏洞檢查與祕密掃描為必要門檻；fmt 與 Clippy 目前以報告方式執行，避免既有警告阻擋所有貢獻。至少一位維護者審查後，才會合併至 `main`。GitHub Copilot review 可作為輔助，不能取代人工審查。
