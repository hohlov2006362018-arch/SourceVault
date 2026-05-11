; Inno Setup script for SourceVault.
;
; Usage:
;   iscc /DAppVersion=0.1.0 /DPayloadDir=..\stage\SourceVault-0.1.0-x64 sourcevault.iss
;
; Required defines:
;   AppVersion  -- semver string, e.g. "0.1.0"
;   PayloadDir  -- folder containing SourceVault.exe + sourcevault.exe + LICENSE.txt

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

#ifndef PayloadDir
  #define PayloadDir "..\stage"
#endif

#define AppName "SourceVault"
#define AppPublisher "SourceVault contributors"
#define AppURL "https://github.com/hohlov2006362018-arch/SourceVault"
#define AppExe "SourceVault.exe"
#define AppExeCli "sourcevault.exe"

[Setup]
AppId={{6B2FB5D2-FF2A-4FF5-B27D-3B16B4E0E2AA}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}/releases
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
LicenseFile={#PayloadDir}\LICENSE.txt
UninstallDisplayName={#AppName} {#AppVersion}
UninstallDisplayIcon={app}\{#AppExe}
OutputBaseFilename=SourceVault-{#AppVersion}-setup-x64
OutputDir=output
SetupIconFile=..\crates\sourcevault-gui\assets\sourcevault.ico
WizardStyle=modern
Compression=lzma2/ultra64
SolidCompression=yes
ArchitecturesAllowed=x64 arm64
ArchitecturesInstallIn64BitMode=x64 arm64
PrivilegesRequired=admin
DisableProgramGroupPage=auto
MinVersion=6.1
ChangesAssociations=yes

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "uk"; MessagesFile: "compiler:Languages\Ukrainian.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "ja"; MessagesFile: "compiler:Languages\Japanese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associate"; Description: "Register file associations and ""Open with SourceVault"" context menu"; GroupDescription: "Integration"; Flags: checkedonce
Name: "path"; Description: "Add the SourceVault CLI to your PATH"; GroupDescription: "Integration"; Flags: unchecked

[Files]
Source: "{#PayloadDir}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\{#AppExeCli}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\README.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{commondesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[Registry]
; Application registration (used by "Open With").
Root: HKLM; Subkey: "Software\Classes\Applications\{#AppExe}"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "{#AppName}"; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\Classes\Applications\{#AppExe}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Flags: uninsdeletekey

; Per-extension ProgID + association.
Root: HKLM; Subkey: "Software\Classes\SourceVault.Archive"; ValueType: string; ValueName: ""; ValueData: "Valve Source archive"; Flags: uninsdeletekey; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\SourceVault.Archive\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#AppExe},0"; Flags: uninsdeletekey; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\SourceVault.Archive\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Flags: uninsdeletekey; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\SourceVault.Archive\shell\extract"; ValueType: string; ValueName: ""; ValueData: "Extract here with SourceVault"; Flags: uninsdeletekey; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\SourceVault.Archive\shell\extract\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeCli}"" extract ""%1"""; Flags: uninsdeletekey; Tasks: associate

Root: HKLM; Subkey: "Software\Classes\.vpk"; ValueType: string; ValueName: ""; ValueData: "SourceVault.Archive"; Flags: uninsdeletevalue; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\.pak"; ValueType: string; ValueName: ""; ValueData: "SourceVault.Archive"; Flags: uninsdeletevalue; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\.gcf"; ValueType: string; ValueName: ""; ValueData: "SourceVault.Archive"; Flags: uninsdeletevalue; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\.sga"; ValueType: string; ValueName: ""; ValueData: "SourceVault.Archive"; Flags: uninsdeletevalue; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\.wad"; ValueType: string; ValueName: ""; ValueData: "SourceVault.Archive"; Flags: uninsdeletevalue; Tasks: associate
Root: HKLM; Subkey: "Software\Classes\.xzp"; ValueType: string; ValueName: ""; ValueData: "SourceVault.Archive"; Flags: uninsdeletevalue; Tasks: associate

; Open-with entries (right-click → Open With → SourceVault) for each extension.
Root: HKLM; Subkey: "Software\Classes\.vpk\OpenWithProgids"; ValueType: string; ValueName: "SourceVault.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKLM; Subkey: "Software\Classes\.pak\OpenWithProgids"; ValueType: string; ValueName: "SourceVault.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKLM; Subkey: "Software\Classes\.gcf\OpenWithProgids"; ValueType: string; ValueName: "SourceVault.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKLM; Subkey: "Software\Classes\.sga\OpenWithProgids"; ValueType: string; ValueName: "SourceVault.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKLM; Subkey: "Software\Classes\.wad\OpenWithProgids"; ValueType: string; ValueName: "SourceVault.Archive"; ValueData: ""; Flags: uninsdeletevalue
Root: HKLM; Subkey: "Software\Classes\.xzp\OpenWithProgids"; ValueType: string; ValueName: "SourceVault.Archive"; ValueData: ""; Flags: uninsdeletevalue

; PATH integration (optional).
Root: HKLM; Subkey: "System\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath('{app}'); Tasks: path; Flags: preservestringtype

[Code]
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE,
    'System\CurrentControlSet\Control\Session Manager\Environment',
    'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;
