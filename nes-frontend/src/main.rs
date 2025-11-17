pub mod drivers;
pub mod zero_page;

use crate::drivers::controller_sdl2::ControllerManager;
use crate::zero_page::ZeroPageWindow;
use nes_core::{cpu_6502::ExitReason, mappers::SimpleProgram, nes_core::NesCore};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

/// The front-end for the NES core, powered by SLD2.
struct NesFrontend {
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

    /// Run the program by:
    ///
    ///   1. Processing the events
    ///   2. Advancing the CPU by at most 1 frame.
    ///   3. Drawing that frame.
    fn run(&mut self) -> Result<(), String> {
        loop {
            if self.process_events()? {
                // The program should escape.
                break;
            }

            match self.nes_core.frame() {
                ExitReason::KIL => break,
                ExitReason::MaxTicks | ExitReason::BRK => {}
            }

            if let Some(window) = self.zero_page_window.as_mut() {
                let bus = self.nes_core.bus.borrow();
                window.update(&bus)?;
            }
        }

        Ok(())
    }

    /// Process the global event_pump, and return true if the program should exit.
    fn process_events(&mut self) -> Result<bool, String> {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(true),
                _ => self.controller_manager.handle_event(&event, &self.nes_core),
            }
        }

        Ok(false)
    }
}

fn main() {
    match NesFrontend::new() {
        Ok(mut system) => {
            if let Err(message) = system.run() {
                eprintln!("Front-end error: {message}");
            }
        }
        Err(message) => {
            eprintln!("Failed to start the system: {message}");
        }
    }
}
