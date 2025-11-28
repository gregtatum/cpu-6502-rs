/// Placeholder for the new zero page UI integration using egui + SDL2.
pub struct ZeroPageNew;

impl ZeroPageNew {
    pub fn new() -> Self {
        Self
    }

    /// Render the zero page UI into the provided egui `Ui`.
    pub fn widget(&mut self, ui: &mut egui::Ui) {
        let _ = ui;
    }
}
