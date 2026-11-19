//! Ventana mínima: muestra si el filtro está trabajando y deja pausarlo.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use eframe::egui::{self, Color32, FontFamily, Frame, RichText, Vec2};

use crate::autoinicio;
use crate::puente::{Control, Fase};

const FONDO: Color32 = Color32::from_rgb(0x1b, 0x1d, 0x1a);
const PANEL: Color32 = Color32::from_rgb(0x24, 0x27, 0x22);
const LINEA: Color32 = Color32::from_rgb(0x3a, 0x3e, 0x37);
const TEXTO: Color32 = Color32::from_rgb(0xdc, 0xdf, 0xd6);
const TENUE: Color32 = Color32::from_rgb(0x8d, 0x93, 0x87);
const ACENTO: Color32 = Color32::from_rgb(0xe8, 0xb3, 0x3c);

pub fn configurar(ctx: &egui::Context) {
    let mut fuentes = egui::FontDefinitions::default();
    // Segoe UI viene con Windows, así que la ventana se ve como el resto del sistema.
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fuentes.font_data.insert("segoe".into(), Arc::new(egui::FontData::from_owned(bytes)));
        fuentes.families.entry(FontFamily::Proportional).or_default().insert(0, "segoe".into());
    }
    ctx.set_fonts(fuentes);

    let mut v = egui::Visuals::dark();
    v.override_text_color = Some(TEXTO);
    v.panel_fill = FONDO;
    v.widgets.inactive.bg_fill = PANEL;
    v.widgets.inactive.weak_bg_fill = PANEL;
    v.widgets.hovered.weak_bg_fill = LINEA;
    v.selection.bg_fill = ACENTO.gamma_multiply(0.4);
    ctx.set_visuals(v);
}

pub struct App {
    control: Arc<Control>,
    inicio_con_windows: bool,
    /// Fotogramas que faltan para minimizar: eframe muestra la ventana tras el primero.
    minimizar_en: Option<u8>,
}

impl App {
    pub fn nueva(control: Arc<Control>, minimizado: bool) -> Self {
        Self { control, inicio_con_windows: autoinicio::activado(), minimizar_en: minimizado.then_some(3) }
    }
}

fn fila(ui: &mut egui::Ui, nombre: &str, valor: &str) {
    ui.label(RichText::new(nombre).color(TENUE));
    ui.label(valor);
    ui.end_row();
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // El puente corre en otro hilo: la ventana se refresca sola para mostrar su estado.
        ctx.request_repaint_after(Duration::from_millis(250));
        if let Some(n) = self.minimizar_en {
            self.minimizar_en = n.checked_sub(1);
            if n == 0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let s = self.control.leer();
        let pausado = self.control.pausado.load(Ordering::Relaxed);

        let (titulo, detalle) = if pausado {
            ("En pausa", "El mando funciona como viene de fábrica, con los botones de atrás.")
        } else {
            match s.fase {
                Fase::Activo => ("Botones de atrás anulados", "Los juegos ven un mando de Xbox 360 sin esos botones."),
                Fase::SinMando => ("Esperando el mando", "Conecta el Redragon Rift GT10 por USB."),
                Fase::FaltaViGEm => ("Falta un componente", "No se puede crear el mando virtual."),
                Fase::Pausado | Fase::Iniciando => ("Iniciando", ""),
            }
        };

        egui::CentralPanel::default().frame(Frame::NONE.fill(FONDO).inner_margin(24.0)).show(ui, |ui| {
            ui.label(RichText::new("Filtro de mando").size(13.0).color(TENUE));
            ui.add_space(4.0);
            ui.label(RichText::new(titulo).size(26.0).strong());
            ui.label(RichText::new(detalle).color(TENUE));

            ui.add_space(24.0);
            egui::Grid::new("estado").num_columns(2).spacing([32.0, 8.0]).show(ui, |ui| {
                let conectado = s.fase == Fase::Activo && !pausado;
                fila(ui, "Redragon Rift GT10", if conectado { "Conectado" } else { "No detectado" });
                fila(ui, "Mando original", if s.oculto && !pausado { "Oculto a los juegos" } else { "Visible para los juegos" });
            });

            ui.add_space(24.0);
            let texto = if pausado { "Reanudar" } else { "Pausar" };
            if ui.add(egui::Button::new(texto).min_size(Vec2::new(120.0, 34.0))).clicked() {
                self.control.pausado.store(!pausado, Ordering::Relaxed);
            }

            ui.add_space(16.0);
            if ui.checkbox(&mut self.inicio_con_windows, "Iniciar con Windows").changed() {
                autoinicio::cambiar(self.inicio_con_windows);
            }

            if let Some(aviso) = &s.aviso {
                ui.add_space(16.0);
                ui.label(RichText::new(aviso).color(ACENTO));
            }
        });
    }
}
