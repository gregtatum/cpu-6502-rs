use std::rc::Rc;

use crate::cpu_6502::Cpu6502;
use crate::{
    bus::{Bus, SharedBus},
    mappers::Mapper,
};

pub struct Emulator {
    pub bus: SharedBus,
    pub cpu: Cpu6502,
}

impl Emulator {
    pub fn new(cartridge: Box<dyn Mapper>) -> Emulator {
        let bus = Bus::new_shared_bus(cartridge);
        Emulator {
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

#[cfg(test)]
mod test {
    use crate::asm::{AsmLexer, BytesLabels};
    use crate::mappers::SimpleProgram;
    use crate::opcodes::OpCode;

    use super::*;

    pub fn create_emulator(text: &str) -> Emulator {
        let mut lexer = AsmLexer::new(text);

        match lexer.parse() {
            Ok(_) => {
                let BytesLabels { mut bytes, .. } = lexer.into_bytes().unwrap();
                bytes.push(OpCode::KIL as u8);
                let cartridge = Box::new(SimpleProgram::load(&bytes));
                Emulator::new(cartridge)
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
        let mut emulator = Emulator::new(cartridge);
        emulator.step();
    }

    #[test]
    fn test_controllers() {
        let mut emulator = create_emulator("");
        emulator.run();
        assert_eq!(emulator.cpu.a, 0);
    }
}
