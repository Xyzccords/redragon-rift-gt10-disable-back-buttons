# Baja los instaladores de ViGEmBus y HidHide que el instalador lleva dentro.
# Comprueba tamaño y SHA-256; si algo no coincide, borra el archivo y se detiene.
$ErrorActionPreference = "Stop"

$destino = Join-Path $PSScriptRoot "..\dependencias"
New-Item -ItemType Directory -Force $destino | Out-Null

$archivos = @(
    @{
        Nombre = "ViGEmBus_1.22.0_x64_x86_arm64.exe"
        Url    = "https://github.com/nefarius/ViGEmBus/releases/download/v1.22.0/ViGEmBus_1.22.0_x64_x86_arm64.exe"
        Bytes  = 6278576
        Sha256 = "89220A7865076B342892F98865F3499FB7C4CFD673159E89D352C360FD014C6A"
    },
    @{
        Nombre = "HidHide_1.5.230_x64.exe"
        Url    = "https://github.com/nefarius/HidHide/releases/download/v1.5.230.0/HidHide_1.5.230_x64.exe"
        Bytes  = 8078016
        Sha256 = "F4BBBCB82E6258641B887C74BC81C4C5F66E4AA811808DFC304347687B7605F6"
    }
)

foreach ($a in $archivos) {
    $ruta = Join-Path $destino $a.Nombre
    if (-not (Test-Path $ruta)) {
        Write-Host "Descargando $($a.Nombre)"
        Invoke-WebRequest -Uri $a.Url -OutFile $ruta -UseBasicParsing
    }
    $tamano = (Get-Item $ruta).Length
    $hash = (Get-FileHash $ruta -Algorithm SHA256).Hash
    if ($tamano -ne $a.Bytes -or $hash -ne $a.Sha256) {
        Remove-Item $ruta
        throw "$($a.Nombre) no coincide con el tamaño o el SHA-256 esperado. Se borró; vuelve a intentarlo."
    }
    Write-Host "OK  $($a.Nombre)"
}
