# Spec Delta

## Purpose

定義專案 OpenSpec 資料的分層讀取與逐步呈現，使進行中變更不受大量封存資料掃描時間影響，同時確保完整讀取相容、重整一致與部分失敗時仍能使用已載入文件。

## ADDED Requirements

### Requirement: OpenSpec 支援指定區段與相容的完整掃描

系統 SHALL 允許 `get_project_specs` 以可選的 `section` 指定 `active`、`specs`、`archived` 或 `all`；回傳維持相同資料形狀，未指定時與 `all` 一樣回傳完整結果。

#### Scenario: 僅掃描進行中變更

- **WHEN** 呼叫方指定 `section: "active"`
- **THEN** 回傳 schema 與進行中變更，其他區段清單為空
- **AND** 大量主規格或封存目錄不需讀取後才可回傳進行中結果

#### Scenario: 僅掃描其餘區段

- **WHEN** 呼叫方指定 `section: "specs"` 或 `section: "archived"`
- **THEN** 僅對應的清單含資料，其餘清單為空，schema 為空值

#### Scenario: 舊呼叫仍可取得完整資料

- **WHEN** 呼叫方不提供 section，或指定 `section: "all"`
- **THEN** 回傳所有區段與 schema
- **AND** 分別讀取三個區段後合併的結果與完整讀取等價

#### Scenario: 專案不存在 OpenSpec 目錄

- **WHEN** 指定專案沒有 `openspec/`
- **THEN** 各區段回傳空清單，schema 為空值

### Requirement: 專案畫面逐層顯示 OpenSpec

系統 SHALL 先呈現進行中變更，再依序補入主規格及封存變更；後續區段載入期間，已載入文件仍可操作。

#### Scenario: 進行中資料先完成

- **WHEN** 進行中變更已載入，而主規格或封存變更仍在載入
- **THEN** Explorer 顯示進行中變更且可開啟其文件
- **AND** 畫面顯示目前背景載入的區段，不誤顯示沒有資料

#### Scenario: 區段讀取失敗

- **WHEN** 任一區段讀取失敗
- **THEN** 畫面顯示可辨識的失敗訊息與重整途徑
- **AND** 已成功載入的文件不因失敗而消失
- **AND** 不持續顯示尚未啟動區段的載入中狀態

### Requirement: 分層資料重整與進度寫入保持一致

系統 SHALL 依專案及區段隔離快取，讓 watcher 與手動重整重新取得區段結果；編輯 tasks 後立即更新所屬區段的顯示進度，寫入失敗時還原。

#### Scenario: 重整專案資料

- **WHEN** 使用者按重整或專案檔案變更觸發掃描
- **THEN** 進行中變更先重掃，主規格與封存變更依序更新
- **AND** A 專案的結果不會套用到 B 專案

#### Scenario: 編輯進行中或封存的 tasks

- **WHEN** 使用者編輯某 change 的 `tasks.md`
- **THEN** 對應區段的進度立即反映編輯結果
- **AND** 寫入失敗時恢復原進度
