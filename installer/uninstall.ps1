Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$installRoot = Join-Path $env:LOCALAPPDATA "Programs\\fix-x"
$installedExe = Join-Path $installRoot "fix-x.exe"
$installedIcon = Join-Path $installRoot "fix-x.ico"
$startMenuDir = Join-Path $env:APPDATA "Microsoft\\Windows\\Start Menu\\Programs"
$appShortcut = Join-Path $startMenuDir "fix-x.lnk"
$uninstallShortcut = Join-Path $startMenuDir "Uninstall fix-x.lnk"
$runKey = "HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run"

Write-Host "Uninstalling fix-x..."

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

if (Test-Path $installedExe) {
    Remove-Item $installedExe -Force -ErrorAction SilentlyContinue
}

if (Test-Path $installedIcon) {
    Remove-Item $installedIcon -Force -ErrorAction SilentlyContinue
}

$cleanupCmd = "ping 127.0.0.1 -n 2 > nul & if exist `"$installRoot`" rmdir /s /q `"$installRoot`""
Start-Process -FilePath "cmd.exe" -ArgumentList "/c $cleanupCmd" -WindowStyle Hidden

Write-Host "fix-x has been removed."
