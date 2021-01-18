use crate::constants::{memory_range, InterruptVectors};

use super::Mapper;

const PROGRAM_SIZE: usize = 0x8000;

/// This is not an official part of the NES, it's a simple way to load
/// up and test custom programs. Once the mappers get more robust, it may
/// be worth removing this in favor of the official mappers.
pub struct SimpleProgram {
    program: [u8; 0x8000],
}

impl SimpleProgram {
    pub fn new() -> SimpleProgram {
        SimpleProgram {
            program: [0; 0x8000],
        }
    }

    pub fn load(program: &[u8]) -> SimpleProgram {
        let mut mapper = SimpleProgram::new();
        if mapper.program.len() > PROGRAM_SIZE {
            panic!(
                "Attempting to load a program that is larger than the SimpleProgram cartridge space."
            );
        }

        // Copy the memory into the buffer.
        for (index, value) in program.iter().enumerate() {
            mapper.program[index] = *value;
        }

        let [low, high] = (memory_range::PRG_ROM.start as u16).to_le_bytes();
        let reset_byte_add = (InterruptVectors::ResetVector as u16 & 0x7fff) as usize;

        // Set the reset vector to the first byte of the program.
        mapper.program[reset_byte_add] = low;
        mapper.program[reset_byte_add + 1] = high;
        mapper
    }
}

impl Mapper for SimpleProgram {
    fn read_cpu(&self, addr: u16) -> Option<u8> {
        match addr {
            0x8000..=0xffff => Some(self.program[(addr & 0x7fff) as usize]),
            _ => None,
        }
    }

    fn write_cpu(&mut self, addr: u16, _value: u8) -> bool {
        addr >= 0x8000
    }
}
