#Requires -Version 7.2
# Local equivalent of the attended job. No runner registration, GitHub credential,
# downloaded executable or public artifact upload is necessary on the test PC.
[CmdletBinding()]
param([Parameter(Mandatory)][string]$EngineTables,
      [Parameter(Mandatory)][ValidatePattern('^[0-9a-fA-F]{40}$')][string]$ReviewedCommit,
      [Parameter(Mandatory)][guid]$PublicClientId)
$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/require-win11.ps1"
if ($PublicClientId -eq [guid]::Empty) { throw 'Use the approved public application ID' }
$tables = (Resolve-Path -LiteralPath $EngineTables).Path
Push-Location "$PSScriptRoot/../.."
try {
    $head = git rev-parse HEAD
    if ($LASTEXITCODE -ne 0 -or $head -ne $ReviewedCommit) { throw 'Reviewed commit does not match the checkout' }
    $changes = git status --porcelain --untracked-files=normal
    if ($LASTEXITCODE -ne 0 -or $changes) { throw 'Use a fresh clean checkout; keep the downloaded ZIP outside it' }
    python tools/windows_engine_tables.py $tables
    if ($LASTEXITCODE -ne 0) { throw 'Invalid generated engine tables' }
    # Start from an empty target directory rather than trusting an old sibling binary.
    $env:CARGO_TARGET_DIR = Join-Path ([IO.Path]::GetTempPath()) ('Volt build ' + [guid]::NewGuid())
    $env:HORIZON_MSA_CLIENT_ID = $PublicClientId.ToString()
    $env:RUSTFLAGS = '-C target-feature=+crt-static'
    $env:CARGO_PROFILE_FAST_INCREMENTAL = 'false'
    cargo build --locked --profile fast --target x86_64-pc-windows-msvc -p horizon-client --bin horizon-client -p volt-launcher --bin volt-launcher --features horizon-client/mods
    if ($LASTEXITCODE -ne 0) { throw 'Native launcher/engine build failed' }
    $bin = Join-Path $env:CARGO_TARGET_DIR 'x86_64-pc-windows-msvc/fast'
    Copy-Item -LiteralPath $tables -Destination "$bin/engine-data.zip"
    & "$PSScriptRoot/test-licensed-play.ps1" -Payload $bin
} finally { Pop-Location }
