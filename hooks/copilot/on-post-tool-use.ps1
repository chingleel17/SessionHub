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
    $title = Get-HookStringValue -InputObject $payload -PropertyNames @("toolName")
    $resultType = if ($payload.PSObject.Properties["toolResult"] -and $payload.toolResult) { [string]$payload.toolResult.resultType } else { $null }
    $errorMessage = if ($resultType -eq "failure" -or $resultType -eq "denied") { "tool $title $resultType" } else { $null }
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "tool.post" -Payload $payload -Title $title -Error $errorMessage
} catch {
    Write-HookErrorLog -EventType "tool.post" -Message $_.Exception.Message
}
