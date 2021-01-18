pub mod memory_range {
    pub struct Range {
        pub start: u16,
        pub end: u16,
    }

    impl Range {
        #[inline]
        pub const fn size(&self) -> u16 {
            self.end - self.start
        }

        pub const fn mask(&self) -> u16 {
            self.end - 1
        }
    }

    // 2KB internal RAM
    pub const RAM_ACTUAL: Range = Range {
        start: 0x0000,
        end: 0x0800,
    };
    // The RAM addresses are mirrored a total of 4 times.
    pub const RAM: Range = Range {
        start: 0x0000,
        end: 0x2000,
    };
    pub const PPU_ACTUAL: Range = Range {
        start: 0x2000,
        end: 0x2008,
    };
    // These mirror the ppu registers every 8 bytes.
    pub const PPU: Range = Range {
        start: 0x2008,
        end: 0x4000,
    };
    pub const APU_AND_IO_REGISTERES: Range = Range {
        start: 0x4000,
        end: 0x4017,
    };
    // APU and I/O functionality that is normally disabled. See CPU Test Mode.
    pub const DISABLED_APU_IO_FEATURES: Range = Range {
        start: 0x4018,
        end: 0x401F,
    };
    // Cartridge space: PRG ROM, PRG RAM, and mapper registers (See Note)
    // Size: 0xBFE0
    pub const CARTRIDGE_SPACE: Range = Range {
        start: 0x4020,
        end: 0xFFFF,
    };

    pub const PRG_ROM: Range = Range {
        start: 0x8000,
        end: 0xFFFF,
    };

    pub const STACK_PAGE: u8 = 0x01;
}

pub enum InterruptVectors {
    // The Non-Maskable Interrupt or NMI ($FFFA)
    NonMaskableInterrupt = 0xFFFA,
    ResetVector = 0xFFFC,
    IrqBrkVector = 0xFFFE,
}
