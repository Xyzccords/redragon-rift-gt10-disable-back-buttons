//! Oculta el mando físico a los demás programas con HidHide mientras el puente corre.
//! Al soltar el valor `Oculto` (o al cerrar con Ctrl+C) el mando vuelve a verse.

use std::path::{Path, PathBuf};
use std::process::Command;

const CARPETA_CLI: &str = r"Nefarius Software Solutions\HidHide\x64\HidHideCLI.exe";

pub struct Oculto {
    cli: PathBuf,
    instancia: String,
    exe: String,
    cloak_estaba_activo: bool,
}

fn buscar_cli() -> Result<PathBuf, String> {
    let base = std::env::var_os("ProgramFiles").ok_or("no se encontró la carpeta Program Files")?;
    let ruta = PathBuf::from(base).join(CARPETA_CLI);
    if ruta.exists() {
        Ok(ruta)
    } else {
        Err("HidHide no está instalado".into())
    }
}

fn ejecutar(cli: &Path, args: &[&str]) -> Result<String, String> {
    let salida = Command::new(cli).args(args).output().map_err(|e| e.to_string())?;
    if !salida.status.success() {
        return Err(format!("HidHideCLI {args:?}: {}", String::from_utf8_lossy(&salida.stderr)));
    }
    Ok(String::from_utf8_lossy(&salida.stdout).into_owned())
}

/// De `\\?\hid#vid_2563&pid_0575#7&13bc8e8b&0&0000#{guid}` a `HID\VID_2563&PID_0575\7&13BC8E8B&0&0000`.
fn instancia_desde_ruta(ruta: &str) -> Option<String> {
    let sin_prefijo = ruta.strip_prefix(r"\\?\")?;
    let (sin_guid, _) = sin_prefijo.rsplit_once("#{")?;
    Some(sin_guid.replace('#', r"\").to_uppercase())
}

impl Oculto {
    pub fn activar(ruta_hid: &str) -> Result<Self, String> {
        let cli = buscar_cli()?;
        let instancia = instancia_desde_ruta(ruta_hid).ok_or("ruta del mando con formato inesperado")?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?.display().to_string();
        let cloak_estaba_activo = ejecutar(&cli, &["--cloak-state"])?.contains("--cloak-on");
        ejecutar(&cli, &["--app-reg", &exe, "--dev-hide", &instancia, "--cloak-on"])?;
        Ok(Self { cli, instancia, exe, cloak_estaba_activo })
    }
}

impl Drop for Oculto {
    fn drop(&mut self) {
        let mut args = vec!["--dev-unhide", &self.instancia, "--app-unreg", &self.exe];
        if !self.cloak_estaba_activo {
            args.push("--cloak-off");
        }
        if let Err(e) = ejecutar(&self.cli, &args) {
            eprintln!("No se pudo volver a mostrar el mando: {e}");
        }
    }
}

/// Muestra de nuevo todo mando de este modelo que haya quedado oculto (por ejemplo,
/// si el puente se cerró a la fuerza).
pub fn liberar(vid_pid: &str) -> Result<(), String> {
    let cli = buscar_cli()?;
    let ocultos = ejecutar(&cli, &["--dev-list"])?;
    let mut hubo = false;
    for linea in ocultos.lines() {
        let Some((_, resto)) = linea.split_once('"') else { continue };
        let Some((instancia, _)) = resto.split_once('"') else { continue };
        if instancia.to_uppercase().contains(vid_pid) {
            ejecutar(&cli, &["--dev-unhide", instancia])?;
            hubo = true;
        }
    }
    // Esta copia ya no necesita ver mandos ocultos (importa al desinstalar o si cambia de carpeta).
    if let Ok(exe) = std::env::current_exe() {
        ejecutar(&cli, &["--app-unreg", &exe.display().to_string()])?;
    }
    // Si no queda ningún mando oculto, se apaga el ocultado; si hay otros, se respetan.
    if !ejecutar(&cli, &["--dev-list"])?.contains("--dev-hide") {
        ejecutar(&cli, &["--cloak-off"])?;
    }
    println!("{}", if hubo { "Mando mostrado de nuevo." } else { "No había ningún mando oculto." });
    Ok(())
}
