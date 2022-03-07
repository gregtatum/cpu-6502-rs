use super::constants::memory_range;
use crate::mappers::Mapper;
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
    // Includes the zero page, stack, and ram.
    //
    // $2000 |-------------------------|-------------------------| $2000
    //       |   Mirrors $0000 - $07FF |                         |
    // $0800 |-------------------------|                         |
    //       |   General RAM           |                         |
    // $0200 |-------------------------|           RAM           |
    //       |   Stack                 |                         |
    // $0100 |-------------------------|                         |
    //       |   Zero Page             |                         |
    // $0000 |-------------------------|-------------------------| $0000
    ram: [u8; memory_range::RAM.end as usize],
    cartridge: Box<dyn Mapper>,
}

impl Bus {
    pub fn new_shared_bus(cartridge: Box<dyn Mapper>) -> Rc<RefCell<Bus>> {
        Rc::new(RefCell::new(Bus {
            // Little endian memory store, 2 kilobytes in size.
            ram: [0; memory_range::RAM.end as usize],
            cartridge,
        }))
    }

    // The NES address range is larger than the actual bits that are pointed
    // at. This function maps the address to the actual bit range.
    fn map_ram_address(&self, address: u16) -> u16 {
        if address < memory_range::RAM.end {
            // $0000-$07FF  $0800  2KB internal RAM
            // $0800-$0FFF  $0800  Mirrors of $0000-$07FF
            // $1000-$17FF  $0800
            // $1800-$1FFF  $0800
            return memory_range::RAM_ACTUAL.mask() & address;
        }

        if address < memory_range::PPU.end {
            // $2000-$2007  $0008  NES PPU registers
            // $2008-$3FFF  $1FF8  Mirrors of $2000-2007 (repeats every 8 bytes)
            return memory_range::PPU.start + (memory_range::PPU_ACTUAL.mask() & address);
        }

        address
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        if let Some(value) = self.cartridge.read_cpu(address) {
            return value;
        }
        self.ram[self.map_ram_address(address) as usize]
    }

    pub fn read_u16(&self, address_before: u16) -> u16 {
        let address = self.map_ram_address(address_before);
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
        self.ram[self.map_ram_address(address) as usize] = value;
    }

    pub fn set_u16(&mut self, address: u16, value: u16) {
        let [le, be] = value.to_le_bytes();
        let mapped_address = self.map_ram_address(address) as usize;
        self.ram[mapped_address] = le;
        self.ram[mapped_address + 1] = be;
    }
}
