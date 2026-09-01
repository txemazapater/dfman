$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..\..')
Push-Location $repoRoot

try {
    Write-Host 'Checking required tools...'
    Get-Command git -ErrorAction Stop | Out-Null
    Get-Command cargo -ErrorAction Stop | Out-Null
    Get-Command rustc -ErrorAction Stop | Out-Null

    Write-Host 'Formatting Rust sources...'
    cargo fmt --all

    Write-Host 'Running tests...'
    cargo test --workspace

    Write-Host 'Installing dfman into Cargo bin...'
    cargo install --path crates/dfman-cli --force

    $dfman = Get-Command dfman -ErrorAction Stop
    Write-Host "dfman installed at: $($dfman.Source)"
    Write-Host 'Run scripts/windows/install-explorer.ps1 to add Explorer integration.'
}
finally {
    Pop-Location
}
