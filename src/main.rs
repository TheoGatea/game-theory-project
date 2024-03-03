mod gametheory;

use eframe::{egui, Error};
use egui::{Color32, FontData, FontFamily, FontId, Margin, RichText, TextStyle};
use gametheory::{prisoners_dillemma_rules, Tournament};
use std::collections::BTreeMap;

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

    // make sure we fit everything next frame
    let total_required_width = total_spacing + max_column_width * 2.0;

    let size = egui::vec2(ui.available_width().max(total_required_width), max_height);
    ui.advance_cursor_after_rect(egui::Rect::from_min_size(top_left, size));
    result
}

struct App {
    /// How many steps each simulation run does
    n_iters: u32,
    /// Game being played.
    game: Tournament,
}

impl App {
    fn new(cc: &eframe::CreationContext, game: Tournament) -> Self {
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

        Self { game, n_iters: 10 }
    }

    fn reset_game(&mut self) {
        self.game = Tournament::from(self.n_iters, prisoners_dillemma_rules);
    }

    fn show_grid(&mut self, ui: &mut egui::Ui) {
        const GRID_LINE_WIDTH: f32 = 4.;

        let rows = self.game.scores().rows();
        let cols = self.game.scores().cols();
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
        for row in 0..=cols + 1 {
            let offset_x = cell_width * row as f32;

            painter.line_segment(
                [
                    egui::pos2(rect.left() + offset_x, rect.top()),
                    egui::pos2(rect.left() + offset_x, rect.bottom()),
                ],
                (GRID_LINE_WIDTH, Color32::BLACK),
            );
        }

        let font_height = cell_width / 6.5;

        // Draw the horizontal grid lines.
        for col in 0..=rows + 1 {
            let offset_y = cell_height * col as f32;

            painter.line_segment(
                [
                    egui::pos2(rect.left(), rect.top() + offset_y),
                    egui::pos2(rect.right(), rect.top() + offset_y),
                ],
                (GRID_LINE_WIDTH, Color32::BLACK),
            );
        }

        // Draw horizontal headers. 
        for (col, player) in self.game.players().iter().enumerate() {
            let offset_x = cell_width * (col + 1) as f32;

            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + offset_x, rect.top()),
                egui::vec2(cell_width, cell_height),
            );

            // Draw the strategy.
            painter.text(
                cell_rect.center(),
                egui::Align2::CENTER_CENTER,
                player.strategy_name(),
                FontId::monospace(font_height),
                Color32::BLACK,
            );
        }

        // Draw vertical headers.
        for (row, player) in self.game.opponents().iter().enumerate() {
            let offset_y = cell_height * (row + 1) as f32;

            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left(), rect.top() + offset_y),
                egui::vec2(cell_width, cell_height),
            );

            // Draw the strategy.
            painter.text(
                cell_rect.center(),
                egui::Align2::CENTER_CENTER,
                player.strategy_name(),
                FontId::monospace(font_height),
                Color32::BLACK,
            );
        }

        // Draw the scores within each cell.
        for row in 0..rows {
            for col in 0..cols {
                let (v_score, h_score) = self.game.scores()[(col, row)];

                // Don't show empty cells.
                if v_score == 0 && h_score == 0 {
                    continue;
                }

                // Offset by 1 to allow for the column/row headers.
                let offset_x = cell_width * (row + 1) as f32;
                let offset_y = cell_height * (col + 1) as f32;

                let cell_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + offset_x, rect.top() + offset_y),
                    egui::vec2(cell_width, cell_height),
                );

                // Draw the value.
                painter.text(
                    cell_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("({v_score}, {h_score})"),
                    FontId::monospace(font_height),
                    Color32::BLACK,
                );
            }
        }
    }

    fn show_left(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().spacing.item_spacing.x = 10.0;

        ui.label(RichText::new(format!("#Iterations {}", self.n_iters)).size(14.0));
        ui.add(egui::widgets::Slider::new(&mut self.n_iters, 10..=100).show_value(false));

        if ui.button("Simulate").clicked() {
            self.reset_game();
            while !self.game.step() {}
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
            self.show_grid(rui);
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| self.show(ui));
    }
}

fn main() -> Result<(), Error> {
    let game = Tournament::from(100, prisoners_dillemma_rules);

    eframe::run_native(
        "Game Theory",
        eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        },
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(App::new(cc, game))
        }),
    )
}
