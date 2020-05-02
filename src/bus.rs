use super::constants::{memory_range, InterruptVectors};
pub struct Bus {
  memory: [u8; 0xFFFF],
}

impl Bus {
  pub fn new() -> Bus {
    Bus {
      // Little endian memory store.
      memory: [0; 0xFFFF],
    }
  }

  pub fn read_u8(&self, address: u16) -> u8 {
    self.memory[address as usize]
  }

  pub fn read_u16(&self, address: u16) -> u16 {
    self.read_u16_disjoint(address, address + 1)
  }

  /**
   * Words are little endian. Use rust's built-in features rather than relying on
   * bit shifting.
   *
   * e.g.
   * Little-Endian:  0x1000  00 10
   *    Big-Endian:  0x1000  10 00
   */
  pub fn read_u16_disjoint(&self, address_a: u16, address_b: u16) -> u16 {
    let a = self.read_u8(address_a);
    let b = self.read_u8(address_b);
    u16::from_le_bytes([a, b])
  }

  pub fn set_u8(&mut self, address: u16, value: u8) {
    self.memory[address as usize] = value;
  }

  pub fn set_u16(&mut self, address: u16, value: u16) {
    let [a, b] = value.to_le_bytes();
    self.memory[address as usize] = a;
    self.memory[address as usize + 1] = b;
  }

  pub fn load_program(&mut self, program: &Vec<u8>) {
    if program.len() > memory_range::CARTRIDGE_SPACE.size() as usize {
      panic!("Attempting to load a program that is larger than the cartridge space.");
    }

    // Copy the memory into the buffer.
    for (index, value) in program.iter().enumerate() {
      self.memory[memory_range::CARTRIDGE_SPACE.min as usize + index] = *value;
    }

    // TODO - For now set the start of the execution to the beginning byte of
    // the program.
    self.set_u16(
      InterruptVectors::ResetVector as u16,
      memory_range::CARTRIDGE_SPACE.min,
    );
  }
}
