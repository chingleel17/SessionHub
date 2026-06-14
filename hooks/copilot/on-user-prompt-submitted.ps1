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
    $prompt = Get-HookStringValue -InputObject $payload -PropertyNames @("prompt")
    $title = if ([string]::IsNullOrEmpty($prompt)) { $null } elseif ($prompt.Length -gt 80) { $prompt.Substring(0, 80) } else { $prompt }
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "prompt.submitted" -Payload $payload -Title $title
} catch {
    Write-HookErrorLog -EventType "prompt.submitted" -Message $_.Exception.Message
}
