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

$settingsRoot = Join-Path $env:LOCALAPPDATA "ClipType"
$configPath = Join-Path $settingsRoot "config.toml"
New-Item -ItemType Directory -Force -Path $settingsRoot | Out-Null
$startupLiteral = if ($StartAtLogin) { "true" } else { "false" }
if (Test-Path -LiteralPath $configPath) {
    $content = [IO.File]::ReadAllText($configPath)
    $pattern = '(?m)^start_at_login = (true|false)\r?$'
    $matches = [Text.RegularExpressions.Regex]::Matches($content, $pattern)
    if ($matches.Count -ne 1) {
        throw "Existing ClipType settings do not contain exactly one start_at_login field"
    }
    $updated = [Text.RegularExpressions.Regex]::Replace(
        $content,
        $pattern,
        "start_at_login = $startupLiteral"
    )
} else {
    $updated = @"
version = 2
enabled = true
mode = "auto"
auto_clipboard_threshold = 256
speed = "normal"
characters_per_second = 40
jitter_percent = 0
typo_probability_percent = 0
notifications = true
start_at_login = $startupLiteral
trigger_hotkey = "ctrl+alt+shift+v"
cancel_hotkey = "ctrl+alt+shift+x"
"@
}
$configTemp = "$configPath.install.tmp"
[IO.File]::WriteAllText($configTemp, $updated, [Text.UTF8Encoding]::new($false))
Move-Item -LiteralPath $configTemp -Destination $configPath -Force

$runKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$valueName = "ClipType"
$command = '"{0}" --background' -f $target
if ($StartAtLogin) {
    New-Item -Path $runKey -Force | Out-Null
    New-ItemProperty -Path $runKey -Name $valueName -PropertyType String -Value $command -Force | Out-Null
} else {
    $properties = Get-ItemProperty -Path $runKey -Name $valueName -ErrorAction SilentlyContinue
    $current = if ($null -ne $properties) { $properties.$valueName } else { $null }
    if ($current -and $current.StartsWith(('"{0}"' -f $target), [StringComparison]::OrdinalIgnoreCase)) {
        Remove-ItemProperty -Path $runKey -Name $valueName -ErrorAction SilentlyContinue
    }
}

if (-not $NoLaunch) {
    Start-Process -FilePath $target -ArgumentList "--background"
}

Write-Output "cliptype_install result=ok startup=$($StartAtLogin.IsPresent) launched=$(-not $NoLaunch.IsPresent)"
