# SessionHub 發布腳本
# 用法: .\scripts\release.ps1 -Version 0.1.7
#
# 流程:
#   1. 檢查工作區乾淨、位於 main 分支
#   2. 更新 package.json / Cargo.toml / Cargo.lock 版本號
#   3. 檢查 CHANGELOG.md 已有該版本的段落（沒有就中止，先寫 changelog）
#   4. commit + 建立 tag vX.Y.Z + push
#   5. GitHub Actions (release.yml) 會自動建置安裝檔並建立 draft release
#   6. 到 GitHub Releases 頁面檢查後按 Publish

param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

# 1. 前置檢查
$branch = git rev-parse --abbrev-ref HEAD
if ($branch -ne "main") { throw "目前在 '$branch' 分支，請切到 main 再發布" }

$dirty = git status --porcelain
if ($dirty) { throw "工作區有未提交的變更，請先 commit 或 stash:`n$dirty" }

$tag = "v$Version"
if (git tag -l $tag) { throw "tag $tag 已存在" }

# 2. 檢查 CHANGELOG 已有該版本段落
$changelog = Get-Content "CHANGELOG.md" -Raw
if ($changelog -notmatch [regex]::Escape("## [$Version]")) {
    throw "CHANGELOG.md 中找不到 '## [$Version]' 段落，請先撰寫該版本的變更內容（新增/調整/修正/移除）"
}

# 3. 更新版本號
Write-Host "更新版本號至 $Version ..." -ForegroundColor Cyan

$pkg = Get-Content "package.json" -Raw
$pkg = $pkg -replace '"version":\s*"[\d.]+"', "`"version`": `"$Version`""
Set-Content "package.json" $pkg -NoNewline

$cargo = Get-Content "src-tauri/Cargo.toml" -Raw
$cargo = $cargo -replace '(?m)^version\s*=\s*"[\d.]+"', "version = `"$Version`""
Set-Content "src-tauri/Cargo.toml" $cargo -NoNewline

Push-Location src-tauri
cargo update -p session-hub --offline | Out-Null
Pop-Location

# 4. commit + tag + push
Write-Host "建立 commit 與 tag $tag ..." -ForegroundColor Cyan
git add package.json src-tauri/Cargo.toml src-tauri/Cargo.lock CHANGELOG.md
git commit -m "chore: release $tag"
git tag $tag
git push origin main
git push origin $tag

Write-Host ""
Write-Host "完成！GitHub Actions 正在建置安裝檔。" -ForegroundColor Green
Write-Host "建置完成後會產生 draft release，請到以下網址檢查並按 Publish：" -ForegroundColor Green
Write-Host "  https://github.com/chingleel17/SessionHub/releases" -ForegroundColor Yellow
Write-Host "查看建置進度： gh run watch" -ForegroundColor Yellow
