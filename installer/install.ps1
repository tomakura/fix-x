Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$installRoot = Join-Path $env:LOCALAPPDATA "Programs\\fix-x"
$installedExe = Join-Path $installRoot "fix-x.exe"
$installedUninstaller = Join-Path $installRoot "uninstall.ps1"
$startMenuDir = Join-Path $env:APPDATA "Microsoft\\Windows\\Start Menu\\Programs"
$appShortcut = Join-Path $startMenuDir "fix-x.lnk"
$uninstallShortcut = Join-Path $startMenuDir "Uninstall fix-x.lnk"
$sourceExe = Join-Path $PSScriptRoot "fix-x.exe"
$sourceUninstaller = Join-Path $PSScriptRoot "uninstall.ps1"

Write-Host "Installing fix-x..."
Write-Host "fix-x をインストールしています..."

Get-Process -Name "fix-x" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

New-Item -ItemType Directory -Path $installRoot -Force | Out-Null
Copy-Item $sourceExe $installedExe -Force
Copy-Item $sourceUninstaller $installedUninstaller -Force

$wsh = New-Object -ComObject WScript.Shell

$appShortcutObject = $wsh.CreateShortcut($appShortcut)
$appShortcutObject.TargetPath = $installedExe
$appShortcutObject.WorkingDirectory = $installRoot
$appShortcutObject.IconLocation = $installedExe
$appShortcutObject.Save()

$uninstallShortcutObject = $wsh.CreateShortcut($uninstallShortcut)
$uninstallShortcutObject.TargetPath = "powershell.exe"
$uninstallShortcutObject.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$installedUninstaller`""
$uninstallShortcutObject.WorkingDirectory = $installRoot
$uninstallShortcutObject.Save()

Start-Process -FilePath $installedExe -WorkingDirectory $installRoot

Write-Host "fix-x installation completed."
Write-Host "fix-x のインストールが完了しました。"
