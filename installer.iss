[Setup]
AppId={{8E3B2A41-6D57-4C8F-9A2E-4F1B7D9C6E30}
AppName=Joinus
AppVersion=0.4.0
AppVerName=Joinus 0.4.0
AppPublisher=Johanes-mg
AppPublisherURL=https://johanes-mg.github.io
AppSupportURL=https://johanes-mg.github.io
AppUpdatesURL=https://johanes-mg.github.io
DefaultDirName={autopf}\Joinus
DefaultGroupName=Joinus
DisableProgramGroupPage=yes
OutputDir=Installeur
OutputBaseFilename=JoinusSetup
SetupIconFile=images\logo.ico
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
PrivilegesRequired=admin
UninstallDisplayIcon={app}\joinus.exe
DisableDirPage=no
DisableReadyPage=no
UsePreviousAppDir=yes
AllowUNCPath=no
ChangesEnvironment=no
VersionInfoVersion=0.4.0
VersionInfoCompany=Johanes-mg
VersionInfoDescription=Joinus 0.4.0 - Installation
VersionInfoProductName=Joinus
VersionInfoProductVersion=0.4.0

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\x86_64-pc-windows-gnullvm\release\joinus.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "images\*.png"; DestDir: "{app}\images"; Flags: ignoreversion
Source: "images\*.ico"; DestDir: "{app}\images"; Flags: ignoreversion

[Icons]
Name: "{group}\Joinus"; Filename: "{app}\joinus.exe"; IconFilename: "{app}\joinus.exe"; IconIndex: 0; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,Joinus}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Joinus"; Filename: "{app}\joinus.exe"; Tasks: desktopicon; IconFilename: "{app}\joinus.exe"; IconIndex: 0; WorkingDir: "{app}"

[Run]
Filename: "{app}\joinus.exe"; Description: "{cm:LaunchProgram,Joinus}"; Flags: nowait postinstall skipifsilent

[Code]
const
  REGLE_ENTRANTE_PROG = 'Joinus (programme entrant)';
  REGLE_ENTRANTE_UDP = 'Joinus (UDP 42001 entrant)';

function ExecNetsh(Params: String): Boolean;
var
  Code: Integer;
begin
  Result := Exec(ExpandConstant('{sys}\netsh.exe'), Params, '', SW_HIDE, ewWaitUntilTerminated, Code) and (Code = 0);
end;

procedure AjouterReglesPareFeu();
var
  CheminExe: String;
begin
  CheminExe := ExpandConstant('{app}\joinus.exe');
  ExecNetsh('advfirewall firewall delete rule name="' + REGLE_ENTRANTE_PROG + '"');
  ExecNetsh('advfirewall firewall delete rule name="' + REGLE_ENTRANTE_UDP + '"');
  ExecNetsh('advfirewall firewall add rule name="' + REGLE_ENTRANTE_PROG + '" dir=in action=allow program="' + CheminExe + '" enable=yes profile=any');
  ExecNetsh('advfirewall firewall add rule name="' + REGLE_ENTRANTE_UDP + '" dir=in action=allow protocol=UDP localport=42001 enable=yes profile=any');
end;

procedure SupprimerReglesPareFeu();
begin
  ExecNetsh('advfirewall firewall delete rule name="' + REGLE_ENTRANTE_PROG + '"');
  ExecNetsh('advfirewall firewall delete rule name="' + REGLE_ENTRANTE_UDP + '"');
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    AjouterReglesPareFeu();
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Reponse: Integer;
  DossierDonnees: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    SupprimerReglesPareFeu();
    DossierDonnees := ExpandConstant('{userappdata}\Joinus');
    if DirExists(DossierDonnees) then
    begin
      Reponse := MsgBox(
        'Voulez-vous également supprimer vos données Joinus ?' + #13#10 + #13#10 +
        'Dossier : ' + DossierDonnees + #13#10 + #13#10 +
        'Si vous répondez Non, vos données (profil, avatar, session) seront conservées.',
        mbConfirmation, MB_YESNO);

      if Reponse = IDYES then
      begin
        DelTree(DossierDonnees, True, True, True);
      end;
    end;
  end;
end;
