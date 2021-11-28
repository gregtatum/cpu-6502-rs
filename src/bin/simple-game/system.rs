use std::cell::RefCell;

use nes::cpu_6502::Cpu6502;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl,
};

pub struct ScreenBuffer<'a> {
    pub texture_data: Vec<u8>,
    pub texture: Texture<'a>,
    pub texture_row_size: usize,
    pub mem_offset: (u16, u16),
}

impl<'a> ScreenBuffer<'a> {
    pub fn new(system: &'a System, mem_offset: (u16, u16)) -> ScreenBuffer<'a> {
        let texture = system
            .texture_creator
            .create_texture_target(
                PixelFormatEnum::RGB24,
                system.window_size,
                system.window_size,
            )
            .unwrap();

        let u8s_per_pixel = 3;

        assert_eq!(
            mem_offset.1 - mem_offset.0,
            (system.window_size * system.window_size) as u16,
            "The mem_offset was not the correct size for the buffer"
        );

        let texture_size =
            (system.window_size * system.window_size * u8s_per_pixel) as usize;

        ScreenBuffer {
            texture_data: vec![0; texture_size],
            texture,
            texture_row_size: (system.window_size * u8s_per_pixel) as usize,
            mem_offset,
        }
    }

    pub fn update(&mut self, cpu: &Cpu6502) -> bool {
        let mut frame_index = 0;
        let mut texture_dirty = false;
        let bus = cpu.bus.borrow_mut();
        for index in self.mem_offset.0..self.mem_offset.1 {
            let (b1, b2, b3) = color(bus.read_u8(index as u16)).rgb();
            if self.texture_data[frame_index] != b1
                || self.texture_data[frame_index + 1] != b2
                || self.texture_data[frame_index + 2] != b3
            {
                self.texture_data[frame_index] = b1;
                self.texture_data[frame_index + 1] = b2;
                self.texture_data[frame_index + 2] = b3;
                texture_dirty = true;
            }
            frame_index += 3;
        }
        if texture_dirty {
            self.texture
                .update(None, &self.texture_data, self.texture_row_size)
                .unwrap();
        }
        texture_dirty
    }
}

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

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
    pub screen: ScreenBuffer<'a>,
}

impl<'a> SimpleGame<'a> {
    pub fn new(cpu: Cpu6502, system: &'a System) -> SimpleGame<'a> {
        SimpleGame {
            cpu,
            system,
            // 0x200 to 0x600 is within the RAM range of the CPU.
            screen: ScreenBuffer::new(&system, (0x200, 0x600)),
        }
    }

    pub fn draw(&mut self) -> Result<(), String> {
        if self.screen.update(&self.cpu) {
            let mut canvas = self.system.canvas.borrow_mut();
            canvas.copy(&self.screen.texture, None, None).unwrap();
            canvas.present();
        }

        Ok(())
    }

    pub fn run_loop(&mut self) -> Result<(), String> {
        let mut event_pump = self.system.sdl_context.event_pump().unwrap();
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

            self.cpu
                .bus
                .borrow_mut()
                .set_u8(0xfe, rand::random::<u8>() % 15 + 1);

            self.cpu.tick();
            self.draw()?;
            ::std::thread::sleep(std::time::Duration::new(0, 10_000));
        }
    }
}
