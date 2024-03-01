use gametheory::*;

use eframe::egui;
use egui::{Color32, FontData, FontFamily, FontId, TextStyle};
use grid::Grid;
use std::collections::{BTreeMap, HashMap};

struct Player {
    // stores own previous move towards players keyed by a String, values initialised to None
    prev_move_self: HashMap<String, Option<Decision>>,
    // stores other players decisions towards self, same storage
    prev_move_other: HashMap<String, Option<Decision>>,
    // function pointer to strategy
    strategy: DecisionTable,
    // name of used player strategy
    strategy_name: String
}

struct Tournament {
    players: Vec<Player>,
    scores: Grid<(i32, i32)>,
    max_iter: u32,
    current_iter: u32,
    rewardsystem: RewardFunc
}

struct App {
    show_viewport: bool,
    grid_size: usize,
    grid: Grid<u8>,
}

type RewardFunc = fn (Decision, Decision) -> i32;

impl Tournament {
    fn initialise_from(n_iter: u32, rules: RewardFunc) -> Self {
        let score_grid = Grid::from_vec(vec![(0, 0); 100], 10);
        let player_init_data: [(&str, DecisionTable); 10] = 
            [("trusting tit for tat", good_tit_for_tat),
            ("suspicious tit for tat", sus_tit_for_tat),
            ("naive", naive),
            ("evil", evil),
            ("random", random),
            ("xor logic", xor),
            ("opposite tit for tat", opposite_tit_for_tat),
            ("xnor logic", xnor),
            ("nand lodic", nand),
            ("Bernoulli uncooperative", random_biased)];
        let mut players_lst = Vec::new();
        for (name, table) in player_init_data {
            let mut initial_player_memory: HashMap<String, Option<Decision>> = HashMap::new();
            for (opponent_name, _) in player_init_data {
                initial_player_memory.insert(opponent_name.to_owned(), None);
            }
            let memory_of_opponents = initial_player_memory.clone();
            let p = Player {
                prev_move_self: initial_player_memory,
                prev_move_other: memory_of_opponents,
                strategy: table,
                strategy_name: name.to_owned()
            };
            players_lst.push(p);
        }
        Tournament {
            players: players_lst,
            scores: score_grid,
            max_iter: n_iter,
            current_iter: 0,
            rewardsystem: rules
        }
    }
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
        text_styles.insert(TextStyle::Button, FontId::monospace(20.0));
        text_styles.insert(TextStyle::Heading, FontId::monospace(18.0));

        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.style_mut(|s| s.text_styles = text_styles);

        let grid_size = 1;
        Self {
            show_viewport: false,
            grid_size,
            grid: Grid::new(grid_size, grid_size),
        }
    }

    fn show_grid(&mut self, ui: &mut egui::Ui) {
        const GRID_LINE_WIDTH: f32 = 4.;

        let rows = self.grid.rows();
        let cols = self.grid.cols();
        let grid_width = ui.available_width();
        let grid_height = ui.available_height();
        let cell_width = grid_width / cols as f32;
        let cell_height = grid_height / rows as f32;

        // Allocate the total space based on the available size.
        let rect = ui.allocate_space(egui::vec2(grid_width, grid_height)).1;

        let painter = ui.painter();

        // Set the background to white.
        painter.rect_filled(rect, 0.0, Color32::WHITE);

        // Draw the vertical grid lines.
        for row in 0..=cols {
            let offset_x = cell_width * row as f32;

            painter.line_segment(
                [
                    egui::pos2(rect.left() + offset_x, rect.top()),
                    egui::pos2(rect.left() + offset_x, rect.bottom()),
                ],
                (GRID_LINE_WIDTH, Color32::BLACK),
            );
        }

        // Draw the horizontal grid lines.
        for col in 0..=rows {
            let offset_y = cell_height * col as f32;

            painter.line_segment(
                [
                    egui::pos2(rect.left(), rect.top() + offset_y),
                    egui::pos2(rect.right(), rect.top() + offset_y),
                ],
                (GRID_LINE_WIDTH, Color32::BLACK),
            );
        }

        let font_height = cell_width / 4.0;

        // Draw the numbers within each cell.
        for ((row, col), value) in self.grid.indexed_iter() {
            let offset_x = cell_width * row as f32;
            let offset_y = cell_height * col as f32;

            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + offset_x, rect.top() + offset_y),
                egui::vec2(cell_width, cell_height),
            );

            // Draw the value.
            painter.text(
                cell_rect.center(),
                egui::Align2::CENTER_CENTER,
                value.to_string(),
                FontId::monospace(font_height),
                Color32::BLACK,
            );
        }
    }

    fn show_viewport(&mut self, ui: &mut egui::Ui) {
        self.show_grid(ui);

        if ui.input(|i| i.viewport().close_requested()) {
            // Tell parent viewport that we should not show next frame:
            self.show_viewport = false;
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let text = format!("grid size {}", self.grid_size);
        let slider = egui::widgets::Slider::new(&mut self.grid_size, 1..=20)
            .text(text)
            .show_value(false);

        if ui.add(slider).dragged() {
            self.grid = Grid::new(self.grid_size, self.grid_size);
        }

        if ui.button("run").clicked() {
            self.show_viewport = true;
        }

        if self.show_viewport {
            ui.ctx().show_viewport_immediate(
                egui::ViewportId::from_hash_of("grid"),
                egui::ViewportBuilder::default()
                    .with_maximize_button(false)
                    .with_title("Grid")
                    .with_inner_size([400.0, 400.0])
                    .with_min_inner_size([100.0, 100.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| self.show_viewport(ui));
                },
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
