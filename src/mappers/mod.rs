mod simple;

// Re-export the mappers.
pub use simple::*;

pub trait Mapper {
    fn read_cpu(&self, addr: u16) -> Option<u8>;
    fn write_cpu(&mut self, addr: u16, value: u8) -> bool;
}
