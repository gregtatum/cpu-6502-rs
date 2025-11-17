pub mod drivers;
pub mod zero_page;

use std::cell::RefCell;

use crate::drivers::controller_sdl2::ControllerManager;
use crate::zero_page::ZeroPageWindow;
use nes_core::{mappers::SimpleProgram, nes_core::NesCore};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;

/// The front-end for the NES core, powered by SLD2.
struct NesFrontend<'a> {
    event_pump: sdl2::EventPump,
    controller_manager: ControllerManager,
    nes_core: NesCore,
    zero_page_window: Option<ZeroPageWindow>,
}

impl NesFrontend {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;

        let nes_core = NesCore::new({
            let bytes: [u8; 256] = [0; 256];
            Box::new(SimpleProgram::load(&bytes))
        });

        Ok(Self {
            nes_core,
            event_pump: sdl.event_pump()?,
            controller_manager: ControllerManager::new(&sdl)?,
            zero_page_window: Some(ZeroPageWindow::new(&sdl)?),
        })
    }

    fn step(&mut self) {
        // Maps all of the SDL events to their respective components. There is a single global
        // event pump.
        for event in self.event_pump.poll_iter() {
            self.controller_manager.handle_event(&event, &self.nes_core);
        }
    }
}

fn main() {
    match NesFrontend::new() {
        Ok(mut system) => {
            system.step();
        }
        Err(message) => {
            eprintln!("Failed to start the system: {message}");
        }
    }
}
