# Tasks

## 1. 可行性驗證（macOS 打包與簽章）

- [x] 1.1 確認有 Xcode 的環境（本機或 CI）：執行 `xcrun --find actool` 成功，並把環境需求寫入 `docs/` 或 README 的 macOS 建置段落
- [x] 1.2 實測 Tauri 2.12 打包是否支援 `.icon`／`Assets.car`：以 `bun run tauri build` 產生 `.app`，檢查 `Contents/Resources` 與 Info.plist 的 `CFBundleIconName`，結果記錄在 design.md 的 D2
- [x] 1.3 實測 App Group 與簽章：以現有簽章方式打包，確認可寫入 App Group 容器；記錄結果於 design.md 的 D3／D4（決定路線 B 是否繼續）

## 2. 選單列 template 圖示

- [ ] 2.1 新增單色 alpha mask 底圖（macOS 專用資源），並在 `app_setup::build_tray_icon` 以 `cfg(target_os = "macos")` 設定 `icon_as_template(true)`；驗證：打包後的 `.app` 在淺色／深色選單列皆清晰
- [x] 2.2 在 `tray_icon.rs` 新增 template 繪製分支：`icon_only` 三段填充、`percentage`／`bar` 單色；驗證：新增單元測試涵蓋三段門檻（<50%、50–80%、>80%）且輸出像素只含黑色與透明
- [ ] 2.3 確認 Windows 路徑不變：`cargo test` 中既有 tray_icon 測試通過，且 Windows 仍輸出彩色圖示

## 3. App 圖示多外觀

- [ ] 3.1 依任務 1.2 的結論建立 `.icon` 資源（或單色 `.icns` 退路），驗證：`.app` 中存在 `Assets.car` 並於 macOS 26 切換圖示風格後 Dock 圖示變色
- [x] 3.2 更新打包流程（`tauri.conf.json` 或打包後腳本）與重簽步驟，驗證：全新打包的 `.app` 通過 `codesign --verify`
- [x] 3.3 在 README 記錄「圖示風格只在打包後的 `.app` 生效，`tauri dev` 不會」

## 4. 活動快照共用

- [x] 4.1 將 `get_overlay_activity` 的合併邏輯與 quota 快照抽成 `ActivitySnapshot` 產生函式，overlay 與 tray panel 改用它；驗證：既有 `merge_activity` 測試保留並通過，overlay 行為不變
- [x] 4.2 實作節流且原子的 `activity-snapshot.json` 寫入（含 `generatedAt`，不含憑證或完整路徑）；驗證：單元測試涵蓋原子寫入、節流與欄位內容
- [x] 4.3 實作 App Group 路徑解析與退回 `app_data_dir` 的邏輯；驗證：無 App Group 時行為與目前相同的單元測試

## 5. 原生 widget（僅在任務 1.1–1.3 皆通過時進行）

- [ ] 5.1 建立 WidgetKit extension：讀取快照檔並顯示紅／黃／灰狀態與各平台用量；驗證：在 Widget 圖庫中可新增且顯示資料
- [ ] 5.2 實作過期處理：快照超過有效期限時顯示灰燈與「資料過期」；驗證：結束 App 後 widget 於期限後轉為過期狀態
- [ ] 5.3 將 extension 納入打包與簽章流程；驗證：乾淨環境打包後 widget 仍可載入

## 6. 整合驗證

- [ ] 6.1 在打包後的 macOS `.app` 驗證：選單列、Dock 圖示、overlay 狀態燈與（若有）widget 的狀態一致
- [ ] 6.2 在 Windows 驗證：系統匣圖示與 overlay 行為與變更前相同

## Workflow follow-up

- 若任務 1 的驗證顯示原生 widget 不可行，以路線 A（共用快照的 Tauri 視窗）結案，並在 design.md 記錄原因。
- 完成後執行 `/opsx:archive`。
