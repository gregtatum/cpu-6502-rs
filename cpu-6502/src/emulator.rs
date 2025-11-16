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

    pub fn step(&mut self) {
        self.cpu.tick();
    }

    pub fn frame(self) {
        // TODO - How do I start stubbing out the first frame here? Rather
        // than loading in a cartridge, I want to use my SimpleProgram. Maybe start
        // creating a mapp
    }
}

#[cfg(test)]
mod test {
    use crate::mappers::SimpleProgram;

    use super::*;

    #[test]
    fn test_emulator() {
        let cartridge = Box::new(SimpleProgram::new());
        let mut emulator = Emulator::new(cartridge);
        emulator.step();
    }
}
