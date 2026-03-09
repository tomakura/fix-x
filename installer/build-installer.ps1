param(
    [ValidateSet("debug", "release")]
    [string]$Configuration = "release"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$outputDir = Join-Path $repoRoot "dist"
$stagingDir = Join-Path $outputDir "installer-staging"
$targetExe = Join-Path $repoRoot "target\\$Configuration\\fix-x.exe"
$installerExe = Join-Path $outputDir "fix-x-installer.exe"
$sedFile = Join-Path $stagingDir "fix-x.sed"
$iexpress = Join-Path $env:WINDIR "System32\\iexpress.exe"

if (-not (Test-Path $targetExe)) {
    $cargoArgs = @("build")
    if ($Configuration -eq "release") {
        $cargoArgs += "--release"
    }

    Push-Location $repoRoot
    try {
        cargo @cargoArgs
    } finally {
        Pop-Location
    }
}

if (Test-Path $stagingDir) {
    Remove-Item $stagingDir -Recurse -Force
}

New-Item -ItemType Directory -Path $stagingDir -Force | Out-Null
New-Item -ItemType Directory -Path $outputDir -Force | Out-Null

Copy-Item $targetExe (Join-Path $stagingDir "fix-x.exe") -Force
Copy-Item (Join-Path $PSScriptRoot "install.ps1") (Join-Path $stagingDir "install.ps1") -Force
Copy-Item (Join-Path $PSScriptRoot "uninstall.ps1") (Join-Path $stagingDir "uninstall.ps1") -Force

$escapedStagingDir = ($stagingDir.TrimEnd('\') + '\')

$sed = @"
[Version]
Class=IEXPRESS
SEDVersion=3
[Options]
PackagePurpose=InstallApp
ShowInstallProgramWindow=0
HideExtractAnimation=1
UseLongFileName=1
InsideCompressed=1
CAB_FixedSize=0
CAB_ResvCodeSigning=0
RebootMode=N
InstallPrompt=
DisplayLicense=
FinishMessage=
TargetName=$installerExe
FriendlyName=fix-x Installer
AppLaunched=powershell.exe -NoProfile -ExecutionPolicy Bypass -File install.ps1
PostInstallCmd=<None>
AdminQuietInstCmd=powershell.exe -NoProfile -ExecutionPolicy Bypass -File install.ps1
UserQuietInstCmd=powershell.exe -NoProfile -ExecutionPolicy Bypass -File install.ps1
SourceFiles=SourceFiles
[Strings]
FILE0="fix-x.exe"
FILE1="install.ps1"
FILE2="uninstall.ps1"
[SourceFiles]
SourceFiles0=$escapedStagingDir
[SourceFiles0]
%FILE0%=
%FILE1%=
%FILE2%=
"@

Set-Content -Path $sedFile -Value $sed -Encoding ASCII

& $iexpress /N $sedFile | Out-Null

if (-not (Test-Path $installerExe)) {
    throw "Installer was not created: $installerExe"
}

Write-Host "Created installer: $installerExe"
