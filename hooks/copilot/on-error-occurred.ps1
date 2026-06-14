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
    $title = if ($payload.PSObject.Properties["error"] -and $payload.error) { [string]$payload.error.name } else { $null }
    $errorMessage = if ($payload.PSObject.Properties["error"] -and $payload.error) { [string]$payload.error.message } else { "unknown error" }
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "session.errored" -Payload $payload -Title $title -Error $errorMessage
} catch {
    Write-HookErrorLog -EventType "session.errored" -Message $_.Exception.Message
}
