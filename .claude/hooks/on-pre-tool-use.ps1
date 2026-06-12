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
    $title = Get-HookStringValue -InputObject $payload -PropertyNames @("tool_name", "toolName")
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "tool.pre" -Payload $payload -Title $title
} catch {
    Write-HookErrorLog -EventType "tool.pre" -Message $_.Exception.Message
}
