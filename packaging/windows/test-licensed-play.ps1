#Requires -Version 7.2
# Human-assisted: no password, device code, refresh token or account export is accepted as input.
[CmdletBinding()]
param([Parameter(Mandatory)][string]$Payload)
$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/require-win11.ps1"
if (!$env:HORIZON_MSA_CLIENT_ID) { throw 'Configure the approved public app ID (repository variable VOLT_MSA_CLIENT_ID)' }
$bin = (Resolve-Path -LiteralPath $Payload).Path
$engine = (Get-Item -LiteralPath "$bin/horizon-client.exe").FullName
$launcher = (Get-Item -LiteralPath "$bin/volt-launcher.exe").FullName
python "$PSScriptRoot/../../tools/windows_engine_tables.py" "$bin/engine-data.zip"
if ($LASTEXITCODE -ne 0) { throw 'Invalid engine tables' }
$root = Join-Path ([IO.Path]::GetTempPath()) ('Volt fresh 测试 ' + [guid]::NewGuid())
$info = [Diagnostics.ProcessStartInfo]::new($launcher)
$info.UseShellExecute = $false
$info.WorkingDirectory = $bin
# Never stream the human-assisted session's private output into Actions logs.
$info.RedirectStandardOutput = $true
$info.RedirectStandardError = $true
foreach ($key in 'HORIZON_HOME','HORIZON_LAUNCHER_HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','HORIZON_LAUNCHER_TRASH','HORIZON_LAUNCHER_CLIPBOARD','HORIZON_LAUNCHER_DESKTOP') {
    $info.Environment[$key] = Join-Path $root $key
}
$homeDir = $info.Environment['HORIZON_LAUNCHER_HOME']
if (Test-Path -LiteralPath $root) { throw 'Test must start without accounts, resources or saves' }
$info.Environment['HORIZON_NO_KEYCHAIN'] = '1' # real login; secrets live only for this run
$info.Environment['VOLT_QA_LICENSED_PLAY'] = '1'
$info.Environment['HORIZON_LAUNCHER_NO_SELF_UPDATE'] = '1'
$info.Environment['HORIZON_TELEMETRY_URL'] = 'off'
$info.Environment['HORIZON_NO_AUDIO'] = '1'
Write-Host 'At the VM desktop: sign in via Microsoft, choose Play, wait for install, create a singleplayer world, walk for at least 30 seconds, save and quit the game. Leave Volt Launcher open.'
$app = [Diagnostics.Process]::Start($info)
$game = $null
try {
    $app.BeginOutputReadLine()
    $app.BeginErrorReadLine()
    $deadline = [datetime]::UtcNow.AddMinutes(70)
    do {
        if ($app.HasExited) { throw 'Launcher exited before engine handoff' }
        $child = Get-CimInstance Win32_Process -Filter "ParentProcessId = $($app.Id) AND Name = 'horizon-client.exe'" | Where-Object ExecutablePath -eq $engine | Select-Object -First 1
        if ($child) { $game = Get-Process -Id $child.ProcessId; $null = $game.Handle; break }
        if ([datetime]::UtcNow -ge $deadline) { throw 'No authenticated Play handoff before timeout' }
        Start-Sleep -Seconds 2
    } while ($true)
    if (!$game.WaitForExit(1200000) -or $game.ExitCode -ne 0) { throw 'Engine did not exit cleanly after playing' }
    python "$PSScriptRoot/../../tools/check_licensed_world.py" $homeDir
    if ($LASTEXITCODE -ne 0) { throw 'Missing install or signed-in world evidence' }
    $app.Refresh()
    if ($app.HasExited -or !$app.Responding -or !$app.CloseMainWindow() -or !$app.WaitForExit(15000) -or $app.ExitCode -ne 0) { throw 'Launcher did not survive gameplay and close cleanly' }
    Write-Host 'Fresh Microsoft sign-in, official installation, Play handoff, saved world and return passed. No account data uploaded.'
} finally {
    if (!$app.HasExited) { $app.Kill($true); $app.WaitForExit() }
    if ($game) { if (!$game.HasExited) { $game.Kill($true); $game.WaitForExit() }; $game.Dispose() }
    $app.Dispose()
    # Do not copy this directory into the workspace/artifacts. Destroy the VM after testing.
}
