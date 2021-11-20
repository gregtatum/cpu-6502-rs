use std::time::Duration;

use nes::cpu_6502::Cpu6502;
use sdl2::{Sdl, event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window};

pub struct System {
    pub cpu: Cpu6502,
    pub sdl_context: Sdl,
    pub canvas: Canvas<Window>,
    pub window_size: u32,
    pub window_scale: u32,
    pub device_pixels: u32,
}

impl System {
    pub fn new(cpu: Cpu6502) -> System {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window_size: u32 = 32;
        let window_scale: u32 = 8;
        let device_pixels: u32 = ((window_size as f32) * (window_scale as f32)) as u32;

        let window = video_subsystem.window("Simple Game", device_pixels, device_pixels)
            .build().unwrap();

        let canvas : Canvas<Window> = window.into_canvas()
            .present_vsync()
            .build()
            .unwrap();

        System {
            cpu,
            sdl_context,
            canvas,
            window_size,
            window_scale,
            device_pixels
        }
    }

    pub fn draw(&mut self, x: i32, y: i32) -> Result<(), String> {
        // Clear the canvas.
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Draw a demo square.
        self.canvas.set_draw_color(Color::RGB(255, 210, 0));
        let w = self.device_pixels as f32;
        self.canvas.fill_rect(Rect::new(
            (w * 0.25) as i32 + x,
            (w * 0.25) as i32 + y,
            (w * 0.5) as u32,
            (w * 0.5) as u32
        ))?;

        self.canvas.present();
        Ok(())
    }

    pub fn run_loop(&mut self) -> Result<(), String> {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let mut x = 0;
        let mut y = 0;
        loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        return Ok(());
                    },
                    Event::KeyDown { keycode: Some(key), .. } => {
                        match key {
                            Keycode::W | Keycode::Up => { y -= 1; },
                            Keycode::S | Keycode::Down => { y += 1; },
                            Keycode::A | Keycode::Left => { x -= 1; },
                            Keycode::D | Keycode::Right => { x += 1; },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            self.draw(x, y)?;

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
