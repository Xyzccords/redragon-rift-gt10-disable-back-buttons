//! El puente: lee el mando, quita los traseros y entrega el resto a un mando virtual de Xbox 360.
//! Se reconecta solo si el mando se desenchufa y se puede pausar sin cerrar el programa.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

use hidapi::HidApi;
use vigem_client::{Client, TargetId, Xbox360Wired};

use crate::informe::{LARGO_MINIMO, PID, VID, a_xbox, describir, leer_estado};
use crate::ocultar::Oculto;

#[derive(Clone, Copy, PartialEq)]
pub enum Fase {
    Iniciando,
    SinMando,
    Activo,
    Pausado,
    FaltaViGEm,
}

#[derive(Clone)]
pub struct Situacion {
    pub fase: Fase,
    pub oculto: bool,
    /// Aviso para el usuario: qué pasó y cómo arreglarlo.
    pub aviso: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Modo {
    /// Crea el mando virtual y oculta el físico.
    Puente,
    /// Solo imprime lo que se pulsa.
    Ver,
}

pub struct Control {
    pub parar: AtomicBool,
    pub pausado: AtomicBool,
    pub situacion: Mutex<Situacion>,
}

impl Control {
    pub fn nuevo() -> Self {
        Self {
            parar: AtomicBool::new(false),
            pausado: AtomicBool::new(false),
            situacion: Mutex::new(Situacion {
                fase: Fase::Iniciando,
                oculto: false,
                aviso: None,
            }),
        }
    }

    fn poner(&self, f: impl FnOnce(&mut Situacion)) {
        f(&mut self.situacion.lock().unwrap());
    }

    pub fn leer(&self) -> Situacion {
        self.situacion.lock().unwrap().clone()
    }

    fn detenido(&self) -> bool {
        self.parar.load(Ordering::Relaxed) || self.pausado.load(Ordering::Relaxed)
    }
}

enum Fin {
    Detenido,
    SinMando,
}

/// Corre hasta que `c.parar` se activa o pasa `limite`.
pub fn ejecutar(c: &Control, modo: Modo, limite: Option<Duration>) {
    let inicio = Instant::now();
    let vence = || limite.is_some_and(|l| inicio.elapsed() >= l);
    while !c.parar.load(Ordering::Relaxed) && !vence() {
        if c.pausado.load(Ordering::Relaxed) {
            c.poner(|s| {
                s.fase = Fase::Pausado;
                s.oculto = false;
            });
            sleep(Duration::from_millis(200));
            continue;
        }
        match sesion(c, modo, &vence) {
            Ok(Fin::SinMando) => {
                c.poner(|s| {
                    s.fase = Fase::SinMando;
                    s.oculto = false;
                });
                sleep(Duration::from_millis(1000));
            }
            Ok(Fin::Detenido) => {}
            Err(aviso) => {
                c.poner(|s| {
                    s.aviso = Some(aviso);
                    s.oculto = false;
                });
                sleep(Duration::from_millis(2000));
            }
        }
    }
}

/// Una sesión con el mando enchufado. Al terminar se sueltan el mando virtual y el ocultado.
fn sesion(c: &Control, modo: Modo, vence: &dyn Fn() -> bool) -> Result<Fin, String> {
    let api = HidApi::new().map_err(|e| format!("No se pudo leer los dispositivos: {e}"))?;
    let Some(info) = api.device_list().find(|d| d.vendor_id() == VID && d.product_id() == PID) else {
        return Ok(Fin::SinMando);
    };
    let ruta = info.path().to_string_lossy().into_owned();

    let mut virtual_ = None;
    if modo == Modo::Puente {
        let cliente = Client::connect().map_err(|_| {
            c.poner(|s| s.fase = Fase::FaltaViGEm);
            "Falta ViGEmBus, el componente que crea el mando virtual. Instálalo y vuelve a abrir el programa.".to_string()
        })?;
        let mut v = Xbox360Wired::new(cliente, TargetId::XBOX360_WIRED);
        v.plugin().map_err(|e| format!("No se pudo crear el mando virtual: {e}"))?;
        v.wait_ready().map_err(|e| format!("El mando virtual no respondió: {e}"))?;
        virtual_ = Some(v);
    }

    // Se declara antes que `mando` para que se suelte después y el mando vuelva a verse al final.
    let mut aviso = None;
    let oculto = if modo == Modo::Puente {
        match Oculto::activar(&ruta) {
            Ok(o) => Some(o),
            Err(e) => {
                aviso = Some(format!("Los juegos verán también el mando original, con los traseros activos: {e}"));
                None
            }
        }
    } else {
        None
    };
    let mando = info.open_device(&api).map_err(|e| format!("No se pudo abrir el mando: {e}"))?;

    c.poner(|s| {
        s.fase = Fase::Activo;
        s.oculto = oculto.is_some();
        s.aviso = aviso;
    });

    let inicio = Instant::now();
    let mut previo = None;
    let mut buf = [0u8; 64];
    while !c.detenido() && !vence() {
        let n = match mando.read_timeout(&mut buf, 100) {
            Ok(n) => n,
            Err(_) => return Ok(Fin::SinMando), // se desenchufó
        };
        if n < LARGO_MINIMO {
            continue;
        }
        let estado = leer_estado(&buf[..n]);
        if let Some(v) = virtual_.as_mut() {
            if v.update(&a_xbox(&estado)).is_err() {
                return Err("Se perdió el mando virtual.".into());
            }
        }
        if modo == Modo::Ver && previo.is_none_or(|p: crate::informe::Estado| p.pulsaciones() != estado.pulsaciones()) {
            println!("{:5.1}s  {}", inicio.elapsed().as_secs_f32(), describir(&estado));
        }
        previo = Some(estado);
    }
    Ok(Fin::Detenido)
}
