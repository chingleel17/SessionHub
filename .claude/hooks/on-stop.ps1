param(
    [Parameter(Mandatory = $true)]
    [string]$BridgePath,
    [string]$Provider = "claude"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Import-Module (Join-Path $PSScriptRoot "modules\record-event.psm1") -Force

try {
    $payload = Read-HookPayload
    if ($null -eq $payload) { exit 0 }
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "session.stop" -Payload $payload
} catch {
    Write-HookErrorLog -EventType "session.stop" -Message $_.Exception.Message
}
