use nes_core::bus::Bus;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    mouse::MouseButton,
    render::BlendMode,
};
use std::cell::RefCell;

const ZERO_PAGE_SIDE: u16 = 16;
const ZERO_PAGE_BYTES: u16 = ZERO_PAGE_SIDE * ZERO_PAGE_SIDE;
const CELL_SCALE: u32 = 30;
const CELL_PADDING: u32 = 5;
const HEADER_SIZE: u32 = CELL_SCALE;
const FONT_SIZE: u16 = 80;
const WINDOW_WIDTH: u32 = (ZERO_PAGE_SIDE as u32 + 1) * CELL_SCALE;
const WINDOW_HEIGHT: u32 = (ZERO_PAGE_SIDE as u32 + 1) * CELL_SCALE;
const UNSELECTED_DIM: f32 = 0.8;
const UNSELECTED_DIM_HOVERED: f32 = 0.9;
const DIM_1: f32 = 0.9;
const DIM_2: f32 = 0.8;

/// Create a Window that will visualize the zero page memory. The zero page memory
/// in the NES is the fast working memory that is used as working memory. This window
/// once completed will serve as a debug point for the zero page. You will be able
/// to see all of the values, set breakpoints when the memory changes to a certain value.
/// It will support mousemoves, clicks, and keyboard navigation. It's a window that
/// you will be able to open when working on the emulator as a whole.
pub struct ZeroPageWindow {
    canvas: RefCell<Canvas<Window>>,
    hex_textures: HexTextures,
    header_textures: HeaderTextures,
    texture_creator: TextureCreator<WindowContext>,
    hover: Option<(u8, u8)>,    // (row, col)
    selected: Option<(u8, u8)>, // (row, col)
    window_id: u32,
}

impl ZeroPageWindow {
    /// Initialize the video subsystem, creates a [Window]`, the [Canvas].
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl.video()?;

        // The outer window which contains options such as the sizing and borders.
        let window = video_subsystem
            .window("Zero Page Memory", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .allow_highdpi()
            .build()
            .map_err(|e| e.to_string())?;

        // Owns the canvas that can be drawn to, and is associated with the window.
        let mut canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        canvas
            .set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .map_err(|e| e.to_string())?;

        canvas.set_blend_mode(BlendMode::Blend);

        let window_id = canvas.window().id();
        let texture_creator = canvas.texture_creator();
        let mut view = Self {
            canvas: RefCell::new(canvas),
            texture_creator,
            hex_textures: Default::default(),
            header_textures: Default::default(),
            hover: None,
            selected: None,
            window_id,
        };

        view.hex_textures.build_textures(&view.texture_creator)?;
        view.header_textures.build_textures(&view.texture_creator)?;

        Ok(view)
    }

    /// Handle events from the global event_pump.
    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::MouseMotion {
                x, y, window_id, ..
            } => {
                if *window_id != self.window_id {
                    return;
                }
                self.hover = cell_from_point(*x, *y);
            }
            Event::MouseButtonDown {
                x,
                y,
                window_id,
                mouse_btn,
                ..
            } if *mouse_btn == MouseButton::Left => {
                if *window_id != self.window_id {
                    return;
                }
                if let Some((row, col)) = cell_from_point(*x, *y) {
                    self.select_cell(row, col);
                }
            }
            Event::Window {
                win_event: WindowEvent::Leave,
                window_id,
                ..
            } if *window_id == self.window_id => {
                self.hover = None;
            }
            Event::KeyDown {
                keycode: Some(key), ..
            } => {
                self.handle_key(*key);
            }
            _ => {}
        }
    }

    /// Draw the view.
    pub fn draw(&mut self, bus: &Bus) -> Result<(), String> {
        self.draw_memory_cells(&bus)?;
        self.draw_headers()?;
        self.canvas.borrow_mut().present();
        Ok(())
    }

    /// Draw the background and the memory value of each memory "cell" in the zero page.
    fn draw_memory_cells(&mut self, bus: &Bus) -> Result<(), String> {
        for index in 0..ZERO_PAGE_BYTES {
            let byte = bus.read_u8(index);
            let mut canvas = self.canvas.borrow_mut();
            let col = (index as u32 % ZERO_PAGE_SIDE as u32) as u8;
            let row = (index as u32 / ZERO_PAGE_SIDE as u32) as u8;
            let x = col as u32 * CELL_SCALE + HEADER_SIZE;
            let y = row as u32 * CELL_SCALE + HEADER_SIZE;

            // Fill in the background.
            let mut color = byte_to_color(byte);
            let dim = dim_factor(self.hover, self.selected, row as u8, col as u8);
            apply_dim(&mut color, dim);
            canvas.set_draw_color(color);
            canvas
                .fill_rect(Rect::new(x as i32, y as i32, CELL_SCALE, CELL_SCALE))
                .map_err(|e| e.to_string())?;

            // Draw a border.
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas
                .draw_rect(Rect::new(x as i32, y as i32, CELL_SCALE, CELL_SCALE))
                .map_err(|e| e.to_string())?;

            // Draw the value.
            let texture = self.hex_textures.get(byte);
            let target = Rect::new(
                (x + CELL_PADDING) as i32,
                (y + CELL_PADDING) as i32,
                (CELL_SCALE - CELL_PADDING * 2) as u32,
                (CELL_SCALE - CELL_PADDING * 2) as u32,
            );
            canvas.copy(&texture, None, Some(target))?;

            // Dim overlay (covers both background and text).
            if dim < 1.0 {
                let alpha = ((1.0 - dim) * 255.0) as u8;
                canvas.set_draw_color(Color::RGBA(0, 0, 0, alpha));
                canvas
                    .fill_rect(Rect::new(x as i32, y as i32, CELL_SCALE, CELL_SCALE))
                    .map_err(|e| e.to_string())?;
            }

            // Highlight selected cell with white border.
            if let Some((sr, sc)) = self.selected {
                if sr == row && sc == col {
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    canvas
                        .draw_rect(Rect::new(x as i32, y as i32, CELL_SCALE, CELL_SCALE))
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        Ok(())
    }

    /// Draw the address headers for rows and columns.
    fn draw_headers(&mut self) -> Result<(), String> {
        let mut canvas = self.canvas.borrow_mut();

        // Fill header backgrounds.
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas
            .fill_rect(Rect::new(0, 0, WINDOW_WIDTH, HEADER_SIZE))
            .map_err(|e| e.to_string())?;
        canvas
            .fill_rect(Rect::new(0, 0, HEADER_SIZE, WINDOW_HEIGHT))
            .map_err(|e| e.to_string())?;

        // Draw single separating lines
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas
            .draw_line(
                (HEADER_SIZE as i32, HEADER_SIZE as i32),
                (WINDOW_WIDTH as i32, HEADER_SIZE as i32),
            )
            .map_err(|e| e.to_string())?;
        canvas
            .draw_line(
                (HEADER_SIZE as i32, HEADER_SIZE as i32),
                (HEADER_SIZE as i32, WINDOW_HEIGHT as i32),
            )
            .map_err(|e| e.to_string())?;

        // Top header row.
        for col in 0..ZERO_PAGE_SIDE {
            let x = (HEADER_SIZE + col as u32 * CELL_SCALE) as i32;
            let y = 0i32;

            let texture = self.header_textures.top(col as u8);
            let target = Rect::new(
                x + CELL_PADDING as i32,
                y + CELL_PADDING as i32,
                (CELL_SCALE - CELL_PADDING * 2) as u32,
                (CELL_SCALE - CELL_PADDING * 2) as u32,
            );
            canvas.copy(texture, None, Some(target))?;

            // Dim overlay for header cell.
            let dim = dim_factor_top(self.hover, self.selected, col as u8);
            if dim < 1.0 {
                let alpha = ((1.0 - dim) * 255.0) as u8;
                canvas.set_draw_color(Color::RGBA(0, 0, 0, alpha));
                canvas
                    .fill_rect(Rect::new(x, y, CELL_SCALE, CELL_SCALE))
                    .map_err(|e| e.to_string())?;
            }
        }

        // Left header column.
        for row in 0..ZERO_PAGE_SIDE {
            let x = 0i32;
            let y = (HEADER_SIZE + row as u32 * CELL_SCALE) as i32;

            let texture = self.header_textures.side(row as u8);
            let target = Rect::new(
                x + CELL_PADDING as i32,
                y + CELL_PADDING as i32,
                (CELL_SCALE - CELL_PADDING * 2) as u32,
                (CELL_SCALE - CELL_PADDING * 2) as u32,
            );
            canvas.copy(texture, None, Some(target))?;

            // Dim overlay for header cell.
            let dim = dim_factor_side(self.hover, self.selected, row as u8);
            if dim < 1.0 {
                let alpha = ((1.0 - dim) * 255.0) as u8;
                canvas.set_draw_color(Color::RGBA(0, 0, 0, alpha));
                canvas
                    .fill_rect(Rect::new(x, y, CELL_SCALE, CELL_SCALE))
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    fn select_cell(&mut self, row: u8, col: u8) {
        self.selected = Some((row, col));
        let addr = row as u16 * ZERO_PAGE_SIDE as u16 + col as u16;
        println!(
            "Selected zero page cell {:02X} (row {}, col {})",
            addr, row, col
        );
    }

    fn handle_key(&mut self, key: Keycode) {
        match key {
            Keycode::Up => self.change_selection(-1, 0),
            Keycode::Down => self.change_selection(1, 0),
            Keycode::Left => self.change_selection(0, -1),
            Keycode::Right => self.change_selection(0, 1),
            _ => return,
        }
    }

    fn change_selection(&mut self, d_row: i32, d_col: i32) {
        let (row0, col0) = self.selected.or(self.hover).unwrap_or((0u8, 0u8));
        let mut row = row0 as i32 + d_row;
        let mut col = col0 as i32 + d_col;

        row = row.clamp(0, ZERO_PAGE_SIDE as i32 - 1);
        col = col.clamp(0, ZERO_PAGE_SIDE as i32 - 1);

        self.select_cell(row as u8, col as u8);
    }
}

/// Codegened utility to convert a byte value into a representative color.
fn byte_to_color(byte: u8) -> Color {
    let hue_deg = (byte as f32 / 255.0) * 120.0 + 210.0;
    let s = 0.8;
    // Keep it fairly dark; add a tiny variation from the low 3 bits
    let v = 0.35 + ((byte & 0b0000_0111) as f32 / 7.0) * 0.10;
    let (r, g, b) = hsv_to_rgb(hue_deg, s, v);
    Color::RGB(r, g, b)
}

/// Codegened utility to convert the color value.
fn hsv_to_rgb(mut h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    // h in [0,360), s,v in [0,1]
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

/// This struct retains the textures for the memory representations. It's technically
/// unsafe as the texture's lifetime is bounded to the texture_creator, but the lifetimes
/// were a mess with self-referential structures so I gave up on it and use the unsafe
/// lifetimes.
#[derive(Default)]
struct HexTextures {
    textures: Vec<Texture>,
}

impl HexTextures {
    pub fn build_textures(
        &mut self,
        texture_creator: &TextureCreator<WindowContext>,
    ) -> Result<(), String> {
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let font = ttf_context.load_font(
            "assets/liberation_mono/LiberationMono-Regular.ttf",
            FONT_SIZE,
        )?;

        for value in 0u8..=255u8 {
            let label = format!("{:02X}", value); // e.g. "0A", "FF"
            let surface = font
                .render(&label)
                .blended(Color::RGB(255, 255, 255))
                .map_err(|e| e.to_string())?;
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            self.textures.push(texture);
        }

        Ok(())
    }

    pub fn get(&self, value: u8) -> &Texture {
        &self.textures[value as usize]
    }
}

/// Contains the textures for the header.
///   top:  x0 x1 x2 ... xf
///   side: 0x 1x 2x ... fx
#[derive(Default)]
struct HeaderTextures {
    top: Vec<Texture>,
    side: Vec<Texture>,
}

impl HeaderTextures {
    pub fn build_textures(
        &mut self,
        texture_creator: &TextureCreator<WindowContext>,
    ) -> Result<(), String> {
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let font = ttf_context.load_font(
            "assets/liberation_mono/LiberationMono-Regular.ttf",
            FONT_SIZE,
        )?;

        for col in 0u8..ZERO_PAGE_SIDE as u8 {
            let label = format!("x{:X}", col);
            let surface = font
                .render(&label)
                .blended(Color::RGB(255, 255, 255))
                .map_err(|e| e.to_string())?;
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            self.top.push(texture);
        }

        for row in 0u8..ZERO_PAGE_SIDE as u8 {
            let label = format!("{:X}x", row);
            let surface = font
                .render(&label)
                .blended(Color::RGB(255, 255, 255))
                .map_err(|e| e.to_string())?;
            let texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;
            self.side.push(texture);
        }

        Ok(())
    }

    pub fn top(&self, index: u8) -> &Texture {
        &self.top[index as usize]
    }

    pub fn side(&self, index: u8) -> &Texture {
        &self.side[index as usize]
    }
}

fn dim_factor(
    hover: Option<(u8, u8)>,
    selected: Option<(u8, u8)>,
    row: u8,
    col: u8,
) -> f32 {
    let mut factor = match hover {
        None => 1.0,
        Some((hover_row, hover_col)) => {
            if hover_row == row && hover_col == col {
                // This cell is hovered.
                1.0
            } else if hover_row == row || hover_col == col {
                // This cell is in a row or column.
                DIM_1
            } else {
                // This cell is dimmed.
                DIM_2
            }
        }
    };

    if let Some((selected_row, selected_col)) = selected {
        if selected_row == row && selected_col == col {
            return 1.0;
        }

        // Slightly dim all other cells when a selection is present.
        if hover.is_some() {
            factor *= UNSELECTED_DIM_HOVERED;
        } else {
            factor *= UNSELECTED_DIM;
        }
    }

    factor
}

fn dim_factor_top(hover: Option<(u8, u8)>, selected: Option<(u8, u8)>, col: u8) -> f32 {
    let mut factor = match hover {
        None => 1.0,
        Some((_, hc)) => {
            if hc == col {
                1.0
            } else {
                DIM_2
            }
        }
    };

    if let Some((_, selected_col)) = selected {
        if selected_col == col {
            return 1.0;
        }
        factor *= UNSELECTED_DIM_HOVERED;
    }

    factor
}

fn dim_factor_side(hover: Option<(u8, u8)>, selected: Option<(u8, u8)>, row: u8) -> f32 {
    let mut factor = match hover {
        None => 1.0,
        Some((hr, _)) => {
            if hr == row {
                1.0
            } else {
                DIM_2
            }
        }
    };

    if let Some((selected_row, _)) = selected {
        if selected_row == row {
            return 1.0;
        }
        factor *= UNSELECTED_DIM_HOVERED;
    }

    factor
}

fn apply_dim(color: &mut Color, factor: f32) {
    let scale = |v: u8| ((v as f32 * factor).round().clamp(0.0, 255.0)) as u8;
    *color = Color::RGB(scale(color.r), scale(color.g), scale(color.b));
}

fn cell_from_point(x: i32, y: i32) -> Option<(u8, u8)> {
    // Translate from screen to grid.
    let grid_x = x - HEADER_SIZE as i32;
    let grid_y = y - HEADER_SIZE as i32;

    if grid_x < 0 || grid_y < 0 {
        return None;
    }

    let col = (grid_x as u32) / CELL_SCALE;
    let row = (grid_y as u32) / CELL_SCALE;

    if col < ZERO_PAGE_SIDE as u32 && row < ZERO_PAGE_SIDE as u32 {
        Some((row as u8, col as u8))
    } else {
        None
    }
}
