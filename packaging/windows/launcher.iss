; Compile only from an audited public payload, never a game/profile folder.
; build-candidate.ps1 supplies PayloadDir, OutputDir and VoltVersion.
[Setup]
AppId={{04C56C5D-0B93-40D0-AB21-7EB427D03242}
AppName=Volt Launcher
AppVersion={#VoltVersion}
DefaultDirName={localappdata}\Programs\Volt Launcher
DefaultGroupName=Volt Launcher
PrivilegesRequired=lowest
ArchitecturesAllowed=x64os
ArchitecturesInstallIn64BitMode=x64os
OutputDir={#OutputDir}
OutputBaseFilename=Volt-Launcher-Windows-x64-Setup
SetupIconFile=volt.ico
UninstallDisplayIcon={app}\Volt Launcher.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Files]
Source: "{#PayloadDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{group}\Volt Launcher"; Filename: "{app}\Volt Launcher.exe"; WorkingDir: "{app}"
