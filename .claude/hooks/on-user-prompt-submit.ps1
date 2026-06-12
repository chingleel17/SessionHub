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
    $prompt = Get-HookStringValue -InputObject $payload -PropertyNames @("prompt")
    if ($prompt -and $prompt.Length -gt 80) {
        $prompt = $prompt.Substring(0, 80)
    }
    Write-BridgeEventRecord -BridgePath $BridgePath -Provider $Provider -EventType "prompt.submitted" -Payload $payload -Title $prompt
} catch {
    Write-HookErrorLog -EventType "prompt.submitted" -Message $_.Exception.Message
}
