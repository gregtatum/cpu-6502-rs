use std::rc::Rc;

use crate::cpu_6502::Cpu6502;
use crate::{
    bus::{Bus, SharedBus},
    mappers::Mapper,
};

/// The core logic for the NES. It requires a front-end to actually produce
/// video, sound, and take gamepad input.
pub struct NesCore {
    pub bus: SharedBus,
    pub cpu: Cpu6502,
}

impl NesCore {
    pub fn new(cartridge: Box<dyn Mapper>) -> NesCore {
        let bus = Bus::new_shared_bus(cartridge);
        NesCore {
            cpu: Cpu6502::new(Rc::clone(&bus)),
            // Take ownership of the initial bus.
            bus,
        }
    }

    pub fn run(&mut self) {
        self.cpu.run();
    }

    pub fn step(&mut self) {
        self.cpu.tick();
    }

    pub fn frame(self) {
        unimplemented!("The frame code must be implemented still.");
    }
}

pub trait ControllerDriver {
    fn step(&mut self, emulator: &NesCore);
}

#[cfg(test)]
mod test {
    use crate::asm::{AsmLexer, BytesLabels};
    use crate::mappers::SimpleProgram;
    use crate::opcodes::OpCode;

    use super::*;

    pub fn create_emulator(text: &str) -> NesCore {
        let mut lexer = AsmLexer::new(text);

        match lexer.parse() {
            Ok(_) => {
                let BytesLabels { mut bytes, .. } = lexer.into_bytes().unwrap();
                bytes.push(OpCode::KIL as u8);
                let cartridge = Box::new(SimpleProgram::load(&bytes));
                NesCore::new(cartridge)
            }
            Err(parse_error) => {
                parse_error.panic_nicely();
                panic!("");
            }
        }
    }

    #[test]
    fn test_emulator() {
        let cartridge = Box::new(SimpleProgram::new());
        let mut emulator = NesCore::new(cartridge);
        emulator.step();
    }

    #[test]
    fn test_controllers() {
        let mut emulator = create_emulator("
            ; At the same time that we strobe bit 0, we initialize the ring counter
            ; so we're hitting two birds with one stone here
            read_joypad:
                lda #$01
                ; While the strobe bit is set, buttons will be continuously reloaded.
                ; This means that reading from JOYPAD1 will only return the state of the
                ; first button: button A.
                sta $4016    ; JOYPAD1
                sta $33
                lsr a        ; now A is 0
                ; By storing 0 into JOYPAD1, the strobe bit is cleared and the reloading stops.
                ; This allows all 8 buttons (newly reloaded) to be read from JOYPAD1.
                sta $4016    ; JOYPAD1
            loop:
                lda $4016    ; JOYPAD1
                lsr a        ; bit 0 -> Carry
                rol $33      ; Carry -> bit 0; bit 7 -> Carry
                bcc loop
        ");
        emulator.cpu.max_ticks = Some(100);

        {
            // Mutate the controller.
            let bus = emulator.bus.borrow();
            let mut controller = bus
                .controller_1
                .as_ref()
                .expect("controller_1 should exist")
                .borrow_mut();

            controller.a = true;
            controller.select = true;
            controller.up = true;
        }

        emulator.run();

        // The value for the controller read should be on the zero page at $33.
        assert_eq!(emulator.bus.borrow().read_u8(0x33), 0b1010_1000);
    }
}
