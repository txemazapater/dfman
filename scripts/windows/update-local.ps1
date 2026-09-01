$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..\..')
Push-Location $repoRoot

try {
    Write-Host 'Updating dfman repository...'
    git pull --ff-only

    Write-Host 'Formatting Rust sources...'
    cargo fmt --all

    Write-Host 'Running tests...'
    cargo test --workspace

    Write-Host 'Installing dfman into Cargo bin...'
    cargo install --path crates/dfman-cli --force

    $dfman = Get-Command dfman -ErrorAction Stop
    Write-Host "dfman installed at: $($dfman.Source)"
}
finally {
    Pop-Location
}
