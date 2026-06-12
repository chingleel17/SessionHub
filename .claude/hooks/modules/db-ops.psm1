Set-StrictMode -Version Latest

function Invoke-WithRetry {
    param(
        [Parameter(Mandatory = $true)]
        [scriptblock]$ScriptBlock,

        [int]$MaxAttempts = 3,
        [int]$InitialDelayMs = 50
    )

    $attempt = 0
    $delayMs = $InitialDelayMs

    while ($attempt -lt $MaxAttempts) {
        try {
            return & $ScriptBlock
        } catch {
            $attempt++
            if ($attempt -ge $MaxAttempts) {
                throw
            }
            Start-Sleep -Milliseconds $delayMs
            $delayMs = [Math]::Min($delayMs * 2, 500)
        }
    }
}

Export-ModuleMember -Function Invoke-WithRetry
