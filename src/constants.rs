pub mod memory_range {
    pub struct Range {
        pub min: u16,
        pub max: u16,
    }

    impl Range {
        #[inline]
        pub fn size(&self) -> u16 {
            self.max - self.min
        }
    }

    // 2KB internal RAM
    pub const RAM_ACTUAL: Range = Range {
        min: 0x0000,
        max: 0x07ff,
    };
    // The RAM addresses are mirrored a total of 4 times.
    pub const RAM: Range = Range {
        min: 0x0000,
        max: 0x1fff,
    };
    pub const PPU_ACTUAL: Range = Range {
        min: 0x2000,
        max: 0x2007,
    };
    // These mirror the ppu registers every 8 bytes.
    pub const PPU: Range = Range {
        min: 0x2008,
        max: 0x4000,
    };
    pub const APU_AND_IO_REGISTERES: Range = Range {
        min: 0x4000,
        max: 0x4017,
    };
    // APU and I/O functionality that is normally disabled. See CPU Test Mode.
    pub const DISABLED_APU_IO_FEATURES: Range = Range {
        min: 0x4018,
        max: 0x401F,
    };
    // Cartridge space: PRG ROM, PRG RAM, and mapper registers (See Note)
    // Size: 0xBFE0
    pub const CARTRIDGE_SPACE: Range = Range {
        min: 0x4020,
        max: 0xFFFF,
    };

    pub const STACK_PAGE: u8 = 0x01;
}

pub enum InterruptVectors {
    // The Non-Maskable Interrupt or NMI ($FFFA)
    NonMaskableInterrupt = 0xFFFA,
    ResetVector = 0xFFFC,
    IrqBrkVector = 0xFFFE,
}
