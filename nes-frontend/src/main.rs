pub mod drivers;
pub mod zero_page;

use crate::drivers::controller_sdl2::ControllerManager;
use crate::zero_page::ZeroPageWindow;
use nes_core::{
    asm::{AsmLexer, BytesLabels},
    cpu_6502::ExitReason,
    mappers::SimpleProgram,
    nes_core::NesCore,
    opcodes::OpCode,
};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread;
use std::time::{Duration, Instant};

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

        let nes_core = create_demo_core();

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
    ///   4. Sleeping to keep an ~60Hz cadence.
    fn run(&mut self) -> Result<(), String> {
        const TARGET_FRAME_TIME: Duration = Duration::from_nanos(16_666_667);
        loop {
            let frame_start = Instant::now();

            if self.process_events()? {
                break;
            }

            // Keep running after BRK so the SDL window stays alive; only exit on KIL.
            match self.nes_core.frame() {
                ExitReason::KIL => break,
                ExitReason::BRK | ExitReason::MaxTicks => {}
            }

            if let Some(window) = self.zero_page_window.as_mut() {
                let bus = self.nes_core.bus.borrow();
                window.update(&bus)?;
            }

            let elapsed = frame_start.elapsed();
            if elapsed < TARGET_FRAME_TIME {
                thread::sleep(TARGET_FRAME_TIME - elapsed);
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

fn create_demo_core() -> NesCore {
    let mut lexer = AsmLexer::new(
        "
            ; Fill the zero page with incrementing values.
            lda #$22
            root:
                sta $00,x
                adc #1
                inx
                brk
                jmp root
        ",
    );
    match lexer.parse() {
        Ok(()) => {
            let BytesLabels { mut bytes, .. } = lexer.into_bytes().unwrap();
            bytes.push(OpCode::KIL as u8);
            NesCore::new(Box::new(SimpleProgram::load(&bytes)))
        }
        Err(error) => {
            error.panic_nicely();
            panic!("Failed to parse fill zero page script");
        }
    }
}

fn main() {
    match NesFrontend::new() {
        Ok(mut system) => {
            if let Err(message) = system.run() {
                eprintln!("Front-end error: {message}");
            } else {
                println!("Exiting gracefully");
            }
        }
        Err(message) => {
            eprintln!("Failed to start the system: {message}");
        }
    }
}
