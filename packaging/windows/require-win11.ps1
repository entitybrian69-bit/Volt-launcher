#Requires -Version 7.2
# Used both before registration and inside the job: a service can report
# UserInteractive yet have no signed-in desktop that can complete OAuth.
$os = Get-CimInstance Win32_OperatingSystem -ErrorAction Stop
$session = (Get-Process -Id $PID).SessionId
if ($os.ProductType -ne 1 -or $os.Caption -notmatch 'Windows 11' -or [int]$os.BuildNumber -lt 22000 -or
    [Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64' -or
    [Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -ne 'X64' -or
    $session -eq 0 -or ![Environment]::UserInteractive -or
    !(Get-Process explorer -ErrorAction SilentlyContinue | Where-Object SessionId -eq $session)) {
    throw 'Use x64 PowerShell 7 in the signed-in Explorer desktop of a Windows 11 x64 client VM. Do not install the runner as a service.'
}
Write-Host "Interactive Windows 11 x64 desktop: $($os.Caption), build $($os.BuildNumber)."
