param(
    [ValidateSet("debug", "release")]
    [string]$Configuration = "release"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$outputDir = Join-Path $repoRoot "dist"
$nativeInstallerRoot = Join-Path $PSScriptRoot "native"
$payloadDir = Join-Path $nativeInstallerRoot "payload"
$appExe = Join-Path $repoRoot "target\\$Configuration\\fix-x.exe"
$iconFile = Join-Path $repoRoot "assets\\logo.ico"
$installerExe = Join-Path $nativeInstallerRoot "target\\release\\fix-x-installer.exe"
$distInstallerExe = Join-Path $outputDir "fix-x-installer.exe"

Push-Location $repoRoot
try {
    $cargoArgs = @("build", "--bin", "fix-x")
    if ($Configuration -eq "release") {
        $cargoArgs += "--release"
    }
    cargo @cargoArgs
} finally {
    Pop-Location
}

if (-not (Test-Path $appExe)) {
    throw "App build output not found: $appExe"
}

if (-not (Test-Path $iconFile)) {
    throw "Logo icon not found: $iconFile"
}

New-Item -ItemType Directory -Path $payloadDir -Force | Out-Null
New-Item -ItemType Directory -Path $outputDir -Force | Out-Null

Get-Process -Name "fix-x-installer" -ErrorAction SilentlyContinue | Stop-Process -Force

Copy-Item $appExe (Join-Path $payloadDir "fix-x.exe") -Force
Copy-Item $iconFile (Join-Path $payloadDir "fix-x.ico") -Force

Push-Location $nativeInstallerRoot
try {
    cargo build --release
} finally {
    Pop-Location
}

if (-not (Test-Path $installerExe)) {
    throw "Installer build output not found: $installerExe"
}

Copy-Item $installerExe $distInstallerExe -Force

Write-Host "Created installer: $distInstallerExe"
