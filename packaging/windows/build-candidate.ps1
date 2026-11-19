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
foreach ($file in 'Volt Launcher.exe','horizon-client.exe','LICENSE','THIRD_PARTY_NOTICES.md') {
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
$out = [IO.Path]::GetFullPath($OutputDirectory)
if ((Test-Path -LiteralPath $out) -or $out.StartsWith($payloadDir + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Choose a new output directory outside the payload'
}
# Reject links and known private artifacts. The maintainer still audits the *entire* payload.
foreach ($entry in @(Get-Item -LiteralPath $payloadDir) + @(Get-ChildItem -LiteralPath $payloadDir -Force -Recurse)) {
    if ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Linked input: $entry" }
    if ($entry.Name -match '^(\.git|\.local|\.env(\..*)?|saves|accounts(\.json)?|profiles|logs|crash-reports|horizon-resources\.json|session\.json)$|\.(jar|pfx|keystore|jks|apk)$') {
        throw "Non-distributable input: $entry"
    }
}
$null = New-Item -ItemType Directory -Path $out
& $InnoCompiler "/DPayloadDir=$payloadDir" "/DOutputDir=$out" "/DVoltVersion=$Version" "$PSScriptRoot/launcher.iss"
if ($LASTEXITCODE -ne 0) { throw 'Installer compile failed' }
$zip = Join-Path $out 'Volt-Launcher-Windows-x64-Portable.zip'
[IO.Compression.ZipFile]::CreateFromDirectory($payloadDir, $zip)
Write-Host 'Private candidates only. Verify installed and extracted startup on Windows 11 before release.'
