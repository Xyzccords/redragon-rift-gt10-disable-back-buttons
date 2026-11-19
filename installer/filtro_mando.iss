; Instalador de "Filtro de mando". Se compila con: ISCC.exe filtro_mando.iss
; Instala el programa y, si faltan, ViGEmBus (mando virtual) y HidHide (ocultar el mando original).

#define NombreApp "Filtro de mando"
#define Version "0.1.0"
#define Exe "filtro_mando.exe"

[Setup]
AppId={{B4F0C6A2-6C1E-4B7B-9A5E-3D2F8E7A1C90}
AppName={#NombreApp}
AppVersion={#Version}
DefaultDirName={autopf}\{#NombreApp}
DefaultGroupName={#NombreApp}
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\dist
OutputBaseFilename=FiltroMando_Setup_{#Version}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
UninstallDisplayName={#NombreApp}
UninstallDisplayIcon={app}\{#Exe}
CloseApplications=yes
RestartApplications=no
; El arranque con Windows se guarda en HKCU: vale cuando quien instala es el mismo usuario que acepta el aviso de administrador.
UsedUserAreasWarning=no

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "inicio"; Description: "Iniciar con Windows"; GroupDescription: "Opciones:"

[Files]
Source: "..\target\release\{#Exe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\licencias-terceros\LICENCIA_HidHide.txt"; DestDir: "{app}\licencias"; Flags: ignoreversion
Source: "..\licencias-terceros\LICENCIA_ViGEmBus.txt"; DestDir: "{app}\licencias"; Flags: ignoreversion
Source: "..\dependencias\ViGEmBus_1.22.0_x64_x86_arm64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: DebeInstalarViGEm
Source: "..\dependencias\HidHide_1.5.230_x64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: DebeInstalarHidHide

[Icons]
Name: "{group}\{#NombreApp}"; Filename: "{app}\{#Exe}"
Name: "{group}\Mostrar el mando original (si quedó oculto)"; Filename: "{app}\{#Exe}"; Parameters: "--liberar"
Name: "{group}\Desinstalar {#NombreApp}"; Filename: "{uninstallexe}"

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "FiltroMando"; ValueData: """{app}\{#Exe}"" --minimizado"; Flags: uninsdeletevalue; Tasks: inicio

[Run]
Filename: "{tmp}\ViGEmBus_1.22.0_x64_x86_arm64.exe"; Parameters: "/quiet /norestart"; StatusMsg: "Instalando ViGEmBus (mando virtual)..."; Check: DebeInstalarViGEm
Filename: "{tmp}\HidHide_1.5.230_x64.exe"; Parameters: "/quiet /norestart"; StatusMsg: "Instalando HidHide (ocultar el mando original)..."; Check: DebeInstalarHidHide
Filename: "{app}\{#Exe}"; Description: "Abrir {#NombreApp}"; Flags: nowait postinstall skipifsilent runasoriginaluser

[Code]
var
  FaltabaViGEm: Boolean;
  FaltabaHidHide: Boolean;

function ControladorInstalado(Servicio: String): Boolean;
begin
  Result := RegKeyExists(HKLM, 'SYSTEM\CurrentControlSet\Services\' + Servicio);
end;

{ Cierra el programa si está abierto. Primero con calma, para que devuelva solo el mando original, y si
  sigue abierto después de esperar, a la fuerza (el desinstalador libera el mando después). }
procedure CerrarPrograma();
var
  Codigo: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/IM {#Exe}', '', SW_HIDE, ewWaitUntilTerminated, Codigo);
  Sleep(2500);
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM {#Exe}', '', SW_HIDE, ewWaitUntilTerminated, Codigo);
  Sleep(500);
end;

function InitializeSetup(): Boolean;
begin
  FaltabaViGEm := not ControladorInstalado('ViGEmBus');
  FaltabaHidHide := not ControladorInstalado('HidHide');
  CerrarPrograma();
  Result := True;
end;

function InitializeUninstall(): Boolean;
var
  Codigo: Integer;
begin
  CerrarPrograma();
  { Por si el mando hubiera quedado oculto (por ejemplo, si el programa se cerró a la fuerza). }
  Exec(ExpandConstant('{app}\{#Exe}'), '--liberar', '', SW_HIDE, ewWaitUntilTerminated, Codigo);
  Result := True;
end;

function DebeInstalarViGEm(): Boolean;
begin
  Result := FaltabaViGEm;
end;

function DebeInstalarHidHide(): Boolean;
begin
  Result := FaltabaHidHide;
end;

{ HidHide solo oculta mandos después de reiniciar el equipo la primera vez que se instala. }
function NeedRestart(): Boolean;
begin
  Result := FaltabaHidHide;
end;
