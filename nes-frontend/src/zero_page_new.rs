/// Placeholder for the new zero page UI integration using egui + SDL2.
pub struct ZeroPageNew {
    open: bool,
    hover: Option<(u8, u8)>,
    selected: Option<(u8, u8)>,
    sidebar_text: String,
}

impl ZeroPageNew {
    pub fn new() -> Self {
        Self {
            open: true,
            hover: None,
            selected: None,
            sidebar_text: String::new(),
        }
    }

    /// Render the zero page UI into the provided egui `Ui`.
    pub fn widget(&mut self, ui: &mut egui::Ui, zero_page: Option<&[u8; 256]>) {
        let mut open = self.open;

        egui::Window::new("Zero Page")
            .open(&mut open)
            .collapsible(false)
            .auto_sized()
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    self.grid(ui, zero_page);
                    ui.add_space(12.0);
                    self.sidebar(ui);
                });
            });

        self.open = open;
    }

    fn grid(&mut self, ui: &mut egui::Ui, zero_page: Option<&[u8; 256]>) {
        const ZERO_PAGE_SIDE: usize = 16;
        const CELL: f32 = 30.0;
        const HEADER: f32 = 30.0;

        let grid_width = (ZERO_PAGE_SIDE as f32 + 1.0) * CELL;
        let grid_height = (ZERO_PAGE_SIDE as f32 + 1.0) * CELL;
        let desired_size = egui::vec2(grid_width, grid_height);

        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Hex grid");
                ui.add_space(4.0);

                let (response, painter) =
                    ui.allocate_painter(desired_size, egui::Sense::click_and_drag());
                let rect = response.rect;

                // Background
                painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

                let grid_stroke = ui.visuals().widgets.noninteractive.bg_stroke;

                for row in 0..=ZERO_PAGE_SIDE {
                    let y = rect.min.y + HEADER + row as f32 * CELL;
                    let start = egui::pos2(rect.min.x + HEADER, y);
                    let end = egui::pos2(rect.max.x, y);
                    painter.line_segment([start, end], grid_stroke);
                }

                for col in 0..=ZERO_PAGE_SIDE {
                    let x = rect.min.x + HEADER + col as f32 * CELL;
                    let start = egui::pos2(x, rect.min.y + HEADER);
                    let end = egui::pos2(x, rect.max.y);
                    painter.line_segment([start, end], grid_stroke);
                }

                // Header labels
                for col in 0..ZERO_PAGE_SIDE {
                    let label = format!("{col:02X}");
                    let pos = egui::pos2(
                        rect.min.x + HEADER + col as f32 * CELL + CELL * 0.5,
                        rect.min.y + HEADER * 0.5,
                    );
                    painter.text(
                        pos,
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::TextStyle::Body.resolve(ui.style()),
                        ui.visuals().strong_text_color(),
                    );
                }

                for row in 0..ZERO_PAGE_SIDE {
                    let label = format!("{row:02X}");
                    let pos = egui::pos2(
                        rect.min.x + HEADER * 0.5,
                        rect.min.y + HEADER + row as f32 * CELL + CELL * 0.5,
                    );
                    painter.text(
                        pos,
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::TextStyle::Body.resolve(ui.style()),
                        ui.visuals().strong_text_color(),
                    );
                }

                if let Some(values) = zero_page {
                    for row in 0..ZERO_PAGE_SIDE {
                        for col in 0..ZERO_PAGE_SIDE {
                            let index = row * ZERO_PAGE_SIDE + col;
                            let byte = values[index];
                            let label = format!("{byte:02X}");
                            let cell_rect = egui::Rect::from_min_size(
                                egui::pos2(
                                    rect.min.x + HEADER + col as f32 * CELL,
                                    rect.min.y + HEADER + row as f32 * CELL,
                                ),
                                egui::vec2(CELL, CELL),
                            );

                            let fill = match (self.selected, self.hover) {
                                (Some(sel), _) if sel == (row as u8, col as u8) => {
                                    Some(ui.visuals().selection.bg_fill)
                                }
                                (_, Some(hover)) if hover == (row as u8, col as u8) => {
                                    Some(ui.visuals().widgets.hovered.bg_fill)
                                }
                                _ => None,
                            };

                            if let Some(fill) = fill {
                                painter.rect_filled(cell_rect, 2.0, fill);
                            }

                            painter.text(
                                cell_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                label,
                                egui::TextStyle::Body.resolve(ui.style()),
                                ui.visuals().strong_text_color(),
                            );
                        }
                    }
                }

                // Hover + selection mapping
                if let Some(pos) = response.hover_pos() {
                    self.hover = pos_to_cell(pos, rect.min, HEADER, CELL, ZERO_PAGE_SIDE);
                } else {
                    self.hover = None;
                }

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.selected =
                            pos_to_cell(pos, rect.min, HEADER, CELL, ZERO_PAGE_SIDE);
                    }
                }
            });
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label("Sidebar");
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.set_min_size(egui::vec2(220.0, 400.0));
                ui.label("Notes:");
                ui.text_edit_multiline(&mut self.sidebar_text);
            });
        });
    }
}

fn pos_to_cell(
    pos: egui::Pos2,
    rect_min: egui::Pos2,
    header: f32,
    cell: f32,
    side: usize,
) -> Option<(u8, u8)> {
    let local = pos - rect_min;
    if local.x < header || local.y < header {
        return None;
    }
    let col = ((local.x - header) / cell).floor() as i32;
    let row = ((local.y - header) / cell).floor() as i32;
    if row >= 0 && row < side as i32 && col >= 0 && col < side as i32 {
        Some((row as u8, col as u8))
    } else {
        None
    }
}
