const SIDEBAR_WIDTH: f32 = 200.0;

/// Create a Window that will visualize the zero page memory. The zero page memory
/// in the NES is the fast working memory that is used as working memory. This window
/// once completed will serve as a debug point for the zero page. You will be able
/// to see all of the values, set breakpoints when the memory changes to a certain value.
/// It will support mousemoves, clicks, and keyboard navigation. It's a window that
/// you will be able to open when working on the emulator as a whole.
pub struct ZeroPageWindow {
    open: bool,
    hover: Option<(u8, u8)>,
    selected: (u8, u8),
    breakpoint_cell: Option<(u8, u8)>,
    breakpoint_value: Option<(u8, u8, u8)>, // row, col, value
    grid_focused: bool,
    pending_keys: Vec<egui::Key>,
}

impl ZeroPageWindow {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn new() -> Self {
        Self {
            open: true,
            hover: None,
            selected: (0, 0),
            breakpoint_cell: None,
            breakpoint_value: None,
            grid_focused: false,
            pending_keys: Vec::new(),
        }
    }

    pub fn grid_focused(&self) -> bool {
        self.grid_focused
    }

    pub fn enqueue_key(&mut self, key: egui::Key) {
        self.pending_keys.push(key);
    }

    /// Render the zero page widget using egui immediate mode.
    pub fn widget(&mut self, ui: &mut egui::Ui, zero_page: Option<&[u8; 256]>) {
        let mut open = self.open;

        egui::Window::new("Zero Page")
            .open(&mut open)
            .collapsible(false)
            .auto_sized()
            .show(ui.ctx(), |ui| {
                let mut grid_has_focus = false;
                ui.horizontal(|ui| {
                    grid_has_focus = self.memory_grid(ui, zero_page);
                    self.sidebar(ui, zero_page);
                });
                self.grid_focused = grid_has_focus;
                self.handle_keyboard(ui, grid_has_focus);
            });

        self.open = open;
    }

    /// Draw the memory grid, a 16x16 visualization of the zero page memory.
    fn memory_grid(&mut self, ui: &mut egui::Ui, zero_page: Option<&[u8; 256]>) -> bool {
        const ZERO_PAGE_SIDE: usize = 16;
        const CELL: f32 = 30.0;
        const HEADER: f32 = 30.0;
        const UNSELECTED_DIM: f32 = 0.8;
        const UNSELECTED_DIM_HOVERED: f32 = 0.9;
        const DIM_1: f32 = 0.9;
        const DIM_2: f32 = 0.8;
        const CELL_RADIUS: f32 = 2.0;

        let grid_width = (ZERO_PAGE_SIDE as f32 + 1.0) * CELL;
        let grid_height = (ZERO_PAGE_SIDE as f32 + 1.0) * CELL;

        let mut grid_has_focus = false;
        ui.group(|ui| {
            let sense = egui::Sense::click_and_drag()
                .union(egui::Sense::focusable_noninteractive());
            ui.vertical(|ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::vec2(grid_width, grid_height), sense);
                grid_has_focus = response.has_focus();
                let rect = response.rect;

                // Fill the background.
                painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

                let grid_stroke = ui.visuals().widgets.noninteractive.bg_stroke;

                // Draw the grid.
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

                // Draw the header labels.
                for col in 0..ZERO_PAGE_SIDE {
                    let label = format!("x{col:01X}");
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

                // Draw the column labels.
                for row in 0..ZERO_PAGE_SIDE {
                    let label = format!("{row:01X}x");
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

                            let cell_color = color_with_dim(
                                byte_to_color(byte),
                                dim.max(UNSELECTED_DIM),
                            );
                            painter.rect_filled(cell_rect, CELL_RADIUS, cell_color);

                            let stroke = if self.selected == (row as u8, col as u8) {
                                let selected_color = if grid_has_focus {
                                    ui.visuals().widgets.active.fg_stroke.color
                                } else {
                                    ui.visuals().widgets.inactive.fg_stroke.color
                                };
                                egui::Stroke {
                                    width: 2.0,
                                    color: selected_color,
                                }
                            } else if self.hover == Some((row as u8, col as u8)) {
                                egui::Stroke {
                                    width: 1.5,
                                    color: ui.visuals().widgets.hovered.fg_stroke.color,
                                }
                            } else {
                                egui::Stroke::NONE
                            };

                            if stroke.width > 0.0 {
                                painter.rect_stroke(
                                    cell_rect,
                                    CELL_RADIUS,
                                    stroke,
                                    egui::StrokeKind::Inside,
                                );
                            }

                            // Draw the breakpoint outline.
                            let breakpoint = self.breakpoint_cell.or_else(|| {
                                self.breakpoint_value.map(|(r, c, _)| (r, c))
                            });
                            if let Some((breakpoint_row, breakpoint_column)) = breakpoint
                            {
                                if breakpoint_row == row as u8
                                    && breakpoint_column == col as u8
                                {
                                    painter.rect_stroke(
                                        cell_rect.shrink(2.0),
                                        CELL_RADIUS,
                                        egui::Stroke {
                                            width: 1.5,
                                            color: egui::Color32::from_rgb(255, 165, 0),
                                        },
                                        egui::StrokeKind::Inside,
                                    );
                                }
                            }

                            // Draw the text, e.g. "2F", "00", "1E"
                            painter.text(
                                cell_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{byte:02X}"),
                                egui::TextStyle::Monospace.resolve(ui.style()),
                                text_color,
                            );
                        }
                    }
                }

                // Hover + selection mapping
                if let Some(position) = response.hover_pos() {
                    self.hover = position_to_cell(
                        position,
                        rect.min,
                        HEADER,
                        CELL,
                        ZERO_PAGE_SIDE,
                    );
                } else {
                    self.hover = None;
                }

                if response.clicked() {
                    response.request_focus();
                    if let Some(position) = response.interact_pointer_pos() {
                        if let Some(cell) = position_to_cell(
                            position,
                            rect.min,
                            HEADER,
                            CELL,
                            ZERO_PAGE_SIDE,
                        ) {
                            self.selected = cell;
                        }
                    }
                }

                // Draw focus outline last so it sits above the grid and avoids clipping.
                if grid_has_focus {
                    let focus_stroke = egui::Stroke {
                        width: 1.0,
                        color: ui.visuals().widgets.active.fg_stroke.color,
                    };
                    painter.rect_stroke(
                        rect.shrink(0.5),
                        4.0,
                        focus_stroke,
                        egui::StrokeKind::Inside,
                    );
                }
            });
        });
        grid_has_focus
    }

    // Handle egui keyboard events when this window is open.
    fn handle_keyboard(&mut self, ui: &mut egui::Ui, grid_has_focus: bool) {
        if !grid_has_focus {
            self.pending_keys.clear();
            return;
        }
        let pending = std::mem::take(&mut self.pending_keys);
        for key in pending {
            ui.ctx()
                .input_mut(|input| input.consume_key(egui::Modifiers::default(), key));
            match key {
                egui::Key::ArrowUp => self.change_selection(0, -1),
                egui::Key::ArrowDown => self.change_selection(0, 1),
                egui::Key::ArrowLeft => self.change_selection(-1, 0),
                egui::Key::ArrowRight => self.change_selection(1, 0),
                _ => {}
            }
        }
    }

    /// Move the grid selection from an arrow key press.
    fn change_selection(&mut self, dx: i8, dy: i8) {
        const ZERO_PAGE_SIDE: i8 = 16;
        let (row, col) = self.selected;
        let mut new_col = col as i8 + dx;
        let mut new_row = row as i8 + dy;
        new_col = new_col.clamp(0, ZERO_PAGE_SIDE - 1);
        new_row = new_row.clamp(0, ZERO_PAGE_SIDE - 1);
        self.selected = (new_row as u8, new_col as u8);
    }

    /// Create the sidebar UI.
    fn sidebar(&mut self, ui: &mut egui::Ui, zero_page: Option<&[u8; 256]>) {
        let zero_page = zero_page.expect("The zero page exists");

        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.set_max_width(SIDEBAR_WIDTH);
                // Draw the value information.
                let (row, col) = self.selected;
                let address: u16 = (row as u16) * 0x10 + col as u16;
                let value = {
                    let (row, col) = self.selected;
                    let idx = (row as usize) * 16 + col as usize;
                    zero_page
                        .get(idx)
                        .expect("Out of bounds access on the zero_page.")
                };

                ui.monospace(format!("Address ${address:02X}"));
                ui.monospace(format!("Hex     ${}", format!("0x{value:02X}")));
                ui.monospace(format!("Decimal {}", format!("{value}")));
                ui.monospace(format!("Binary  {}", format_as_bits(*value)));

                ui.separator();

                // Compute the breakpoint display state.
                let mut is_breakpoint_cell = self.breakpoint_cell == Some(self.selected);
                let mut is_breakpoint_value = self
                    .breakpoint_value
                    .map(|(r, c, _)| Some((r, c)) == Some(self.selected))
                    .unwrap_or(false);
                let mut target_value: u8 = self
                    .breakpoint_value
                    .and_then(|(_, _, v)| Some(v))
                    .unwrap_or(0);

                // Draw the breakpoints.
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut is_breakpoint_cell, "Breakpoint").clicked() {
                        if is_breakpoint_cell {
                            self.breakpoint_cell = Some(self.selected);
                            self.breakpoint_value = None;
                        } else {
                            self.breakpoint_cell = None;
                        }
                    }

                    if ui
                        .checkbox(&mut is_breakpoint_value, "Break on value")
                        .clicked()
                    {
                        if is_breakpoint_value {
                            let (row, col) = self.selected;
                            self.breakpoint_value = Some((row, col, target_value));
                            self.breakpoint_cell = None;
                        } else {
                            self.breakpoint_value = None;
                        }
                    }
                });

                if is_breakpoint_value {
                    ui.horizontal(|ui| {
                        ui.label("Target value (hex):");
                        let mut value_str = format!("{target_value:02X}");
                        if ui.text_edit_singleline(&mut value_str).changed() {
                            if let Ok(val) = u8::from_str_radix(value_str.trim(), 16) {
                                target_value = val;
                                let (row, col) = self.selected;
                                self.breakpoint_value = Some((row, col, target_value));
                            }
                        }
                    });
                }
            });
        });
    }
}

/// Take an egui position and map it to a cell position.
fn position_to_cell(
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

// Apply a dim factor the hex cells.
fn dim_factor(
    hover: Option<(u8, u8)>,
    selected: (u8, u8),
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

    let (selected_row, selected_col) = selected;
    if selected_row == row && selected_col == col {
        return 1.0;
    }
    if hover.is_some() {
        factor *= unselected_dim_hovered;
    } else {
        factor *= unselected_dim;
    }

    factor
}

// Apply a dim factor to the top cells.
fn dim_factor_top(
    hover: Option<(u8, u8)>,
    selected: (u8, u8),
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

    if selected.1 == col {
        return 1.0;
    }
    factor *= 0.9;

    factor
}

// Apply a dim factor to the side.
fn dim_factor_side(
    hover: Option<(u8, u8)>,
    selected: (u8, u8),
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

    let (selected_row, _) = selected;
    if selected_row == row {
        return 1.0;
    }
    factor *= 0.9;

    factor
}

fn color_with_dim(color: egui::Color32, factor: f32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    let scale = |v: u8| ((v as f32) * factor).clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgba_premultiplied(scale(r), scale(g), scale(b), a)
}

fn byte_to_color(byte: u8) -> egui::Color32 {
    let hue_deg = (byte as f32 / 255.0) * 120.0 + 210.0;
    let s = 0.8;
    let v = 0.35 + ((byte & 0b0000_0111) as f32 / 7.0) * 0.10;
    let (r, g, b) = hsv_to_rgb(hue_deg, s, v);
    egui::Color32::from_rgb(r, g, b)
}

fn hsv_to_rgb(mut h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    h = h % 360.0;
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (rp, gp, bp) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let to_u8 = |f: f32| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to_u8(rp), to_u8(gp), to_u8(bp))
}

fn format_as_bits(b: u8) -> String {
    let bits = format!("{:08b}", b); // "00111111"
    let grouped = bits[..4].to_string() + "_" + &bits[4..]; // "0011_1111"
    format!("0b{}", grouped)
}
