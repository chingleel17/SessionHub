# Tasks

## 1. 準備可信的量測環境（阻擋所有後續步驟）

- [ ] 1.1 修正或暫時移開 `C:\Users\User\.config\opencode\agents\planbyproject.md`（`tools:` 為陣列，1.18.4 要求 object 或 undefined），使 `opencode debug skill` 不再 exit 1
- [ ] 1.2 終止所有殘留的 `opencode` 常駐程序，確認 `Get-Process opencode` 無輸出
- [ ] 1.3 在手動終端執行 `opencode debug skill`，確認能正常回傳且 log 中該 run 掃到 `~/.agents`、`~/.claude` 與專案 3 個根目錄（建立黃金基準）
- [ ] 1.4 撰寫「從 log 還原某 run 掃描根目錄集合」的可重複腳本，存於 scratchpad，供後續所有比對使用

## 2. 階段一：根因定位（未完成前不得進入第 3 節）

- [ ] 2.1 釐清 SessionHub 程序本身的啟動方式（開機自動啟動 / 手動捷徑 / 開發模式），記錄其父程序與權限脈絡（integrity level、token）
- [ ] 2.2 在 SessionHub 開啟的終端中執行 `node -e "console.log(require('os').homedir())"`，與 `opencode debug paths` 的 home 比對，確認 discovery 用的 home 是否同源
- [ ] 2.3 在 SessionHub 終端中測試 `~/.agents\skills` 與 `~/.claude\skills` 的逐項遍歷：確認每個 `Directory, ReparsePoint` 是否能解析到 `D:\ching\AI tool setting\agents\skills\<name>`，並與手動終端結果比對
- [ ] 2.4 在 SessionHub 終端執行 `opencode debug skill` 並以 1.4 腳本還原根目錄集合，確認與黃金基準的差異
- [ ] 2.5 檢查 log 中是否出現 `failed to scan global skills` / `failed to scan project skills` 錯誤（`u()` 的靜默失敗路徑），若有則記錄其 `error` 內容
- [ ] 2.6 釐清 log 中 `dcad77ba`（count=15，只有 global、無 project）該 run 的來源終端，作為對稱失效的線索
- [ ] 2.7 從 SessionHub 終端逐步還原至手動終端狀態（環境變數、cwd、權限脈絡各為一個維度），找出使根目錄集合恢復一致的最小差異
- [ ] 2.8 將結論（含已排除項目與最小重現步驟）寫回 `design.md` 的 Open Questions 與 Context 章節

## 3. 階段二：實作修正（依 2.8 結論選擇 design D4 決策樹分支）

- [ ] 3.1 在 `src-tauri/src/sessions/mod.rs` 抽出統一的子程序環境組態 helper，涵蓋現有 MSYS 合併語意與完整環境區塊傳遞
- [ ] 3.2 依 2.8 結論在該 helper 中實作修正（環境傳遞 / 權限脈絡 / cwd 脈絡三擇一或組合），不硬編 skills 路徑、不注入 `HOME`/`USERPROFILE`/`XDG_*`
- [ ] 3.3 將 `open_terminal_internal`（`sessions/copilot.rs`）改為呼叫統一 helper
- [ ] 3.4 將 `open_in_tool_internal` 的所有 provider 分支（`commands/tools.rs`）改為呼叫統一 helper
- [ ] 3.5 將 `resume_session_in_terminal_internal`（`commands/tools.rs`）改為呼叫統一 helper
- [ ] 3.6 若 2.8 結論指向跨磁碟 junction 解析，評估並實作 `agents_config.rs` 的連結型別調整，且不破壞既有 `agents-skills-sync` 規格的狀態判定語意

## 4. 測試

- [ ] 4.1 為統一 helper 新增單元測試：驗證既有環境變數不被移除或覆寫
- [ ] 4.2 新增測試：驗證使用者已設定的 `OPENCODE_DISABLE_EXTERNAL_SKILLS` / `OPENCODE_DISABLE_CLAUDE_CODE_SKILLS` / `OPENCODE_PURE` 維持原值
- [ ] 4.3 新增測試：驗證使用者未設定停用旗標時，系統不主動設定（含不設為空字串）
- [ ] 4.4 新增測試：驗證三個啟動入口產生相同的環境組態
- [ ] 4.5 確認既有 `merge_msys_options` 測試仍通過，合併語意未改變
- [ ] 4.6 執行 `cd src-tauri && cargo test`，全數通過
- [ ] 4.7 執行 `bun run build`，型別檢查與前端建置通過

## 5. 驗收

- [ ] 5.1 重新建置並啟動 SessionHub，從其開啟終端執行 `opencode debug skill`
- [ ] 5.2 以 1.4 腳本還原根目錄集合，確認包含 `~/.agents`、`~/.claude` 與專案 3 個根目錄，與手動終端一致
- [ ] 5.3 以相同方式驗證 `open_in_tool`（opencode）與 `resume_session_in_terminal` 兩個入口
- [ ] 5.4 驗證使用者若設定 `OPENCODE_DISABLE_EXTERNAL_SKILLS=1`，SessionHub 終端中全域 skills 確實不載入（停用旗標未被繞過）
- [ ] 5.5 確認使用者既有的未提交工作樹變更（`src/App.css`、`src/components/ContentViewer.tsx`、`src/styles/themes/*.css`、`openspec/changes/add-launch-on-startup/`）未被本變更影響
