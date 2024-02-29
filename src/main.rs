use gametheory::Error;

use eframe::egui;
use egui::{FontData, FontFamily, FontId, TextStyle};
use std::collections::BTreeMap;

struct App {
    show_viewport: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let font = FontData::from_static(include_bytes!("../PixelMplus12.ttf"));
        let fonts = egui::FontDefinitions {
            font_data: BTreeMap::from([("pixelmplus".to_string(), font)]),
            families: BTreeMap::from([(FontFamily::Monospace, vec!["pixelmplus".to_string()])]),
        };
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.style_mut(|s| {
            s.text_styles = BTreeMap::from([
                (
                    TextStyle::Small,
                    FontId {
                        size: 9.0,
                        family: FontFamily::Monospace,
                    },
                ),
                (
                    TextStyle::Body,
                    FontId {
                        size: 12.5,
                        family: FontFamily::Monospace,
                    },
                ),
                (
                    TextStyle::Monospace,
                    FontId {
                        size: 12.0,
                        family: FontFamily::Monospace,
                    },
                ),
                (
                    TextStyle::Button,
                    FontId {
                        size: 20.0,
                        family: FontFamily::Monospace,
                    },
                ),
                (
                    TextStyle::Heading,
                    FontId {
                        size: 18.0,
                        family: FontFamily::Monospace,
                    },
                ),
            ]);
        });

        Self {
            show_viewport: false
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hello World!");

        if ui.button("run").clicked() {
            self.show_viewport = true;
        }

        if self.show_viewport {
            ui.ctx().show_viewport_immediate(
                egui::ViewportId::from_hash_of("immediate_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("Immediate Viewport")
                    .with_inner_size([200.0, 100.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Hello from immediate viewport");
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        // Tell parent viewport that we should not show next frame:
                        self.show_viewport = false;
                    }
                }
            );
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| self.show(ui));
    }
}

fn main() -> Result<(), Error> {
    // let vr = gametheory::tit_for_tat(None, None);

    eframe::run_native(
        "Game Theory",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        },
        Box::new(move |cc| Box::new(App::new(cc))),
    )?;

    Ok(())
}
