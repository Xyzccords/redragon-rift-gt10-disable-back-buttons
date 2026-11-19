//! Cómo está armado el informe del Redragon Rift GT10 y cómo pasarlo a un mando de Xbox 360.

use vigem_client::{XButtons, XGamepad};

pub const VID: u16 = 0x2563;
pub const PID: u16 = 0x0575;

// Posiciones dentro del informe, medidas con el mando real el 20/11/2026.
// hidapi entrega el informe sin el byte de identificador.
const BOTONES_BAJOS: usize = 0; // botones 1 a 8, un bit cada uno
const BOTONES_ALTOS: usize = 1; // botones 9 a 16
const CRUCETA_Y_TRASEROS: usize = 2; // nibble bajo: cruceta (8 = reposo); nibble alto: traseros
const PALOS: usize = 3; // LX, LY, RX, RY, centrados en 0x80
pub const LARGO_MINIMO: usize = 7;

/// Cada trasero activa el bit del botón delantero que copia más un bit propio.
/// (bit propio en CRUCETA_Y_TRASEROS, botón delantero que arrastra)
const TRASEROS: [(u8, u16); 4] = [(0x80, 1 << 0), (0x20, 1 << 1), (0x10, 1 << 2), (0x40, 1 << 3)];

/// (bit del botón en el mando, botón de Xbox, nombre)
const BOTONES: [(u16, u16, &str); 11] = [
    (1 << 0, XButtons::Y, "Y"), // el de arriba
    (1 << 1, XButtons::B, "B"), // el de la derecha
    (1 << 2, XButtons::A, "A"), // el de abajo
    (1 << 3, XButtons::X, "X"), // el de la izquierda
    (1 << 4, XButtons::LB, "L1"),
    (1 << 5, XButtons::RB, "R1"),
    (1 << 8, XButtons::BACK, "Select"),
    (1 << 9, XButtons::START, "Start"),
    (1 << 10, XButtons::LTHUMB, "L3"),
    (1 << 11, XButtons::RTHUMB, "R3"),
    (1 << 12, XButtons::GUIDE, "Centro"),
];
const L2: u16 = 1 << 6;
const R2: u16 = 1 << 7;

const CRUCETA: [(&str, u16); 8] = [
    ("arriba", XButtons::UP),
    ("arriba derecha", XButtons::UP | XButtons::RIGHT),
    ("derecha", XButtons::RIGHT),
    ("abajo derecha", XButtons::DOWN | XButtons::RIGHT),
    ("abajo", XButtons::DOWN),
    ("abajo izquierda", XButtons::DOWN | XButtons::LEFT),
    ("izquierda", XButtons::LEFT),
    ("arriba izquierda", XButtons::UP | XButtons::LEFT),
];

#[derive(PartialEq, Clone, Copy)]
pub struct Estado {
    botones: u16, // botones ya sin las señales de los traseros
    traseros: u8, // bits de traseros detectados y descartados
    cruceta: Option<usize>,
    palos: [u8; 4],
}

impl Estado {
    /// Lo que cambia cuando el usuario pulsa algo (los palos se mueven siempre un poco).
    pub fn pulsaciones(&self) -> (u16, u8, Option<usize>) {
        (self.botones, self.traseros, self.cruceta)
    }
}

pub fn leer_estado(d: &[u8]) -> Estado {
    let mut botones = u16::from(d[BOTONES_BAJOS]) | u16::from(d[BOTONES_ALTOS]) << 8;
    let traseros = d[CRUCETA_Y_TRASEROS] & 0xF0;
    for (bit, delantero) in TRASEROS {
        if traseros & bit != 0 {
            botones &= !delantero;
        }
    }
    let cruceta = Some(usize::from(d[CRUCETA_Y_TRASEROS] & 0x0F)).filter(|&n| n < 8);
    let palos = [d[PALOS], d[PALOS + 1], d[PALOS + 2], d[PALOS + 3]];
    Estado { botones, traseros, cruceta, palos }
}

fn palo(v: u8) -> i16 {
    ((i32::from(v) - 128) * 256).clamp(-32767, 32767) as i16
}

pub fn a_xbox(e: &Estado) -> XGamepad {
    let mut raw = 0;
    for (bit, xbox, _) in BOTONES {
        if e.botones & bit != 0 {
            raw |= xbox;
        }
    }
    if let Some(n) = e.cruceta {
        raw |= CRUCETA[n].1;
    }
    XGamepad {
        buttons: XButtons { raw },
        left_trigger: if e.botones & L2 != 0 { 255 } else { 0 },
        right_trigger: if e.botones & R2 != 0 { 255 } else { 0 },
        thumb_lx: palo(e.palos[0]),
        thumb_ly: -palo(e.palos[1]),
        thumb_rx: palo(e.palos[2]),
        thumb_ry: -palo(e.palos[3]),
    }
}

pub fn describir(e: &Estado) -> String {
    let mut partes: Vec<&str> = BOTONES
        .iter()
        .filter(|(bit, _, _)| e.botones & bit != 0)
        .map(|(_, _, nombre)| *nombre)
        .collect();
    if e.botones & L2 != 0 {
        partes.push("L2");
    }
    if e.botones & R2 != 0 {
        partes.push("R2");
    }
    let cruceta = e.cruceta.map_or("", |n| CRUCETA[n].0);
    format!(
        "botones: [{}]  cruceta: {}  traseros descartados: {:#04x}  palos: {:?}",
        partes.join(" "),
        cruceta,
        e.traseros,
        e.palos
    )
}
