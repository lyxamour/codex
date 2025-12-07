; Inno Setup Script for Codex AI
; This script creates a Windows installer for Codex

#define MyAppName "Codex AI"
#define MyAppVersion "0.4.4"
#define MyAppPublisher "Lyxamour"
#define MyAppURL "https://github.com/lyxamour/codex"
#define MyAppExeName "codex.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{01234567-89AB-CDEF-0123-456789ABCDEF}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
;AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\Codex
DefaultGroupName={#MyAppName}
OutputDir={#MyAppName}-Setup
OutputBaseFilename=Codex-Setup-{#MyAppVersion}
Compression=lzma
SolidCompression=yes
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 0,6.1
Name: "addtopath"; Description: "Add Codex to PATH environment variable"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\x86_64-pc-windows-msvc\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Registry]
; Add Codex to PATH environment variable
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "PATH"; ValueData: "{olddata};{app}"; Check: NeedsAddPath({app}); Flags: preservestringtype

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{userdesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: quicklaunchicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{userappdata}\Codex"

[UninstallRun]
Filename: "{app}\uninstall_path.exe"; Parameters: "{app}"; Flags: runhidden

[Code]
{!--
This function checks if the specified directory is already in the PATH environment variable
--}
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, 
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 
    'PATH', OrigPath) 
  then begin
    Result := True;
    exit;
  end;
  { See if the path is already there }
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;

{!--
This function is called when the installer is finished
--}
procedure CurStepChanged(CurStep: TSetupStep);
var
  sPath: string;
  sCmd: string;
begin
  if CurStep = ssPostInstall then
  begin
    if WizardIsTaskSelected('addtopath') then
    begin
      sPath := ExpandConstant('{app}');
      if NeedsAddPath(sPath) then
      begin
        RegWriteStringValue(HKEY_LOCAL_MACHINE, 
          'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 
          'PATH', GetEnvironmentVariable('PATH') + ';' + sPath);
        { Notify all windows of environment changes }
        SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, 0, 
          LPARAM(PChar('Environment')), SMTO_ABORTIFHUNG, 5000, nil);
      end;
    end;
  end;
end;

{!--
This function checks if Codex is already installed
--}
function InitializeSetup: Boolean;
var
  InstalledVersion: string;
  ResultCode: Integer;
begin
  Result := True;
  
  { Check if Codex is already installed }
  if RegQueryStringValue(HKEY_LOCAL_MACHINE, 
    'SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\' + '{#SetupSetting('AppId')}', 
    'DisplayVersion', InstalledVersion) 
  then
  begin
    { Codex is already installed, check version }
    if CompareVersion(InstalledVersion, '{#MyAppVersion}') >= 0 then
    begin
      MsgBox(Format('{#MyAppName} %s is already installed on your computer.\n\nDo you want to continue with the installation?', [InstalledVersion]), mbInformation, MB_YESNO) = IDYES;
    end;
  end;
end;

{!--
This function creates a batch file to remove Codex from PATH during uninstall
--}
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  BatchFile: string;
  BatchContent: string;
  ResultCode: Integer;
begin
  if CurUninstallStep = usUninstall then
  begin
    { Create a batch file to remove Codex from PATH }
    BatchFile := ExpandConstant('{tmp}\uninstall_path.bat');
    BatchContent := 
      '@echo off\n' +
      'setlocal enabledelayedexpansion\n' +
      'set "RegPath=HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment"\n' +
      'for /f "tokens=3*" %%A in (''reg query "!RegPath!" /v PATH'') do set "OldPath=%%B"\n' +
      'set "NewPath=!OldPath:{app};=!"\n' +
      'set "NewPath=!NewPath:;{app}=!"\n' +
      'reg add "!RegPath!" /v PATH /d "!NewPath!" /f >nul\n' +
      'endlocal\n' +
      'exit /b 0';
    
    { Replace {app} with actual path }
    StringChangeEx(BatchContent, '{app}', ExpandConstant('{app}'), True);
    
    { Write the batch file }
    SaveStringToFile(BatchFile, BatchContent, False);
    
    { Run the batch file }
    Exec(BatchFile, '', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  end;
end;
