#Requires -Version 7.2
# Run only on a CLEAN disposable Windows 11 x64 desktop VM, logged in as a non-admin.
# Inputs must be trusted, built locally from the reviewed commit. This executes the installer.
[CmdletBinding()]
param([Parameter(Mandatory)][string]$Payload, [Parameter(Mandatory)][string]$CandidateDirectory)
$ErrorActionPreference = 'Stop'
$os = Get-CimInstance Win32_OperatingSystem
if ($os.ProductType -ne 1 -or $os.Caption -notmatch 'Windows 11' -or [int]$os.BuildNumber -lt 22000 -or
    [Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64' -or ![Environment]::UserInteractive) {
    throw 'An interactive Windows 11 x64 client OS is required, not Windows Server'
}
$out = (Resolve-Path -LiteralPath $CandidateDirectory).Path
$stage = (Resolve-Path -LiteralPath $Payload).Path
$installer = Get-Item -LiteralPath "$out/Volt-Launcher-Windows-x64-Setup.exe"
$zip = Get-Item -LiteralPath "$out/Volt-Launcher-Windows-x64-Portable.zip"
$sums = Join-Path $out 'SHA256SUMS'
if (Test-Path -LiteralPath $sums) { throw 'Use a fresh candidate directory without stale verification results' }
$initial = @($installer,$zip) | ForEach-Object { (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash }
$root = Join-Path ([IO.Path]::GetTempPath()) ('Volt release verification ' + [guid]::NewGuid())
$installed = Join-Path $root 'Installed app'
$portable = Join-Path $root 'Portable app'
$null = New-Item -ItemType Directory -Path $root
function Test-Payload($directory, [switch]$Exact) {
    $files = @(Get-ChildItem -LiteralPath $stage -Recurse -File -Force)
    if ($Exact -and @(Get-ChildItem -LiteralPath $directory -Recurse -File -Force).Count -ne $files.Count) { throw 'Unexpected ZIP contents' }
    foreach ($file in $files) {
        $name = [IO.Path]::GetRelativePath($stage, $file.FullName)
        if ((Get-FileHash -LiteralPath $file.FullName).Hash -ne (Get-FileHash -LiteralPath (Join-Path $directory $name)).Hash) { throw "Payload mismatch: $name" }
    }
}
function Test-Startup($directory) {
    $info = [Diagnostics.ProcessStartInfo]::new((Join-Path $directory 'Volt Launcher.exe'))
    $info.UseShellExecute = $false
    $info.WorkingDirectory = $directory
    foreach ($key in 'HORIZON_HOME','HORIZON_LAUNCHER_HOME','XDG_DATA_HOME','XDG_CONFIG_HOME','HORIZON_LAUNCHER_TRASH','HORIZON_LAUNCHER_CLIPBOARD','HORIZON_LAUNCHER_DESKTOP') {
        $info.Environment[$key] = Join-Path $root ((Split-Path -Leaf $directory) + " data/$key")
    }
    $info.Environment['HORIZON_LAUNCHER_NO_SELF_UPDATE'] = '1'
    $info.Environment['HORIZON_NO_AUDIO'] = '1'
    $app = [Diagnostics.Process]::Start($info)
    try {
        $deadline = [datetime]::UtcNow.AddSeconds(60)
        do {
            Start-Sleep -Milliseconds 250
            $app.Refresh()
            if ($app.HasExited) { throw "Launcher exited during startup: $($app.ExitCode)" }
            if ($app.MainWindowHandle -ne 0 -and $app.MainWindowTitle -eq 'Volt Launcher' -and $app.Responding) { break }
            if ([datetime]::UtcNow -ge $deadline) { throw 'No responding Volt Launcher window' }
        } while ($true)
        Start-Sleep -Seconds 5
        $app.Refresh()
        if ($app.HasExited -or !$app.Responding -or $app.MainWindowTitle -ne 'Volt Launcher') { throw 'Launcher window did not remain ready' }
        if (!$app.CloseMainWindow() -or !$app.WaitForExit(15000) -or $app.ExitCode -ne 0) { throw 'Launcher did not shut down cleanly' }
    } finally {
        if (!$app.HasExited) { $app.Kill($true); $app.WaitForExit() }
        $app.Dispose()
    }
    # A responsive HWND is not a presented frame. Exercise the installed/extracted
    # binary itself, not another copy from target/ or the staging directory.
    $info.ArgumentList.Add('--smoke-frame')
    $probe = [Diagnostics.Process]::Start($info)
    try {
        if (!$probe.WaitForExit(120000) -or $probe.ExitCode -ne 0) { throw 'Packaged launcher failed real frame/navigation probe' }
    } finally {
        if (!$probe.HasExited) { $probe.Kill($true); $probe.WaitForExit() }
        $probe.Dispose()
    }
}
# Keep the private VM and scratch folder for diagnosis on failure. Discard VM after test.
[IO.Compression.ZipFile]::ExtractToDirectory($zip.FullName, $portable)
Test-Payload $portable -Exact
Test-Startup $portable
$p = Start-Process -FilePath $installer.FullName -ArgumentList "/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /DIR=`"$installed`"" -PassThru -Wait
if ($p.ExitCode -ne 0) { throw "Install failed: $($p.ExitCode)" }
Test-Payload $installed
Test-Startup $installed
$p = Start-Process -FilePath "$installed/unins000.exe" -ArgumentList '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART' -PassThru -Wait
if ($p.ExitCode -ne 0 -or (Test-Path -LiteralPath "$installed/Volt Launcher.exe")) { throw 'Uninstall failed' }
$lines = for ($i = 0; $i -lt 2; $i++) {
    $file = @($installer,$zip)[$i]
    if ((Get-FileHash -LiteralPath $file.FullName).Hash -ne $initial[$i]) { throw "Candidate changed during test: $file" }
    "$($initial[$i].ToLowerInvariant())  $($file.Name)"
}
$lines | Set-Content -LiteralPath $sums -Encoding ascii
Write-Host "Install, portable extraction and window startup passed on $($os.Caption) build $($os.BuildNumber). Not gameplay or account proof."
