use std::{cell::RefCell, time::Duration};

use nes::cpu_6502::Cpu6502;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl,
};

pub struct System {
    pub sdl_context: Sdl,
    pub canvas: RefCell<Canvas<Window>>,
    pub window_size: u32,
    pub window_scale: u32,
    pub device_pixels: u32,
    pub texture_creator: TextureCreator<WindowContext>,
}

impl System {
    pub fn new() -> System {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window_size: u32 = 32;
        let window_scale: u32 = 8;
        let device_pixels: u32 = ((window_size as f32) * (window_scale as f32)) as u32;

        let window = video_subsystem
            .window("Simple Game", device_pixels, device_pixels)
            .build()
            .unwrap();

        let canvas = RefCell::from(window.into_canvas().present_vsync().build().unwrap());

        let texture_creator = canvas.borrow_mut().texture_creator();

        System {
            sdl_context,
            canvas,
            window_size,
            window_scale,
            device_pixels,
            texture_creator,
        }
    }
}

pub struct SimpleGame<'a> {
    pub cpu: Cpu6502,
    pub system: &'a System,
    pub texture: Texture<'a>,
}

impl<'a> SimpleGame<'a> {
    pub fn new(cpu: Cpu6502, system: &'a System) -> SimpleGame<'a> {
        let texture = system
            .texture_creator
            .create_texture_target(
                PixelFormatEnum::RGB24,
                system.window_size,
                system.window_size,
            )
            .unwrap();

        SimpleGame {
            cpu,
            system,
            texture,
        }
    }

    pub fn draw(&mut self, x: i32, y: i32) -> Result<(), String> {
        // Clear the canvas.
        let mut canvas = self.system.canvas.borrow_mut();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Draw a demo square.
        canvas.set_draw_color(Color::RGB(255, 210, 0));
        let w = self.system.device_pixels as f32;
        canvas.fill_rect(Rect::new(
            (w * 0.25) as i32 + x,
            (w * 0.25) as i32 + y,
            (w * 0.5) as u32,
            (w * 0.5) as u32,
        ))?;

        canvas.present();
        Ok(())
    }

    pub fn run_loop(&mut self) -> Result<(), String> {
        let mut event_pump = self.system.sdl_context.event_pump().unwrap();
        let mut x = 0;
        let mut y = 0;
        loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        return Ok(());
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => match key {
                        Keycode::W | Keycode::Up => {
                            self.cpu.bus.borrow_mut().set_u8(0xff, 0x77);
                        }
                        Keycode::S | Keycode::Down => {
                            self.cpu.bus.borrow_mut().set_u8(0xff, 0x73);
                        }
                        Keycode::A | Keycode::Left => {
                            self.cpu.bus.borrow_mut().set_u8(0xff, 0x61);
                        }
                        Keycode::D | Keycode::Right => {
                            self.cpu.bus.borrow_mut().set_u8(0xff, 0x64);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            self.draw(x, y)?;

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
