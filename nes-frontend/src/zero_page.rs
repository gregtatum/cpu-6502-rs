use nes_core::bus::Bus;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::cell::RefCell;
use std::collections::HashMap;

const ZERO_PAGE_SIDE: u16 = 16;
const ZERO_PAGE_BYTES: u16 = ZERO_PAGE_SIDE * ZERO_PAGE_SIDE;
const CELL_SCALE: u32 = 30;
const CELL_PADDING: u32 = 5;
const HEADER_SIZE: u32 = CELL_SCALE;
const FONT_SIZE: u16 = 80;
const WINDOW_WIDTH: u32 = (ZERO_PAGE_SIDE as u32 + 1) * CELL_SCALE;
const WINDOW_HEIGHT: u32 = (ZERO_PAGE_SIDE as u32 + 1) * CELL_SCALE;

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

        let texture_creator = canvas.texture_creator();
        let mut view = Self {
            canvas: RefCell::new(canvas),
            texture_creator,
            hex_textures: Default::default(),
            header_textures: Default::default(),
        };

        view.hex_textures.build_textures(&view.texture_creator)?;
        view.header_textures.build_textures(&view.texture_creator)?;

        Ok(view)
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
            let x = (index as u32 % ZERO_PAGE_SIDE as u32) * CELL_SCALE + HEADER_SIZE;
            let y = (index as u32 / ZERO_PAGE_SIDE as u32) * CELL_SCALE + HEADER_SIZE;

            // Fill in the background.
            canvas.set_draw_color(byte_to_color(byte));
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
        }

        Ok(())
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
    textures: HashMap<u8, Texture>,
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
            self.textures.insert(value, texture);
        }

        Ok(())
    }

    pub fn get(&self, value: u8) -> &Texture {
        self.textures
            .get(&value)
            .expect("Unable to get a texture from its byte value")
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
