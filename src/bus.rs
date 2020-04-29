pub struct Bus {
  memory: [u8; 0xFFFF],
}

impl Bus {
  pub fn read_word(&self, address: u16) -> u16 {
    self.construct_word(address, address + 1)
  }

  fn construct_word(&self, address_a: u16, address_b: u16) -> u16 {
    let a = self.memory[address_a as usize];
    let b = self.memory[address_b as usize];
    ((a as u16) << 8) | (b as u16)
  }
}

pub mod memory_range {
  pub struct Range {
    pub min: u16,
    pub max: u16,
  }

  // 2KB internal RAM
  pub const RAM: Range = Range {min: 0x0000, max: 0x07FF};
  // These addresses just look up RAM.
  pub const RAM_MIRROR_1: Range = Range {min: 0x0800, max: 0x0FFF};
  pub const RAM_MIRROR_2: Range = Range {min: 0x1000, max: 0x17FF};
  pub const RAM_MIRROR_3: Range = Range {min: 0x1800, max: 0x1FFF};
  pub const PPU_REGISTERS: Range = Range {min: 0x2000, max: 0x2007};
  // These mirror the ppu registers every 8 bytes.
  pub const PPU_REGISTER_MIRRORS: Range = Range {min: 0x2008, max: 0x3FFF};
  pub const APU_AND_IO_REGISTERES: Range = Range {min: 0x4000, max: 0x4017};
  // APU and I/O functionality that is normally disabled. See CPU Test Mode.
  pub const DISABLED_APU_IO_FEATURES: Range = Range {min: 0x4018, max: 0x401F};
  // Cartridge space: PRG ROM, PRG RAM, and mapper registers (See Note)
  // Size: 0xBFE0
  pub const CARTRIDGE_SPACE: Range = Range {min: 0x4020, max: 0xFFFF};
}
