Set-StrictMode -Version Latest

function New-HookTask {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Type,

        [Parameter(Mandatory = $true)]
        [hashtable]$Payload
    )

    return [ordered]@{
        type = $Type
        payload = $Payload
        queuedAt = [DateTimeOffset]::UtcNow.ToString("o")
    }
}

function Get-HookTaskQueueItems {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable[]]$Tasks
    )

    return @($Tasks)
}

Export-ModuleMember -Function New-HookTask, Get-HookTaskQueueItems
