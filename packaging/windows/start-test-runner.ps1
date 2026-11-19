#Requires -Version 7.2
# Run locally in a new, disposable VM as a standard user. See docs/windows-acceptance.md.
[CmdletBinding()]
param([Parameter(Mandatory)][string]$RunnerZip,
      [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{64}$')][string]$ExpectedSHA256)
$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/require-win11.ps1"
if ((Get-FileHash -LiteralPath $RunnerZip -Algorithm SHA256).Hash -ne $ExpectedSHA256) { throw 'Runner ZIP checksum mismatch' }
foreach ($tool in 'git','cargo','rustup','python','pwsh') { $null = Get-Command $tool -ErrorAction Stop }
# Keep GitHub admin credentials off the runner. config.cmd prompts for the
# short-lived repository registration token locally, not in chat or Actions logs.
$dir = Join-Path $env:LOCALAPPDATA ('Volt acceptance runner ' + [guid]::NewGuid())
[IO.Compression.ZipFile]::ExtractToDirectory((Resolve-Path -LiteralPath $RunnerZip).Path, $dir)
Push-Location -LiteralPath $dir
try {
    ./config.cmd --url https://github.com/luciferstudios/Volt-launcherPc --name ('volt-desktop-' + [guid]::NewGuid()) --labels volt-win11-x64-interactive --no-default-labels --ephemeral
    if ($LASTEXITCODE -ne 0) { throw 'Runner registration failed' }
    ./run.cmd
    if ($LASTEXITCODE -ne 0) { throw 'Interactive runner failed' }
} finally { Pop-Location }
