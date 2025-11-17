use nes_core::bus::Bus;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, Sdl};
use std::cell::RefCell;

const ZERO_PAGE_SIDE: u32 = 16;
const ZERO_PAGE_BYTES: usize = (ZERO_PAGE_SIDE * ZERO_PAGE_SIDE) as usize;
const WINDOW_SCALE: u32 = 20;
const RGB_BYTES: usize = 3;

pub struct ZeroPageWindow<'a> {
    texture: RefCell<Texture<'a>>,
    buffer: Vec<u8>,
    row_stride: usize,
    canvas: RefCell<Canvas<Window>>,
    texture_creator: TextureCreator<WindowContext>,
}

impl<'a> ZeroPageWindow<'a> {
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        let video_subsystem = sdl.video()?;

        let window_width = ZERO_PAGE_SIDE * WINDOW_SCALE;
        let window_height = ZERO_PAGE_SIDE * WINDOW_SCALE;

        // The outer window which contains options such as the sizing and borders.
        let window = video_subsystem
            .window("NES Emulator", window_width, window_height)
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
            .set_logical_size(ZERO_PAGE_SIDE, ZERO_PAGE_SIDE)
            .map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();

        let texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGB24,
                ZERO_PAGE_SIDE,
                ZERO_PAGE_SIDE,
            )
            .map_err(|e| e.to_string())?;

        let view = Self {
            texture: RefCell::new(texture),
            texture_creator,
            buffer: vec![0; ZERO_PAGE_BYTES * RGB_BYTES],
            row_stride: ZERO_PAGE_SIDE as usize * RGB_BYTES,
            canvas: RefCell::new(canvas),
        };

        view.blit_buffer();

        Ok(view)
    }

    fn blit_buffer(&self) -> Result<(), String> {
        self.texture
            .borrow_mut()
            .update(None, &self.buffer, self.row_stride)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update(&mut self, bus: &Bus) -> Result<bool, String> {
        let mut dirty = false;
        for index in 0..ZERO_PAGE_BYTES {
            let byte = bus.read_u8(index as u16);
            let [r, g, b] = byte_to_color(byte);
            let offset = index * RGB_BYTES;
            if self.buffer[offset] != r
                || self.buffer[offset + 1] != g
                || self.buffer[offset + 2] != b
            {
                self.buffer[offset] = r;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = b;
                dirty = true;
            }
        }

        if dirty {
            self.blit_buffer();
        }
        Ok(dirty)
    }

    fn present(&self) -> Result<(), String> {
        let mut canvas = self.canvas.borrow_mut();
        canvas
            .copy(&self.texture.borrow(), None, None)
            .map_err(|e| e.to_string())?;
        canvas.present();
        Ok(())
    }
}

fn byte_to_color(byte: u8) -> [u8; 3] {
    let hue = (byte & 0b1110_0000) >> 5;
    let shade = (byte & 0b0001_1111) << 3;
    match hue {
        0 => [shade, shade, shade],
        1 => [shade, 0, 0],
        2 => [0, shade, 0],
        3 => [0, 0, shade],
        4 => [shade, shade / 2, 0],
        5 => [0, shade, shade],
        6 => [shade, 0, shade],
        _ => [shade, 255 - shade, shade / 2],
    }
}
