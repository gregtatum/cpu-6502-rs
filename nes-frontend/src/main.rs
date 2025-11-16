pub mod drivers;

use crate::drivers::controller_sdl2::ControllerManager;
use nes_core::{mappers::SimpleProgram, nes_core::NesCore};

/// The front-end for the NES core, powered by SLD2.
struct NesFrontend {
    event_pump: sdl2::EventPump,
    controller_manager: ControllerManager,
    nes_core: NesCore,
}

impl NesFrontend {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;
        Ok(Self {
            event_pump: sdl.event_pump()?,
            nes_core: NesCore::new({
                let bytes: [u8; 256] = [0; 256];
                Box::new(SimpleProgram::load(&bytes))
            }),
            controller_manager: ControllerManager::new(&sdl)?,
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
