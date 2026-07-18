# Install termpad to ~/.cargo/bin so you can run: termpad [file]
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Root = Resolve-Path (Join-Path $PSScriptRoot '..')
Push-Location $Root
try {
    cargo install --path . --force
} finally {
    Pop-Location
}

$CargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
Write-Host ""
Write-Host "Installed: termpad -> $CargoBin\termpad.exe"
Write-Host ""
Write-Host "Make sure this directory is in your PATH:"
Write-Host "  $CargoBin"
Write-Host ""
Write-Host "Then run:"
Write-Host "  termpad              # empty buffer"
Write-Host "  termpad README.md    # open a file"
