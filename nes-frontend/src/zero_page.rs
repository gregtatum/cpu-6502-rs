use nes_core::bus::Bus;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use std::cell::RefCell;

const ZERO_PAGE_SIDE: u16 = 16;
const ZERO_PAGE_BYTES: u16 = (ZERO_PAGE_SIDE * ZERO_PAGE_SIDE);
const WINDOW_SCALE: u32 = 20;

/// Create a Window that will visualize the zero page memory.
pub struct ZeroPageWindow {
    canvas: RefCell<Canvas<Window>>,
    buffer: Vec<u8>,
}

impl ZeroPageWindow {
    /// Initialize the video subsystem, creates a [Window]`, the [Canvas], and the buffer.
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl.video()?;

        let window_width = ZERO_PAGE_SIDE as u32 * WINDOW_SCALE;
        let window_height = ZERO_PAGE_SIDE as u32 * WINDOW_SCALE;

        // The outer window which contains options such as the sizing and borders.
        let window = video_subsystem
            .window("Zero Page Memory", window_width, window_height)
            .position_centered()
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
            .set_logical_size(ZERO_PAGE_SIDE as u32, ZERO_PAGE_SIDE as u32)
            .map_err(|e| e.to_string())?;

        let view = Self {
            canvas: RefCell::new(canvas),
            buffer: vec![0xff; ZERO_PAGE_BYTES as usize],
        };

        Ok(view)
    }

    /// Update the underlying buffer from the bus.
    pub fn update(&mut self, bus: &Bus) -> Result<(), String> {
        for index in 0..ZERO_PAGE_BYTES {
            let byte = bus.read_u8(index);
            self.buffer[index as usize] = byte;
            self.fill_rect(index as u32, byte)?;
        }

        self.canvas.borrow_mut().present();

        Ok(())
    }

    fn fill_rect(&mut self, index: u32, byte: u8) -> Result<(), String> {
        let mut canvas = self.canvas.borrow_mut();
        let x = index % ZERO_PAGE_SIDE as u32;
        let y = index / ZERO_PAGE_SIDE as u32;
        canvas.set_draw_color(byte_to_color(byte));

        canvas
            .draw_rect(Rect::new(x as i32, y as i32, x + 1, y + 1))
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn byte_to_color(byte: u8) -> Color {
    let hue = (byte & 0b1110_0000) >> 5;
    let shade = (byte & 0b0001_1111) << 3;
    match hue {
        0 => Color::RGB(shade, shade, shade),
        1 => Color::RGB(shade, 0, 0),
        2 => Color::RGB(0, shade, 0),
        3 => Color::RGB(0, 0, shade),
        4 => Color::RGB(shade, shade / 2, 0),
        5 => Color::RGB(0, shade, shade),
        6 => Color::RGB(shade, 0, shade),
        _ => Color::RGB(shade, 255 - shade, shade / 2),
    }
}
