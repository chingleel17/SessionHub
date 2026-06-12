Set-StrictMode -Version Latest

Import-Module (Join-Path $PSScriptRoot "db-ops.psm1") -Force
Import-Module (Join-Path $PSScriptRoot "task-queue.psm1") -Force

function Get-SessionHubLogDirectory {
    $appData = [Environment]::GetFolderPath("ApplicationData")
    return Join-Path $appData "SessionHub\logs"
}

function Ensure-SessionHubLogDirectory {
    $dir = Get-SessionHubLogDirectory
    if (-not (Test-Path -LiteralPath $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
    return $dir
}

function Write-HookErrorLog {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Message,

        [string]$EventType = "hook.error"
    )

    $logDir = Ensure-SessionHubLogDirectory
    $logPath = Join-Path $logDir "hook-errors.log"
    $entry = [ordered]@{
        timestamp = [DateTimeOffset]::UtcNow.ToString("o")
        level = "error"
        eventType = $EventType
        message = $Message
    }
    [System.IO.File]::AppendAllText(
        $logPath,
        (($entry | ConvertTo-Json -Compress) + [Environment]::NewLine),
        [System.Text.UTF8Encoding]::new($false)
    )
}

function Read-HookPayload {
    $raw = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($raw)) {
        return $null
    }

    try {
        return $raw | ConvertFrom-Json -Depth 20
    } catch {
        Write-HookErrorLog -Message ("Failed to parse hook payload: " + $_.Exception.Message)
        return $null
    }
}

function Get-HookStringValue {
    param(
        $InputObject,
        [string[]]$PropertyNames
    )

    foreach ($name in $PropertyNames) {
        if ($null -eq $InputObject) {
            continue
        }

        $property = $InputObject.PSObject.Properties[$name]
        if ($null -ne $property) {
            $value = [string]$property.Value
            if (-not [string]::IsNullOrWhiteSpace($value)) {
                return $value
            }
        }
    }

    return $null
}

function Add-BridgeRecord {
    param(
        [Parameter(Mandatory = $true)]
        [string]$BridgePath,

        [Parameter(Mandatory = $true)]
        [hashtable]$Record
    )

    Invoke-WithRetry -ScriptBlock {
        $parent = Split-Path -Parent $BridgePath
        if (-not [string]::IsNullOrWhiteSpace($parent) -and -not (Test-Path -LiteralPath $parent)) {
            New-Item -ItemType Directory -Path $parent -Force | Out-Null
        }

        [System.IO.File]::AppendAllText(
            $BridgePath,
            (($Record | ConvertTo-Json -Compress -Depth 20) + [Environment]::NewLine),
            [System.Text.UTF8Encoding]::new($false)
        )
    }
}

function Write-BridgeEventRecord {
    param(
        [Parameter(Mandatory = $true)]
        [string]$BridgePath,

        [Parameter(Mandatory = $true)]
        [string]$Provider,

        [Parameter(Mandatory = $true)]
        [string]$EventType,

        [Parameter(Mandatory = $true)]
        $Payload,

        [string]$Title,

        [string]$Error
    )

    $sessionId = Get-HookStringValue -InputObject $Payload -PropertyNames @("session_id", "sessionId")
    $cwd = Get-HookStringValue -InputObject $Payload -PropertyNames @("cwd")
    $sourcePath = Get-HookStringValue -InputObject $Payload -PropertyNames @("transcript_path", "transcriptPath")

    $record = [ordered]@{
        version = 4
        provider = $Provider
        eventType = $EventType
        timestamp = [DateTimeOffset]::UtcNow.ToString("o")
        sessionId = $sessionId
        cwd = $cwd
        sourcePath = $sourcePath
        title = $Title
        error = $Error
    }

    Add-BridgeRecord -BridgePath $BridgePath -Record $record
}

Export-ModuleMember -Function @(
    "Ensure-SessionHubLogDirectory",
    "Write-HookErrorLog",
    "Read-HookPayload",
    "Get-HookStringValue",
    "Write-BridgeEventRecord"
)
