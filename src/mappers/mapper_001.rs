use crate::rom::{Header, ROMLoadError};

use super::Mapper;

// The Nintendo MMC1 is a mapper ASIC used in Nintendo's SxROM and NES-EVENT
// Game Pak boards. Most common SxROM boards are assigned to iNES Mapper 1.
// This chip first appeared in the April of 1987.
// https://wiki.nesdev.com/w/index.php/MMC1

// CPU $6000-$7FFF: 8 KB PRG RAM bank, (optional)
// CPU $8000-$BFFF: 16 KB PRG ROM bank, either switchable or fixed to the first bank
// CPU $C000-$FFFF: 16 KB PRG ROM bank, either fixed to the last bank or switchable
// PPU $0000-$0FFF: 4 KB switchable CHR bank
// PPU $1000-$1FFF: 4 KB switchable CHR bank

// Header {
//   prg_rom_bytes: 0x10000, (64k)
//   prg_rom_banks: 4,
//   character_rom_bytes: 16384,
//   character_rom_banks: 2,
//   mirroring: Horizontal,
//   persistent_memory: true,
//   has_trainer: false,
//   four_screen_vram: false,
//   mapping_number: 1,
//   vs_unisystem: false,
//   playchoice_10: false,
//   nes_2_0: false,
//   prg_ram_size: 0x2000,
//   tv_system_rarely_used: NTSC,
//   tv_system: PAL,
// }

const RAM_SIZE: usize = 0x2000; // 8kb
const RAM_MASK: u16 = 0x1fff;
const BANK_SIZE: usize = 0x4000; // 16kb
const BANK_MASK: u16 = 0x3fff;

type ProgramBank = [u8; BANK_SIZE];

pub struct Mapper001 {
    ram: Option<Box<[u8; RAM_SIZE]>>,
    program_rom: Vec<u8>,
    last_bank: u8,
    // Only use the last 5 bits.
    // 0b0000_0000
    //      ^ ^^^^
    shift_register: u8,
    shift_register_address: u16,
    shift_register_bits_shifted: u8,

    // Registers are 5 bits, and loaded via the shift register.
    // Register writing range: $8000-0xffff

    // Write to: $8000-$9FFF
    // -----
    // CPPMM
    // |||||
    // |||++- Mirroring (0: one-screen, lower bank; 1: one-screen, upper bank;
    // |||               2: vertical; 3: horizontal)
    // |++--- PRG ROM bank mode (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
    // |                         2: fix first bank at $8000 and switch 16 KB bank at $C000;
    // |                         3: fix last bank at $C000 and switch 16 KB bank at $8000)
    // +----- CHR ROM bank mode (0: switch 8 KB at a time; 1: switch two separate 4 KB banks)
    control_register: u8,

    // Write to: $A000-$BFFF
    // CCCCC
    // |||||
    // +++++- Select 4 KB or 8 KB CHR bank at PPU $0000 (low bit ignored in 8 KB mode)
    //
    // MMC1 can do CHR banking in 4KB chunks. Known carts with CHR RAM have 8 KiB,
    // so that makes 2 banks. RAM vs ROM doesn't make any difference for address lines.
    // For carts with 8 KiB of CHR (be it ROM or RAM), MMC1 follows the common behavior
    // of using only the low-order bits: the bank number is in effect ANDed with 1.
    chr_bank_0_register: u8,

    // Write to: $C000-$DFFF
    // 4bit0
    // -----
    // CCCCC
    // |||||
    // +++++- Select 4 KB CHR bank at PPU $1000 (ignored in 8 KB mode)
    chr_bank_1_register: u8,

    // $E000-$FFFF
    //
    // The high bit does not select a PRG ROM bank. MMC1 with 512K was supported by
    // re-using a line from the CHR banking controls.
    //
    // 4bit0
    // -----
    // RPPPP
    // |||||
    // |++++- Select 16 KB PRG ROM bank (low bit ignored in 32 KB mode)
    // +----- PRG RAM chip enable (0: enabled; 1: disabled; ignored on MMC1A)
    prg_bank_register: u8,
}

impl Mapper001 {
    fn new(header: &Header) -> Result<Mapper001, ROMLoadError> {
        // The RAM is optional, but is always sized 0x2000.
        let ram = match header.prg_ram_size {
            0 => None,
            0x2000 => Some(Box::new([0; 0x2000])),
            _ => {
                return Err(
                    "The ROM had the incorrect sized RAM for a Mapper 001.".into()
                );
            }
        };

        Ok(Mapper001 {
            ram,
            program_rom: vec![0; header.prg_rom_bytes as usize],
            last_bank: header.prg_rom_banks - 1,
            shift_register: 0,
            shift_register_address: 0,
            shift_register_bits_shifted: 0,
            control_register: 0,
            chr_bank_0_register: 0,
            chr_bank_1_register: 0,
            prg_bank_register: 0,
        })
    }

    fn get_prg_rom_bank_mode(&self) -> u8 {
        self.control_register & 0b0000_1100 >> 2
    }

    fn get_prg_rom_bank(&self) -> u8 {
        self.prg_bank_register & 0b0000_1111
    }

    fn get_prg_rom_bank_32(&self) -> u8 {
        self.prg_bank_register & 0b0000_1110
    }

    fn read_prg_bank(&self, bank: u8, absolute_addr: u16) -> u8 {
        *self
            .program_rom
            .get(((bank as u16) * 0x4000 + (absolute_addr & BANK_MASK)) as usize)
            .expect("The mapper is going out of bounds")
    }
}

type Memory = [u8; 0xFFFF];

impl Mapper for Mapper001 {
    fn read_cpu(&self, addr: u16) -> Option<u8> {
        match addr {
            // PRG RAM bank - 8 KB (optional)
            0x6000..=0x7fff => {
                match self.ram {
                    Some(ref ram) => {
                        // Map $6000-$7FFF to $0000-$1FFF
                        Some(ram[(addr & RAM_MASK) as usize])
                    }
                    None => None,
                }
            }

            // Map memory for the PRG-ROM Lower Bank.
            0x8000..=0xbfff => match self.get_prg_rom_bank_mode() {
                0 | 1 => {
                    // Shift based off of 32kb pages. The lowest bit is set to 0.
                    Some(self.read_prg_bank(self.get_prg_rom_bank() & 0b1111_1110, addr))
                }
                2 => {
                    // Fixed to the first bank.
                    Some(self.read_prg_bank(0, addr))
                }
                3 => {
                    // Switch 16 KB bank
                    Some(self.read_prg_bank(self.get_prg_rom_bank(), addr))
                }
                _ => panic!("Unmatched branch for the PRG ROM bank mode."),
            },

            // Map memory for the PRG-ROM Upper Bank.
            0xc000..=0xffff => match self.get_prg_rom_bank_mode() {
                0 | 1 => {
                    // Shift based off of 32kb pages. The lowest bit is set to 1.
                    Some(self.read_prg_bank(self.get_prg_rom_bank() | 0b0000_0001, addr))
                }
                2 => {
                    // Switch 16 KB bank.
                    Some(self.read_prg_bank(self.get_prg_rom_bank(), addr))
                }
                3 => {
                    // Fixed to the last bank.
                    Some(self.read_prg_bank(self.last_bank, addr))
                }
                _ => panic!("Unmatched branch for the PRG ROM bank mode."),
            },
            _ => {
                // Not handled by the mapper.
                None
            }
        }
    }

    fn write_cpu(&mut self, addr: u16, value: u8) -> bool {
        match addr {
            0x6000..=0x7fff => {
                // 8 KB PRG RAM bank, (optional)
                if let Some(ref mut ram) = self.ram {
                    // Map $6000-$7FFF to $0000-$1FFF
                    ram[(addr & RAM_MASK) as usize] = value;
                }
            }
            0x8000..=0xffff => {
                // Writing a value with bit 7 set ($80 through $FF) to any address in
                // $8000-$FFFF clears the shift register to its initial state

                // Internal registers are 5 bits wide.  Meaning to complete a "full" write, games must write to a register 5
                // times (low bit first).  This is usually accomplished with something like the following:

                //    LDA value_to_write
                //    STA $9FFF    ; 1st bit written
                //    LSR A
                //    STA $9FFF    ; 2nd bit written
                //    LSR A
                //    STA $9FFF    ; 3rd bit written
                //    LSR A
                //    STA $9FFF    ; 4th bit written
                //    LSR A
                //    STA $9FFF    ; final 5th bit written -- full write is complete

                // Shift register address changed, invalidate it.
                if addr != self.shift_register_address {
                    self.shift_register = 0;
                    self.shift_register_address = addr;
                    self.shift_register_bits_shifted = 0;
                }

                let bit_7 = 0b1000_0000;

                if value & bit_7 == bit_7 {
                    self.shift_register = 0;
                    self.shift_register_bits_shifted = 0;
                } else {
                    let bit_0 = value & 0b0000_0001;

                    // Shift the register, and place our new bit value into the it at bit 4.
                    self.shift_register =
                        (self.shift_register >> 1) & 0b1110_1111 | (bit_0 << 4);

                    if self.shift_register_bits_shifted == 5 {
                        // The shift register is full, save it to the appropriate register.
                        match addr {
                            0x8000..=0x9fff => {
                                self.control_register = self.shift_register;
                            }
                            0xa000..=0xbfff => {
                                self.chr_bank_0_register = self.shift_register;
                            }
                            0xc000..=0xdfff => {
                                self.chr_bank_1_register = self.shift_register;
                            }
                            0xe000..=0xffff => {
                                self.prg_bank_register = self.shift_register;
                            }
                            _ => panic!("Unexpected address range found."),
                        }
                        self.shift_register = 0;
                        self.shift_register_bits_shifted = 0
                    }
                }
            }
            _ => {
                // Not handled by the mapper.
                return false;
            }
        };
        true
    }
}
