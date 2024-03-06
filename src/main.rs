mod gametheory;

use eframe::{egui, Error};
use egui::mutex::Mutex;
use egui::{Color32, FontData, FontFamily, FontId, Margin, RichText, TextStyle};
use egui_plot::{Line, Plot, PlotPoints};
use gametheory::{prisoners_dillemma_rules, Tournament};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::gametheory::get_new_generation;

// Comes from https://github.com/WINSDK/bite/blob/38ddb5d8f6ee7e46496a2c10d335c2128aceb125/gui/src/panels/source_code.rs#L302
// This was written by Nicolas but sits in a different codebase.
fn show_columns<R>(
    ui: &mut egui::Ui,
    split: f32,
    add_contents: impl FnOnce(&mut egui::Ui, &mut egui::Ui) -> R,
) -> R {
    debug_assert!(split >= 0.0 && split <= 1.0);
    let spacing = ui.spacing().item_spacing.x;
    let total_spacing = spacing * (2 as f32 - 1.0);
    let column_width = ui.available_width() - total_spacing;
    let top_left = ui.cursor().min;

    let (mut left, mut right) = {
        let lpos = top_left;
        let rpos = top_left + egui::vec2(split * (column_width + spacing), 0.0);

        let lrect = egui::Rect::from_min_max(
            lpos,
            egui::pos2(
                lpos.x + column_width * split,
                ui.max_rect().right_bottom().y,
            ),
        );
        let rrect = egui::Rect::from_min_max(
            rpos,
            egui::pos2(
                rpos.x + column_width * (1.0 - split),
                ui.max_rect().right_bottom().y,
            ),
        );

        let mut lcolumn_ui =
            ui.child_ui(lrect, egui::Layout::top_down_justified(egui::Align::LEFT));
        let mut rcolumn_ui =
            ui.child_ui(rrect, egui::Layout::top_down_justified(egui::Align::LEFT));
        lcolumn_ui.set_width(column_width * split);
        rcolumn_ui.set_width(column_width * (1.0 - split));
        (lcolumn_ui, rcolumn_ui)
    };

    let result = add_contents(&mut left, &mut right);

    let mut max_column_width = column_width;
    let mut max_height = 0.0;
    for column in &[left, right] {
        max_column_width = max_column_width.max(column.min_rect().width());
        max_height = column.min_size().y.max(max_height);
    }

    // Make sure we fit everything next frame.
    let total_required_width = total_spacing + max_column_width * 2.0;

    let size = egui::vec2(ui.available_width().max(total_required_width), max_height);
    ui.advance_cursor_after_rect(egui::Rect::from_min_size(top_left, size));
    result
}

struct App {
    ys: Arc<Mutex<Vec<i32>>>,
    simulating: Arc<AtomicBool>,
    n_iters: i32,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let font = FontData::from_static(include_bytes!("../PixelMplus12.ttf"));
        let fonts = egui::FontDefinitions {
            font_data: BTreeMap::from([("pixelmplus".to_string(), font)]),
            families: BTreeMap::from([(FontFamily::Monospace, vec!["pixelmplus".to_string()])]),
        };

        let mut text_styles = BTreeMap::new();
        text_styles.insert(TextStyle::Small, FontId::monospace(9.0));
        text_styles.insert(TextStyle::Body, FontId::monospace(12.5));
        text_styles.insert(TextStyle::Monospace, FontId::monospace(12.0));
        text_styles.insert(TextStyle::Button, FontId::monospace(14.0));
        text_styles.insert(TextStyle::Heading, FontId::monospace(18.0));

        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.style_mut(|s| s.text_styles = text_styles);

        Self {
            ys: Default::default(),
            simulating: Arc::new(AtomicBool::new(false)),
            n_iters: 100,
        }
    }

    fn reset_game(&mut self) {
        self.simulating.store(false, Ordering::Relaxed);
        self.ys.lock().clear();
    }

    fn show_plot(&mut self, ui: &mut egui::Ui) {
        let points = self
            .ys
            .lock()
            .clone()
            .into_iter()
            .zip(0..self.n_iters)
            .map(|(y, x)| [x as f64, y as f64])
            .collect();

        let points = PlotPoints::new(points);
        let price = Line::new(points).color(Color32::LIGHT_BLUE);

        Plot::new("Evolution")
            .x_axis_label("Tournaments")
            .y_axis_label("Score")
            .allow_zoom(false)
            .allow_drag(false)
            .show_x(true)
            .show_y(true)
            .show(ui, |plot_ui| {
                plot_ui.line(price);
            });
    }

    fn show_left(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().spacing.item_spacing.x = 10.0;

        ui.label(RichText::new(format!("#Iterations {}", self.n_iters)).size(14.0));
        ui.add(egui::widgets::Slider::new(&mut self.n_iters, 100..=300).show_value(false));

        if ui.button("Simulate").clicked() {
            let ctx = ui.ctx().clone();
            let xs = self.ys.clone();
            let sim = self.simulating.clone();
            let n_iters = self.n_iters;
            std::thread::spawn(move || simulate(ctx, xs, sim, n_iters));
        }

        if ui.button("Reset").clicked() {
            self.reset_game();
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add(egui::Image::new(egui::include_image!("../felix.png")));
        });
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        show_columns(ui, 0.2, |lui, rui| {
            let margin = Margin {
                right: 5.0,
                ..Default::default()
            };
            egui::Frame::none().inner_margin(margin).show(lui, |lui| {
                self.show_left(lui);
            });

            self.show_plot(rui);
            // self.show_grid(rui);
        });
    }
}

fn simulate(ctx: egui::Context, ys: Arc<Mutex<Vec<i32>>>, sim: Arc<AtomicBool>, n_iters: i32) {
    let mut gen = (0..20).collect::<Vec<u8>>().into_boxed_slice();

    sim.store(true, Ordering::Relaxed);
    ys.lock().clear();

    for _ in 0..n_iters {
        let mut game = Tournament::from(100, prisoners_dillemma_rules, gen);
        game.run();
        let (fittest, mvp_score) = game.select_ten_fittest_and_bestscore();
        let _mvp = &fittest[0];

        if !sim.load(Ordering::Relaxed) {
            return;
        }

        ys.lock().push(mvp_score);
        ctx.request_repaint();

        gen = get_new_generation(fittest);
    }

    sim.store(false, Ordering::Relaxed);
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| self.show(ui));
    }
}

fn main() -> Result<(), Error> {
    eframe::run_native(
        "Game Theory",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        },
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(App::new(cc))
        }),
    )
}
