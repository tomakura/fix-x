Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$installRoot = Join-Path $env:LOCALAPPDATA "Programs\\fix-x"
$installedExe = Join-Path $installRoot "fix-x.exe"
$startMenuDir = Join-Path $env:APPDATA "Microsoft\\Windows\\Start Menu\\Programs"
$appShortcut = Join-Path $startMenuDir "fix-x.lnk"
$uninstallShortcut = Join-Path $startMenuDir "Uninstall fix-x.lnk"
$runKey = "HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run"

Write-Host "Uninstalling fix-x..."
Write-Host "fix-x をアンインストールしています..."

Get-Process -Name "fix-x" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

if (Test-Path $appShortcut) {
    Remove-Item $appShortcut -Force
}

if (Test-Path $uninstallShortcut) {
    Remove-Item $uninstallShortcut -Force
}

if (Test-Path $runKey) {
    Remove-ItemProperty -Path $runKey -Name "fix-x" -ErrorAction SilentlyContinue
}

if (Test-Path $installRoot) {
    Remove-Item $installRoot -Recurse -Force
}

Write-Host "fix-x has been removed."
Write-Host "fix-x のアンインストールが完了しました。"
