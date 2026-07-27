## MODIFIED Requirements

### Requirement: Windows 路徑大小寫正規化
系統 SHALL 在分組時對 `cwd` 路徑進行大小寫不敏感的正規化，避免因磁碟代號或路徑大小寫差異而產生重複專案群組。顯示路徑 SHALL 將磁碟機代號轉為大寫，路徑其餘部分維持 session 原始資料的大小寫。

#### Scenario: 路徑大小寫不同但實際為相同目錄
- **WHEN** 兩個 session 的 `cwd` 分別為 `D:\ching\project` 與 `d:\ching\project`
- **THEN** 系統應將其視為相同專案群組，合併顯示

#### Scenario: 顯示路徑的磁碟機代號一律大寫
- **WHEN** 某專案群組的顯示路徑來源為 `d:\ching\SourceCode\Unity practise`
- **THEN** 畫面顯示為 `D:\ching\SourceCode\Unity practise`
- **AND** 磁碟機代號以外的部分維持原始大小寫，不被轉為大寫或小寫

## ADDED Requirements

### Requirement: 分組依據為改寫後的路徑
系統 SHALL 在套用專案路徑對應規則之後才進行分組，使原始 `cwd` 不同但經規則改寫為相同路徑的 session 歸入同一專案群組。

#### Scenario: 舊路徑與新路徑的 session 合併為同一群組
- **WHEN** 存在 `D:\old\proj` → `D:\new\proj` 的對應規則，且部分 session 的 `cwd` 為 `D:\old\proj`、部分為 `D:\new\proj`
- **THEN** 兩者顯示在同一專案群組下，路徑標籤為 `D:\new\proj`
