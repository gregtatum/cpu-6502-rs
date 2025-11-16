pub mod drivers;

use crate::drivers::controller_sdl2::ControllerManager;
use cpu_6502::{emulator::Emulator, mappers::SimpleProgram};

struct System {
    event_pump: sdl2::EventPump,
    controller_manager: ControllerManager,
    emulator: Emulator,
}

impl System {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init()?;
        Ok(Self {
            event_pump: sdl.event_pump()?,
            emulator: Emulator::new({
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
            self.controller_manager.handle_event(&event, &self.emulator);
        }
    }
}

fn main() {
    match System::new() {
        Ok(mut system) => {
            system.step();
        }
        Err(message) => {
            eprintln!("Failed to start the system: {message}");
        }
    }
}
