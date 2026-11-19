#Requires -Version 7.2
# Private release engineering tool. Requires a real, prebuilt and audited public payload.
# Never points at target/, a player profile, the ancestral launcher, or an engine-only artifact.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Payload,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version,
    [string]$InnoCompiler = 'ISCC.exe'
)
$ErrorActionPreference = 'Stop'
$payloadDir = (Get-Item -LiteralPath $Payload).FullName
$publicFiles = @('Volt Launcher.exe','horizon-client.exe','engine-data.zip','LICENSE','THIRD_PARTY_NOTICES.md')
foreach ($file in $publicFiles) {
    if (!(Test-Path -LiteralPath (Join-Path $payloadDir $file) -PathType Leaf)) { throw "Missing public payload file: $file" }
}
if ((Get-FileHash -LiteralPath "$payloadDir/Volt Launcher.exe").Hash -eq (Get-FileHash -LiteralPath "$payloadDir/horizon-client.exe").Hash) {
    throw 'Refusing an engine renamed as a launcher'
}
$repo = (Resolve-Path "$PSScriptRoot/../..").Path
foreach ($file in 'Volt Launcher.exe','horizon-client.exe') {
    python "$repo/tools/check_imports.py" --windows-x64 "$payloadDir/$file"
    if ($LASTEXITCODE -ne 0) { throw "Failed DLL import audit: $file" }
}
python "$repo/tools/windows_engine_tables.py" "$payloadDir/engine-data.zip"
if ($LASTEXITCODE -ne 0) { throw 'Missing, private or invalid generated engine tables' }
$out = [IO.Path]::GetFullPath($OutputDirectory)
if ((Test-Path -LiteralPath $out) -or $out.StartsWith($payloadDir + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Choose a new output directory outside the payload'
}
# This statically linked payload has no redistributable DLL subdirectory. Do not
# use a blacklist: no scratch accounts, arbitrary archives, reports or saves.
if ((Get-Item -LiteralPath $payloadDir).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Linked staging directory' }
foreach ($entry in Get-ChildItem -LiteralPath $payloadDir -Force) {
    if ($entry.PSIsContainer -or ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $entry.Name -notin $publicFiles) {
        throw 'Unexpected or linked input in public staging directory'
    }
}
$null = New-Item -ItemType Directory -Path $out
& $InnoCompiler "/DPayloadDir=$payloadDir" "/DOutputDir=$out" "/DVoltVersion=$Version" "$PSScriptRoot/launcher.iss"
if ($LASTEXITCODE -ne 0) { throw 'Installer compile failed' }
$zip = Join-Path $out 'Volt-Launcher-Windows-x64-Portable.zip'
[IO.Compression.ZipFile]::CreateFromDirectory($payloadDir, $zip)
Write-Host 'Private candidates only. Verify installed and extracted startup on Windows 11 before release.'
