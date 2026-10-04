# Builds Wishboard and packs the release zip into dist\Wishboard-v1.0.0-win-x64.zip
# Usage: powershell -ExecutionPolicy Bypass -File scripts\build.ps1 [-NoBundle]
param([switch]$NoBundle)
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$version = '1.0.0'
$release = "Wishboard-v$version-win-x64.zip"
$dist = Join-Path $root 'dist'

Push-Location (Join-Path $root 'src-tauri')
try {
    $hasTauri = $null -ne (Get-Command cargo-tauri -ErrorAction SilentlyContinue)
    if ($hasTauri -and -not $NoBundle) {
        Write-Host '> cargo tauri build' -ForegroundColor DarkYellow
        cargo tauri build
    } else {
        Write-Host '> cargo build --release (tauri-cli not found or -NoBundle)' -ForegroundColor DarkYellow
        cargo build --release
    }
    if ($LASTEXITCODE -ne 0) { throw "build failed with exit code $LASTEXITCODE" }
} finally {
    Pop-Location
}

$target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'src-tauri\target' }
$exe = Join-Path $target 'release\Wishboard.exe'
if (-not (Test-Path $exe)) { $exe = Join-Path $target 'release\wishboard.exe' }
if (-not (Test-Path $exe)) { throw "executable not found under $target\release" }

$stage = Join-Path $dist 'stage'
Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item $exe (Join-Path $stage 'Wishboard.exe')
Copy-Item (Join-Path $root 'LICENSE') $stage
Get-ChildItem (Join-Path $target 'release\bundle\nsis') -Filter '*.exe' -ErrorAction SilentlyContinue |
    ForEach-Object { Copy-Item $_.FullName (Join-Path $stage "Wishboard-v$version-setup.exe") }

$zip = Join-Path $dist $release
Remove-Item $zip -Force -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
Remove-Item $stage -Recurse -Force
$size = [math]::Round((Get-Item $zip).Length / 1MB, 1)
Write-Host "Packed $zip ($size MB)" -ForegroundColor Green
