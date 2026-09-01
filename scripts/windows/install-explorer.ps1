$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$dfman = Get-Command dfman -ErrorAction Stop
$exe = $dfman.Source

$entries = @(
    @{
        Key = 'HKCU:\Software\Classes\Directory\shell\dfman'
        Label = 'Open in dfman'
        Command = ('"{0}" open "%1"' -f $exe)
    },
    @{
        Key = 'HKCU:\Software\Classes\Directory\Background\shell\dfman'
        Label = 'Open dfman here'
        Command = ('"{0}" open "%V"' -f $exe)
    }
)

foreach ($entry in $entries) {
    New-Item -Path $entry.Key -Force | Out-Null
    Set-ItemProperty -Path $entry.Key -Name '(default)' -Value $entry.Label
    Set-ItemProperty -Path $entry.Key -Name 'Icon' -Value $exe

    $commandKey = Join-Path $entry.Key 'command'
    New-Item -Path $commandKey -Force | Out-Null
    Set-ItemProperty -Path $commandKey -Name '(default)' -Value $entry.Command
}

Write-Host 'Explorer integration installed for the current user.'
Write-Host "Executable: $exe"
Write-Host 'On Windows 11 the entry may initially appear under "Show more options".'
