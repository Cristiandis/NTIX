; NTIX Inno Setup Script
; https://github.com/Cristiandis/NTIX
;
; Build:
;   iscc /DVersion=1.0.0 ntix-setup.iss
;
; Silent install:
;   ntix-setup.exe /SILENT

#ifndef Version
  #define Version "1.0.0"
#endif

[Setup]
AppId={{B3A7F2E0-8C4D-4E6A-9F1B-2D3E5A7C8B90}}
AppName=NTIX
AppVersion={#Version}
AppPublisher=Cristiandis
AppPublisherURL=https://github.com/Cristiandis/NTIX
AppSupportURL=https://github.com/Cristiandis/NTIX/issues
AppUpdatesURL=https://github.com/Cristiandis/NTIX/releases
DefaultDirName={autopf}\NTIX
DefaultGroupName=NTIX
DisableDirPage=no
LicenseFile=..\LICENSE
SetupIconFile=ntix.ico
UninstallIconFile=ntix.ico
OutputDir=Output
OutputBaseFilename=ntix-setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
WizardStyle=modern
DisableProgramGroupPage=yes
MinVersion=10.0.17763

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "installwinget"; Description: "Install &Winget (Windows Package Manager)"; GroupDescription: "Optional — Package Managers:"; Flags: unchecked
Name: "installchoco"; Description: "Install &Chocolatey"; GroupDescription: "Optional — Package Managers:"; Flags: unchecked
Name: "installscoop"; Description: "Install &Scoop"; GroupDescription: "Optional — Package Managers:"; Flags: unchecked

[Dirs]
; Ensure the config folder exists before we offer to open it post-install
Name: "{%USERPROFILE}\ntix"

[Files]
Source: "..\src\NTIX.Cli\bin\Release\net10.0\win-x64\publish\ntix.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\branding\LICENSE"; DestDir: "{app}"; DestName: "LICENSE-BRANDING.txt"; Flags: ignoreversion

[Icons]
Name: "{group}\NTIX"; Filename: "{app}\ntix.exe"
Name: "{group}\Uninstall NTIX"; Filename: "{uninstallexe}"

[Registry]
; Add {app} to the system PATH so `ntix` is callable from any terminal
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; \
    ValueType: expandsz; ValueName: "Path"; \
    ValueData: "{olddata};{app}"; \
    Check: NeedsAddPath('{app}')

[Run]
; Winget — provisioned via AppX on Windows 10/11, skips if already present
Filename: "powershell.exe"; \
Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""$ProgressPreference='SilentlyContinue'; Add-AppxPackage -RegisterByFamilyName -MainPackage Microsoft.DesktopAppInstaller_8wekyb3d8bbwe"""; \
StatusMsg: "Installing Winget..."; \
Tasks: installwinget; \
Flags: RunHidden WaitUntilTerminated

; Chocolatey — official install script (skips if choco already on PATH)
Filename: "powershell.exe"; \
Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""if (Get-Command choco -ErrorAction SilentlyContinue) {{ Write-Host 'Chocolatey already installed.'; exit 0 }; $ErrorActionPreference='Stop'; [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12; $p = Join-Path $env:TEMP 'choco-install.ps1'; Invoke-WebRequest -UseBasicParsing -Uri 'https://community.chocolatey.org/install.ps1' -OutFile $p; & $p"""; \
StatusMsg: "Installing Chocolatey..."; \
Tasks: installchoco; \
Flags: RunHidden WaitUntilTerminated

; Scoop — official install script (skips if scoop already on PATH)
Filename: "powershell.exe"; \
Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""if (Get-Command scoop -ErrorAction SilentlyContinue) {{ Write-Host 'Scoop already installed.'; exit 0 }; $ErrorActionPreference='Stop'; [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12; $p = Join-Path $env:TEMP 'scoop-install.ps1'; Invoke-WebRequest -UseBasicParsing -Uri 'https://get.scoop.sh' -OutFile $p; & $p -RunAsAdmin"""; \
StatusMsg: "Installing Scoop..."; \
Tasks: installscoop; \
Flags: RunHidden WaitUntilTerminated

; Open documentation after install
Filename: "https://cristiandis.gitbook.io/ntix"; \
Description: "&Open documentation"; \
Flags: nowait postinstall skipifsilent shellexec

; Open config folder after install (folder is created in [Dirs], so this is always safe)
Filename: "{win}\explorer.exe"; \
Parameters: "{%USERPROFILE}\ntix"; \
Description: "&Open config folder"; \
Flags: nowait postinstall skipifsilent unchecked

[Code]
const
  WM_SETTINGCHANGE = $001A;
  SMTO_ABORTIFHUNG = $0002;

function SendMessageTimeout(hWnd: Longint; Msg: Longint; wParam: Longint;
  lParam: string; fuFlags, uTimeout: Longint; var lpdwResult: Longint): Longint;
  external 'SendMessageTimeoutW@user32.dll stdcall';

function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;

procedure BroadcastEnvironmentChange;
var
  ResultCode: Longint;
begin
  { Notify running processes (e.g. Explorer, cmd, PowerShell hosts) that
    environment variables changed, so newly opened terminals pick up the
    updated PATH without requiring a reboot or logoff. }
  SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, 0, 'Environment',
    SMTO_ABORTIFHUNG, 5000, ResultCode);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    BroadcastEnvironmentChange;
  end;
end;
