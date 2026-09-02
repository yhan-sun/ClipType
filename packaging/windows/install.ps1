[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$SourceExe,

    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA "Programs\ClipType"),

    [switch]$StartAtLogin,

    [switch]$NoLaunch
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$source = (Resolve-Path -LiteralPath $SourceExe).Path
if ([IO.Path]::GetFileName($source) -ne "cliptype.exe") {
    throw "Source executable must be named cliptype.exe"
}

New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null
$target = Join-Path $InstallRoot "cliptype.exe"
Copy-Item -LiteralPath $source -Destination $target -Force

$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$valueName = "ClipType"
$command = '"{0}" --background' -f $target
if ($StartAtLogin) {
    New-Item -Path $runKey -Force | Out-Null
    New-ItemProperty -Path $runKey -Name $valueName -PropertyType String -Value $command -Force | Out-Null
}

if (-not $NoLaunch) {
    Start-Process -FilePath $target -ArgumentList "--background"
}

Write-Output "cliptype_install result=ok startup=$($StartAtLogin.IsPresent) launched=$(-not $NoLaunch.IsPresent)"
