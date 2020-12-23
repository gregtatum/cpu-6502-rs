use crate::bus::{Bus, SharedBus};
use crate::cpu_6502::Cpu6502;
use crate::ppu::Ppu;

pub struct Emulator {
    pub bus: SharedBus,
    pub cpu: Cpu6502,
    pub ppu: Ppu,
}

impl Emulator {
    pub fn new() -> Emulator {
        let bus = Bus::new_shared_bus();

        Emulator {
            cpu: Cpu6502::new(bus.clone()),
            ppu: Ppu::new(bus.clone()),
            // Take ownership of the initial bus.
            bus,
        }
    }
}
