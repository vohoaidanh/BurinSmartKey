; Script tao bo cai dat chuan Inno Setup cho Burin Smart Key
#define MyAppName "Burin Smart Key"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Burin Software"
#define MyAppExeName "BurinSmartKey.exe"

[Setup]
AppId={{C7829B63-3F5C-46F2-8742-187515F08BE2}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\BurinSmartKey
DisableDirPage=no
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=D:\Project\smart-vietnamese-ime
OutputBaseFilename=Burin_Smart_Key_Setup
SetupIconFile=D:\Project\smart-vietnamese-ime\burin.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\burin.ico
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "startup"; Description: "Khoi dong cung Windows khi bat may (Start with Windows)"; GroupDescription: "Tuy chon he thong (System options)"

[Files]
Source: "D:\Project\smart-vietnamese-ime\installer\BurinSmartKey.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "D:\Project\smart-vietnamese-ime\burin.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\burin.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"; IconFilename: "{app}\burin.ico"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\burin.ico"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "BurinSmartKey"; ValueData: """{app}\{#MyAppExeName}"""; Flags: uninsdeletevalue; Tasks: startup

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[InstallDelete]
Type: files; Name: "{app}\BurinSmartKey.exe"

[Code]
// Hàm tắt ứng dụng trước khi cài đặt hoặc gỡ cài đặt
function StopBurinProcess(): Boolean;
var
  ResultCode: Integer;
begin
  Exec('taskkill.exe', '/F /IM BurinSmartKey.exe /T', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Sleep(300);
  Result := True;
end;

function InitializeSetup(): Boolean;
begin
  StopBurinProcess();
  Result := True;
end;

function InitializeUninstall(): Boolean;
begin
  StopBurinProcess();
  Result := True;
end;
