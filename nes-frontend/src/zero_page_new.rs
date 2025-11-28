/// Placeholder for the new zero page UI integration using egui + SDL2.
pub struct ZeroPageNew {
    open: bool,
}

impl ZeroPageNew {
    pub fn new() -> Self {
        Self { open: true }
    }

    /// Render the zero page UI into the provided egui `Ui`.
    pub fn widget(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Zero Page")
            .open(&mut self.open)
            .collapsible(false)
            .auto_sized()
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Zero page hex grid placeholder");
                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Hex grid layout");
                                ui.add_space(4.0);
                                // Reserve space for the future hex grid drawing.
                                ui.allocate_space(egui::vec2(400.0, 400.0));
                            });
                        });
                    });

                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.label("Sidebar placeholder");
                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.set_min_size(egui::vec2(220.0, 400.0));
                            ui.label("Sidebar content will go here.");
                        });
                    });
                });
            });
    }
}
