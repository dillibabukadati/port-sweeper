; Port Sweeper Windows installer - installs GUI and CLI and adds to PATH
#define MyAppName "Port Sweeper"
#define MyAppVersion "1.0.3"
#define MyAppPublisher "Dilli Babu Kadati"
#define MyAppURL "https://github.com/dillibabukadati/port-sweeper"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=.
OutputBaseFilename=Port-Sweeper-Setup-x86_64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "envpath"; Description: "Add Port Sweeper to PATH (recommended)"; GroupDescription: "Additional tasks:"; Flags: checkablealone

[Files]
Source: "psweep.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "port-sweeper.exe"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Add install dir to system PATH when "Add to PATH" task is selected
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: envpath; Check: EnvPathNeedsAdd

[Code]
function EnvPathNeedsAdd(): Boolean;
var
  Paths: String;
  AppDir: String;
begin
  AppDir := ExpandConstant('{app}');
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', Paths) then
    Paths := '';
  Result := Pos(';' + Uppercase(AppDir) + ';', ';' + Uppercase(Paths) + ';') = 0;
end;

[UninstallDelete]
Type: dirifempty; Name: "{app}"

[UninstallRun]
; Remove from PATH on uninstall (simplified: we'd need EnvRemovePath logic for full cleanup)
; User can re-run installer and uncheck "Add to PATH" then uninstall if needed.
