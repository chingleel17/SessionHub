param(
    [Parameter(Mandatory = $true)]
    [string]$BridgePath,
    [string]$Provider = "codex"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
Import-Module (Join-Path $PSScriptRoot "modules\record-event.psm1") -Force

try {
    $payload = Read-HookPayload
    if ($null -eq $payload) { exit 0 }
    $title = Get-HookStringValue -InputObject $payload -PropertyNames @("tool_name")
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "tool.post" -Payload $payload -Title $title
} catch {
    Write-HookErrorLog -EventType "tool.post" -Message $_.Exception.Message
}
