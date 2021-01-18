use std::rc::Rc;

use crate::cpu_6502::Cpu6502;
use crate::ppu::Ppu;
use crate::{
    bus::{Bus, SharedBus},
    mappers::Mapper,
};

pub struct Emulator {
    pub bus: SharedBus,
    pub cpu: Cpu6502,
    pub ppu: Ppu,
}

impl Emulator {
    pub fn new(cartridge: Box<dyn Mapper>) -> Emulator {
        let bus = Bus::new_shared_bus(cartridge);
        Emulator {
            cpu: Cpu6502::new(Rc::clone(&bus)),
            ppu: Ppu::new(Rc::clone(&bus)),
            // Take ownership of the initial bus.
            bus,
        }
    }
}
