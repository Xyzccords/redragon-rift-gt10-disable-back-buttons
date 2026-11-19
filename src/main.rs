//! Filtro de mando para el Redragon Rift GT10: anula los 4 botones traseros.
//! Lee el informe crudo del mando, quita las señales de los traseros y entrega el resto a un
//! mando virtual de Xbox 360. El mando original se oculta a los juegos mientras el filtro corre.
//!
//! Uso: filtro_mando [--minimizado] [--sin-ventana] [--ver] [--liberar] [segundos]
//!   --minimizado   abre la ventana minimizada (es como arranca con Windows)
//!   --sin-ventana  corre en la consola, sin ventana
//!   --ver          solo muestra lo que se pulsa, sin mando virtual ni ocultar el físico
//!   --liberar      vuelve a mostrar el mando original si el filtro se cerró a la fuerza
//!   segundos       solo con --ver o --sin-ventana: se detiene solo

#![windows_subsystem = "windows"]

mod autoinicio;
mod informe;
mod ocultar;
mod puente;
mod ui;

use std::error::Error;
use std::fs::OpenOptions;
use std::os::windows::fs::OpenOptionsExt;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use eframe::egui;

use puente::{Control, Modo};

/// Impide abrir dos copias a la vez: se queda con un archivo bloqueado hasta que el programa termine.
fn unica_copia() -> Option<std::fs::File> {
    let carpeta = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join("FiltroMando");
    std::fs::create_dir_all(&carpeta).ok()?;
    OpenOptions::new().write(true).create(true).share_mode(0).open(carpeta.join("copia.lock")).ok()
}

/// Con `windows_subsystem = "windows"` el programa nace sin consola; se conecta a la que lo lanzó.
fn conectar_consola() {
    unsafe extern "system" {
        fn AttachConsole(proceso: u32) -> i32;
    }
    unsafe { AttachConsole(u32::MAX) };
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tiene = |a: &str| args.iter().any(|x| x == a);
    let limite = args.iter().find_map(|a| a.parse::<u64>().ok()).map(Duration::from_secs);

    if tiene("--liberar") || tiene("--ver") || tiene("--sin-ventana") {
        conectar_consola();
    }
    if tiene("--liberar") {
        return Ok(ocultar::liberar(&format!("VID_{:04X}&PID_{:04X}", informe::VID, informe::PID))?);
    }

    let Some(_candado) = unica_copia() else {
        eprintln!("El filtro ya está abierto.");
        return Ok(());
    };
    let control = Arc::new(Control::nuevo());

    if tiene("--ver") || tiene("--sin-ventana") {
        let aviso = control.clone();
        ctrlc::set_handler(move || aviso.parar.store(true, Ordering::Relaxed))?;
        let modo = if tiene("--ver") { Modo::Ver } else { Modo::Puente };
        puente::ejecutar(&control, modo, limite);
        return Ok(());
    }

    let hilo = {
        let control = control.clone();
        std::thread::spawn(move || puente::ejecutar(&control, Modo::Puente, None))
    };

    let ventana = egui::ViewportBuilder::default()
        .with_title("Filtro de mando")
        .with_inner_size([460.0, 330.0])
        .with_resizable(false);
    let resultado = eframe::run_native(
        "Filtro de mando",
        eframe::NativeOptions { viewport: ventana, ..Default::default() },
        Box::new({
            let control = control.clone();
            let minimizado = tiene("--minimizado");
            move |cc| {
                ui::configurar(&cc.egui_ctx);
                Ok(Box::new(ui::App::nueva(control, minimizado)))
            }
        }),
    );

    // Al cerrar la ventana se detiene el puente y se espera a que devuelva el mando original.
    control.parar.store(true, Ordering::Relaxed);
    let _ = hilo.join();
    resultado.map_err(|e| e.to_string().into())
}
