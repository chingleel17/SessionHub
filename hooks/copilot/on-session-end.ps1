param(
    [Parameter(Mandatory = $true)]
    [string]$BridgePath,
    [string]$Provider = "copilot"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
Import-Module (Join-Path $PSScriptRoot "modules\record-event.psm1") -Force

try {
    $payload = Read-HookPayload
    if ($null -eq $payload) { exit 0 }
    $errorMessage = if ((Get-HookStringValue -InputObject $payload -PropertyNames @("reason")) -eq "error") { "copilot session ended with error" } else { $null }
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "session.ended" -Payload $payload -Error $errorMessage
} catch {
    Write-HookErrorLog -EventType "session.ended" -Message $_.Exception.Message
}
