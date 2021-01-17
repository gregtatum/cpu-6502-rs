use super::constants::{memory_range, InterruptVectors};
use std::cell::RefCell;
use std::rc::Rc;

/**
 * The bus contains the actual memory used by the NES. This can
 * be referenced and used across modules. In order to allow
 * this to be a shared mutable piece of memory, wrap it in a
 * reference counted refcell. This incurs realtime costs to use
 * the data, but creates a direct way to have shared memory in
 * a way that Rust can compile.
 */
pub type SharedBus = Rc<RefCell<Bus>>;

pub struct Bus {
    memory: [u8; 0xFFFF],
}

impl Bus {
    pub fn new_shared_bus() -> Rc<RefCell<Bus>> {
        Rc::new(RefCell::new(Bus {
            // Little endian memory store, 2 kilobytes in size.
            memory: [0; 0xFFFF],
        }))
    }

    // The NES address range is larger than the actual bits that are pointed
    // at. This function maps the address to the actual bit range.
    fn map_address(&self, address: u16) -> u16 {
        if address <= memory_range::RAM.max {
            // $0000-$07FF  $0800  2KB internal RAM
            // $0800-$0FFF  $0800  Mirrors of $0000-$07FF
            // $1000-$17FF  $0800
            // $1800-$1FFF  $0800
            return memory_range::RAM_ACTUAL.size() & address;
        }

        if address <= memory_range::PPU.max {
            // $2000-$2007  $0008  NES PPU registers
            // $2008-$3FFF  $1FF8  Mirrors of $2000-2007 (repeats every 8 bytes)
            return memory_range::PPU.min + (memory_range::PPU_ACTUAL.size() & address);
        }

        address
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        self.memory[self.map_address(address) as usize]
    }

    pub fn read_u16(&self, address: u16) -> u16 {
        let address = self.map_address(address);
        // Recreate the bug of reading a u16 over a page wraps it back
        // to the beginning of the page.
        let [address_low, address_high] = address.to_le_bytes();
        let address2 = u16::from_le_bytes([address_low.wrapping_add(1), address_high]);
        self.read_u16_disjoint(address, address2)
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
        self.memory[self.map_address(address) as usize] = value;
    }

    pub fn set_u16(&mut self, address: u16, value: u16) {
        let [le, be] = value.to_le_bytes();
        let mapped_address = self.map_address(address) as usize;
        self.memory[mapped_address] = le;
        self.memory[mapped_address + 1] = be;
    }

    pub fn load_program(&mut self, program: &[u8]) {
        if program.len() > memory_range::CARTRIDGE_SPACE.size() as usize {
            panic!(
                "Attempting to load a program that is larger than the cartridge space."
            );
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
