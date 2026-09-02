[CmdletBinding()]
param(
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA "Programs\ClipType"),

    [switch]$RemoveSettings
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$target = Join-Path $InstallRoot "cliptype.exe"
Get-Process -Name "cliptype" -ErrorAction SilentlyContinue | ForEach-Object {
    try {
        if ($_.Path -and ([IO.Path]::GetFullPath($_.Path) -eq [IO.Path]::GetFullPath($target))) {
            Stop-Process -Id $_.Id -Force
        }
    } catch {
        # A process can exit between enumeration and inspection.
    }
}

$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$valueName = "ClipType"
$current = (Get-ItemProperty -Path $runKey -Name $valueName -ErrorAction SilentlyContinue).$valueName
if ($current -and $current.StartsWith(('"{0}"' -f $target), [StringComparison]::OrdinalIgnoreCase)) {
    Remove-ItemProperty -Path $runKey -Name $valueName -ErrorAction SilentlyContinue
}

if (Test-Path -LiteralPath $InstallRoot) {
    Remove-Item -LiteralPath $InstallRoot -Recurse -Force
}

if ($RemoveSettings) {
    $settingsRoot = Join-Path $env:LOCALAPPDATA "ClipType"
    if (Test-Path -LiteralPath $settingsRoot) {
        Remove-Item -LiteralPath $settingsRoot -Recurse -Force
    }
}

Write-Output "cliptype_uninstall result=ok settings_removed=$($RemoveSettings.IsPresent)"
