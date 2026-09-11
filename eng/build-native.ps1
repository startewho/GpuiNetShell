#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Debug"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    if ($Configuration -eq "Release") {
        cargo build --release -p gpui-net-shell
        $profile = "release"
    }
    else {
        cargo build -p gpui-net-shell
        $profile = "debug"
    }
    Write-Host "Native host: target/$profile/gpui_net_shell.dll"
}
finally {
    Pop-Location
}
