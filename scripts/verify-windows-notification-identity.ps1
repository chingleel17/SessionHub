param(
    [string]$ShortcutPath,
    [string]$ExpectedAppId = "com.ching.sessionhub"
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($ShortcutPath)) {
    $startMenu = [Environment]::GetFolderPath([Environment+SpecialFolder]::StartMenu)
    $shortcuts = @(Get-ChildItem -LiteralPath $startMenu -Filter "SessionHub.lnk" -File -Recurse)
    if ($shortcuts.Count -ne 1) {
        throw "Expected exactly one SessionHub Start-menu shortcut under $startMenu, found $($shortcuts.Count)."
    }
    $ShortcutPath = $shortcuts[0].FullName
}

if (-not (Test-Path -LiteralPath $ShortcutPath -PathType Leaf)) {
    throw "SessionHub shortcut was not found: $ShortcutPath"
}

$shell = New-Object -ComObject Shell.Application
$folder = $shell.NameSpace([System.IO.Path]::GetDirectoryName($ShortcutPath))
$shortcut = $folder.ParseName([System.IO.Path]::GetFileName($ShortcutPath))
$appId = $shortcut.ExtendedProperty("System.AppUserModel.ID")
$target = $shortcut.ExtendedProperty("System.Link.TargetParsingPath")
$icon = $shortcut.ExtendedProperty("System.Link.IconLocation")

if ($appId -ne $ExpectedAppId) {
    throw "Expected shortcut AUMID '$ExpectedAppId', found '$appId'."
}

if ([string]::IsNullOrWhiteSpace($target) -or -not (Test-Path -LiteralPath $target -PathType Leaf)) {
    throw "Shortcut target is missing or invalid: $target"
}

if (-not [string]::IsNullOrWhiteSpace($icon) -and -not $icon.StartsWith($target, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Shortcut icon does not resolve from the installed SessionHub executable: $icon"
}

"Verified SessionHub notification identity: $appId"
