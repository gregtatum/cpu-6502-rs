/// Placeholder for the new zero page UI integration using egui + SDL2.
pub struct ZeroPageNew {
    open: bool,
    hover: Option<(u8, u8)>,
    selected: Option<(u8, u8)>,
    sidebar_text: String,
}

impl ZeroPageNew {
    pub fn is_open(&self) -> bool {
        self.open
    }

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
                self.handle_keyboard(ui);
                ui.horizontal(|ui| {
                    self.grid(ui, zero_page);
                    ui.add_space(12.0);
                    self.sidebar(ui, zero_page);
                });
            });

        self.open = open;
    }

    fn grid(&mut self, ui: &mut egui::Ui, zero_page: Option<&[u8; 256]>) {
        const ZERO_PAGE_SIDE: usize = 16;
        const CELL: f32 = 30.0;
        const HEADER: f32 = 30.0;
        const UNSELECTED_DIM: f32 = 0.8;
        const UNSELECTED_DIM_HOVERED: f32 = 0.9;
        const DIM_1: f32 = 0.9;
        const DIM_2: f32 = 0.8;

        let grid_width = (ZERO_PAGE_SIDE as f32 + 1.0) * CELL;
        let grid_height = (ZERO_PAGE_SIDE as f32 + 1.0) * CELL;
        let desired_size = egui::vec2(grid_width, grid_height);

        ui.group(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
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
                    let dim = dim_factor_top(self.hover, self.selected, col as u8, DIM_2);
                    let color = color_with_dim(ui.visuals().strong_text_color(), dim);
                    let pos = egui::pos2(
                        rect.min.x + HEADER + col as f32 * CELL + CELL * 0.5,
                        rect.min.y + HEADER * 0.5,
                    );
                    painter.text(
                        pos,
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::TextStyle::Monospace.resolve(ui.style()),
                        color,
                    );
                }

                for row in 0..ZERO_PAGE_SIDE {
                    let label = format!("{row:02X}");
                    let dim =
                        dim_factor_side(self.hover, self.selected, row as u8, DIM_2);
                    let color = color_with_dim(ui.visuals().strong_text_color(), dim);
                    let pos = egui::pos2(
                        rect.min.x + HEADER * 0.5,
                        rect.min.y + HEADER + row as f32 * CELL + CELL * 0.5,
                    );
                    painter.text(
                        pos,
                        egui::Align2::CENTER_CENTER,
                        label,
                        egui::TextStyle::Monospace.resolve(ui.style()),
                        color,
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

                            let dim = dim_factor(
                                self.hover,
                                self.selected,
                                row as u8,
                                col as u8,
                                DIM_1,
                                DIM_2,
                                UNSELECTED_DIM,
                                UNSELECTED_DIM_HOVERED,
                            );
                            let text_color =
                                color_with_dim(ui.visuals().strong_text_color(), dim);

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
                                egui::TextStyle::Monospace.resolve(ui.style()),
                                text_color,
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

    fn handle_keyboard(&mut self, ui: &mut egui::Ui) {
        let mut select_if_empty = false;
        ui.input(|input| {
            for event in &input.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: _,
                    ..
                } = event
                {
                    match key {
                        egui::Key::ArrowUp => self.bump_selection(0i8, -1),
                        egui::Key::ArrowDown => self.bump_selection(0i8, 1),
                        egui::Key::ArrowLeft => self.bump_selection(-1, 0),
                        egui::Key::ArrowRight => self.bump_selection(1, 0),
                        egui::Key::Enter => select_if_empty = true,
                        _ => {}
                    }
                }
            }
        });

        if select_if_empty && self.selected.is_none() {
            self.selected = Some((0, 0));
        }
    }

    fn bump_selection(&mut self, dx: i8, dy: i8) {
        const ZERO_PAGE_SIDE: i8 = 16;
        if self.selected.is_none() {
            self.selected = Some((0, 0));
            return;
        }
        if let Some((row, col)) = self.selected {
            let mut new_col = col as i8 + dx;
            let mut new_row = row as i8 + dy;
            new_col = new_col.clamp(0, ZERO_PAGE_SIDE - 1);
            new_row = new_row.clamp(0, ZERO_PAGE_SIDE - 1);
            self.selected = Some((new_row as u8, new_col as u8));
        }
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, zero_page: Option<&[u8; 256]>) {
        ui.vertical(|ui| {
            ui.label("Sidebar");
            ui.add_space(8.0);
            ui.group(|ui| {
                ui.set_min_size(egui::vec2(240.0, 420.0));
                if let Some((row, col)) = self.selected {
                    let address: u16 = (row as u16) * 0x10 + col as u16;
                    ui.monospace(format!("Selected: 0x{address:02X} ({row},{col})"));
                } else {
                    ui.label("Selected: none");
                }

                ui.add_space(8.0);
                ui.label("Value:");
                if let Some(value) = self.sidebar_value(zero_page) {
                    ui.monospace(value);
                } else {
                    ui.label("N/A");
                }

                ui.add_space(12.0);
                ui.label("Notes:");
                ui.text_edit_multiline(&mut self.sidebar_text);
            });
        });
    }

    fn sidebar_value(&self, zero_page: Option<&[u8; 256]>) -> Option<String> {
        let (row, col) = self.selected?;
        let idx = (row as usize) * 16 + col as usize;
        let value = zero_page.map(|zp| zp.get(idx).copied()).flatten()?;
        Some(format!("0x{value:02X} @ {idx}"))
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

fn dim_factor(
    hover: Option<(u8, u8)>,
    selected: Option<(u8, u8)>,
    row: u8,
    col: u8,
    dim_1: f32,
    dim_2: f32,
    unselected_dim: f32,
    unselected_dim_hovered: f32,
) -> f32 {
    let mut factor = match hover {
        None => 1.0,
        Some((hover_row, hover_col)) => {
            if hover_row == row && hover_col == col {
                1.0
            } else if hover_row == row || hover_col == col {
                dim_1
            } else {
                dim_2
            }
        }
    };

    if let Some((selected_row, selected_col)) = selected {
        if selected_row == row && selected_col == col {
            return 1.0;
        }
        if hover.is_some() {
            factor *= unselected_dim_hovered;
        } else {
            factor *= unselected_dim;
        }
    }

    factor
}

fn dim_factor_top(
    hover: Option<(u8, u8)>,
    selected: Option<(u8, u8)>,
    col: u8,
    dim_2: f32,
) -> f32 {
    let mut factor = match hover {
        None => 1.0,
        Some((_, hc)) => {
            if hc == col {
                1.0
            } else {
                dim_2
            }
        }
    };

    if let Some((_, selected_col)) = selected {
        if selected_col == col {
            return 1.0;
        }
        factor *= 0.9;
    }

    factor
}

fn dim_factor_side(
    hover: Option<(u8, u8)>,
    selected: Option<(u8, u8)>,
    row: u8,
    dim_2: f32,
) -> f32 {
    let mut factor = match hover {
        None => 1.0,
        Some((hr, _)) => {
            if hr == row {
                1.0
            } else {
                dim_2
            }
        }
    };

    if let Some((selected_row, _)) = selected {
        if selected_row == row {
            return 1.0;
        }
        factor *= 0.9;
    }

    factor
}

fn color_with_dim(color: egui::Color32, factor: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    let scale = |v: u8| ((v as f32) * factor).clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgba_premultiplied(scale(r), scale(g), scale(b), a)
}
