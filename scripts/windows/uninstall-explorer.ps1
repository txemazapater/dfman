$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$keys = @(
    'HKCU:\Software\Classes\Directory\shell\dfman',
    'HKCU:\Software\Classes\Directory\Background\shell\dfman'
)

foreach ($key in $keys) {
    if (Test-Path $key) {
        Remove-Item -Path $key -Recurse -Force
    }
}

Write-Host 'Explorer integration removed for the current user.'
