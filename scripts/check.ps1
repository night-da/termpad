# Run before commit: format check, clippy, tests
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Root = Resolve-Path (Join-Path $PSScriptRoot '..')
Push-Location $Root
try {
    Write-Host "cargo fmt --check ..."
    cargo fmt -- --check

    Write-Host "cargo clippy ..."
    cargo clippy -- -D warnings

    Write-Host "cargo test ..."
    cargo test
} finally {
    Pop-Location
}

Write-Host "All checks passed."
