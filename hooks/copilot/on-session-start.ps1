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
    $timestamp = if ($payload.PSObject.Properties["timestamp"]) { [DateTimeOffset]::FromUnixTimeMilliseconds([int64]$payload.timestamp).UtcDateTime.ToString("o") } else { $null }
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "session.started" -Payload $payload -Timestamp $timestamp
} catch {
    Write-HookErrorLog -EventType "session.started" -Message $_.Exception.Message
}
