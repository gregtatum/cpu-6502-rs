pub use cpu_6502::ppu::NTSC_PALETTE;
pub const NAMETABLE_W: usize = 32;
pub const NAMETABLE_H: usize = 30;

// NTSC 720x480 display, but 720x534 display due to a 0.9 pixel aspect ratio.
pub const TEXTURE_DISPLAY_W: f32 = 720.0;
pub const TEXTURE_DISPLAY_H: f32 = 534.0;
pub const SIDE_PANEL_INNER_WIDTH: f32 = 256.0;
pub const SIDE_PANEL_MARGIN: f32 = 7.0;
pub const SIDE_PANEL_WIDTH: f32 =
    SIDE_PANEL_INNER_WIDTH + SIDE_PANEL_MARGIN + SIDE_PANEL_MARGIN;
pub const PALETTE_SWATCH_SIZE: f32 = 22.0;
