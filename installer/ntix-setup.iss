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
AppPublisherURL=https://github.com/cristiandis/NTIX
AppSupportURL=https://github.com/cristiandis/NTIX/issues
AppUpdatesURL=https://github.com/cristiandis/NTIX/releases
DefaultDirName={autopf}\NTIX
DefaultGroupName=NTIX
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
Name: "desktopicon"; Description: "Create a &desktop icon"; GroupDescription: "Additional icons:"
Name: "installwinget"; Description: "Install &Winget (Windows Package Manager)"; GroupDescription: "Optional — Package Managers:"; Flags: unchecked
Name: "installchoco"; Description: "Install &Chocolatey"; GroupDescription: "Optional — Package Managers:"; Flags: unchecked
Name: "installscoop"; Description: "Install &Scoop"; GroupDescription: "Optional — Package Managers:"; Flags: unchecked

[Files]
Source: "..\src\NTIX.Cli\bin\Release\net10.0\win-x64\publish\ntix.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\branding\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\NTIX"; Filename: "{app}\ntix.exe"
Name: "{group}\Uninstall NTIX"; Filename: "{uninstallexe}"
Name: "{autodesktop}\NTIX"; Filename: "{app}\ntix.exe"; Tasks: desktopicon

[Run]
; Winget — provisioned via AppX on Windows 10/11, skips if already present
Filename: "powershell.exe"; \
  Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""$ProgressPreference='SilentlyContinue'; Add-AppxPackage -RegisterByFamilyName -MainPackage Microsoft.DesktopAppInstaller_8wekyb3d8bbwe"""; \
  StatusMsg: "Installing Winget..."; \
  Tasks: installwinget; \
  Flags: RunHidden WaitUntilTerminated SkipIfFailed

; Chocolatey — official install script (skips if choco already on PATH)
Filename: "powershell.exe"; \
  Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""if (Get-Command choco -ErrorAction SilentlyContinue) { Write-Host 'Chocolatey already installed.' } else { irm community.chocolatey.org/install.ps1 | iex }"""; \
  StatusMsg: "Installing Chocolatey..."; \
  Tasks: installchoco; \
  Flags: RunHidden WaitUntilTerminated SkipIfFailed

; Scoop — official install script (skips if scoop already on PATH)
Filename: "powershell.exe"; \
  Parameters: "-NoProfile -ExecutionPolicy Bypass -Command ""if (Get-Command scoop -ErrorAction SilentlyContinue) { Write-Host 'Scoop already installed.' } else { irm get.scoop.sh | iex }"""; \
  StatusMsg: "Installing Scoop..."; \
  Tasks: installscoop; \
  Flags: RunHidden WaitUntilTerminated SkipIfFailed

; Open documentation after install
Filename: "https://cristiandis.gitbook.io/ntix"; \
  Description: "&Open documentation"; \
  Flags: nowait postinstall skipifsilent shellexec

; Open config folder after install
Filename: "explorer.exe"; \
  Parameters: "{userprofile}\ntix"; \
  Description: "&Open config folder"; \
  Flags: nowait postinstall skipifsilent unchecked
