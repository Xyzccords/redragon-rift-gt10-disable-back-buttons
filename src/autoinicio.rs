//! Arranque con Windows: una entrada en la clave Run del usuario, sin permisos de administrador.

use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

const CLAVE: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const NOMBRE: &str = "FiltroMando";

pub fn activado() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(CLAVE)
        .and_then(|k| k.get_value::<String, _>(NOMBRE))
        .is_ok()
}

pub fn cambiar(activar: bool) {
    let Ok((clave, _)) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(CLAVE) else { return };
    if activar {
        if let Ok(exe) = std::env::current_exe() {
            let _ = clave.set_value(NOMBRE, &format!("\"{}\" --minimizado", exe.display()));
        }
    } else {
        let _ = clave.delete_value(NOMBRE);
    }
}
