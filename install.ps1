<#
.SYNOPSIS
    Installs NTIX via the official Inno Setup installer.

.DESCRIPTION
    Downloads the latest NTIX release from GitHub and runs the installer silently.
    Requires administrator privileges.

.EXAMPLE
    iwr -useb https://raw.githubusercontent.com/cristiandis/NTIX-DEV/master/install.ps1 | iex
#>

$ErrorActionPreference = 'Stop'

# Check admin privileges
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "NTIX installer requires administrator privileges." -ForegroundColor Red
    Write-Host "Please run PowerShell as Administrator and try again." -ForegroundColor Yellow
    exit 1
}

$repo = "cristiandis/NTIX-DEV"
$assetName = "ntix-setup.exe"

Write-Host "Fetching latest release from GitHub..." -ForegroundColor Cyan

try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" -UseBasicParsing
} catch {
    Write-Host "Failed to fetch release info: $_" -ForegroundColor Red
    exit 1
}

$asset = $release.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
if (-not $asset) {
    Write-Host "Could not find $assetName in release $($release.tag_name)." -ForegroundColor Red
    Write-Host "Available assets: $($release.assets.name -join ', ')" -ForegroundColor Yellow
    exit 1
}

$version = $release.tag_name -replace '^v', ''
$downloadUrl = $asset.browser_download_url
$tempFile = Join-Path $env:TEMP "ntix-setup-$version.exe"

Write-Host "Downloading NTIX $version..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
} catch {
    Write-Host "Download failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host "Running installer..." -ForegroundColor Cyan
try {
    $process = Start-Process -FilePath $tempFile -ArgumentList '/SILENT' -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        Write-Host "Installer exited with code $($process.ExitCode)." -ForegroundColor Yellow
    }
} catch {
    Write-Host "Installer failed: $_" -ForegroundColor Red
    exit 1
} finally {
    if (Test-Path $tempFile) { Remove-Item $tempFile -Force -ErrorAction SilentlyContinue }
}

Write-Host "NTIX $version installed successfully!" -ForegroundColor Green
Write-Host "Run 'ntix diff config.lua' to get started." -ForegroundColor Cyan
