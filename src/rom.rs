use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
pub enum Mirroring {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub enum TvSystem {
    NTSC,
    PAL,
    DualCompatible,
}

#[derive(Debug)]
pub struct Header {
    pub prg_rom_bytes: u32,
    pub prg_rom_banks: u8,
    pub character_rom_bytes: u32,
    pub character_rom_banks: u8,
    pub mirroring: Mirroring,
    pub persistent_memory: bool,
    pub has_trainer: bool,
    pub four_screen_vram: bool,
    pub mapping_number: u8,
    pub vs_unisystem: bool,
    pub playchoice_10: bool,
    pub nes_2_0: bool,
    pub prg_ram_size: u32,
    pub tv_system_rarely_used: TvSystem,
    pub tv_system: TvSystem,
}

pub enum ROMLoadError {
    IoError(io::Error),
    Message(&'static str),
}

impl From<io::Error> for ROMLoadError {
    fn from(error: io::Error) -> Self {
        ROMLoadError::IoError(error)
    }
}

impl From<&'static str> for ROMLoadError {
    fn from(string: &'static str) -> Self {
        ROMLoadError::Message(string)
    }
}

struct Byte {
    value: u8,
}

impl Byte {
    /// Get the bit where 0 is the least significant bit, and 7 is the most
    fn bit(&self, n: u8) -> bool {
        (0b0000_0001 << n) & self.value != 0
    }
}

pub struct ROM {
    pub header: Header,
    pub program_rom: Vec<u8>,
    pub character_rom: Vec<u8>,
    // These can safely be ignored.
    // http://forums.nesdev.com/viewtopic.php?t=3657
    // NES trainers are 512 bytes of code which is loaded into $7000 before the game
    // starts. Famicom copiers used them to hold code to translate mapper writes into
    // the copier's own mapper system. The games were altered so that instead of
    // writing to the original mapper, they jumped to a subroutine in the trainer.
    // They probably aren't necessary to emulate today now that we have non-hacked
    // ROM dumps of all games. There might be some old hacks which use them because
    // the hackers couldn't allocate static space in the ROM for their new code.
    pub trainer: Option<Vec<u8>>,
    pub title: Option<String>,
}

impl ROM {
    /// https://wiki.nesdev.com/w/index.php/INES
    pub fn load_ines_file(path: &Path) -> Result<ROM, ROMLoadError> {
        let mut file = File::open(path)?;
        let header_bytes = read_bytes(&mut file, 16)?;
        let header = process_header(&header_bytes[..])?;

        let trainer = if header.has_trainer {
            eprintln!("A trainer was found when loading the ROM. This will be ignored.");
            Some(read_bytes(&mut file, 512)?)
        } else {
            None
        };

        let program_rom = read_bytes(&mut file, header.prg_rom_bytes as usize)?;
        let character_rom = read_bytes(&mut file, header.character_rom_bytes as usize)?;

        let _play_choice_inst_rom = if header.playchoice_10 {
            eprintln!("Found play choice data in the NES file, this is not supported.");
            Some(read_bytes(&mut file, 8192))
        } else {
            None
        };

        let _play_choice_prom = if header.playchoice_10 {
            // (16 bytes Data, 16 bytes CounterOut)
            Some(read_bytes(&mut file, 32))
        } else {
            None
        };

        // Some ROM-Images additionally contain a 128-byte (or sometimes 127-byte) title
        // at the end of the file.
        let mut title_bytes = Vec::new();
        file.read_to_end(&mut title_bytes)?;
        let title = if title_bytes.is_empty() {
            None
        } else {
            let mut title = String::new();
            for ch in &title_bytes {
                if *ch == 0 {
                    break;
                }
                title.push(*ch as char);
            }

            Some(String::from(title.trim()))
        };

        Ok(ROM {
            program_rom,
            character_rom,
            header,
            trainer,
            title,
        })
    }
}

fn process_header(header: &[u8]) -> Result<Header, ROMLoadError> {
    eprintln!("Header: {:?}", header[0..4].to_vec());
    // 0-3: Constant $4E $45 $53 $1A ("NES" followed by MS-DOS end-of-file)
    if header[0..4] != [0x4E, 0x45, 0x53, 0x1A] {
        return Err(ROMLoadError::Message(
            "This does not appear to be an NES file.",
        ));
    }

    // 4: Size of PRG ROM in 16 KB units
    let prg_rom_banks = header[4];
    let prg_rom_bytes: u32 = prg_rom_banks as u32 * 16 * 1024;

    // 5: Size of CHR ROM in 8 KB units (Value 0 means the board uses CHR RAM)
    let character_rom_banks: u8 = header[5];
    let character_rom_bytes: u32 = character_rom_banks as u32 * 8 * 1024;

    let flag6 = Byte { value: header[6] };
    let flag7 = Byte { value: header[7] };
    let flag8 = Byte { value: header[8] };
    let flag9 = Byte { value: header[9] };
    let flag10 = Byte { value: header[10] };

    // 6: Flags 6 - Mapper, mirroring, battery, trainer
    //
    // 76543210
    // ||||||||
    // |||||||+- Mirroring: 0: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)
    // |||||||              1: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
    // ||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
    // |||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
    // ||||+---- 1: Ignore mirroring control or above mirroring bit; instead provide four-screen VRAM
    // ++++----- Lower nybble of mapper number
    let mirroring = if flag6.bit(0) {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    };
    let persistent_memory = flag6.bit(1);
    let has_trainer = flag6.bit(2);
    let four_screen_vram = flag6.bit(3);
    let mapping_number_lower = flag6.value >> 4; // Move 0bXXXX_0000 to 0b0000_XXXX

    // 7: Flags 7 - Mapper, VS/Playchoice, NES 2.0
    // 76543210
    // ||||||||
    // |||||||+- VS Unisystem
    // ||||||+-- PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
    // ||||++--- If equal to 2, flags 8-15 are in NES 2.0 format
    // ++++----- Upper nybble of mapper number
    let vs_unisystem = flag7.bit(0);
    let playchoice_10 = flag7.bit(1);
    let nes_2_0 = flag7.bit(3) && !flag7.bit(2);
    let mapping_number_upper = flag7.value & 0b1111_0000; // Mask the upper bits.
    let mapping_number = mapping_number_upper | mapping_number_lower;

    if nes_2_0 {
        return Err("NES 2.0 format is not currently supported".into());
    }

    // 8: Flags 8 - PRG-RAM size (rarely used extension)
    let prg_ram_size = flag8.value.max(1) as u32 * 8 * 1024;

    // 9: Flags 9 - TV system (rarely used extension)
    // 76543210
    // ||||||||
    // |||||||+- TV system (0: NTSC; 1: PAL)
    // +++++++-- Reserved, set to zero
    let tv_system_rarely_used = if flag9.bit(0) {
        TvSystem::PAL
    } else {
        TvSystem::NTSC
    };

    // 10: Flags 10 - TV system, PRG-RAM presence (unofficial, rarely used extension)
    // 76543210
    // ||  ||
    // ||  ++- TV system (0: NTSC; 2: PAL; 1/3: dual compatible)
    // |+----- PRG RAM ($6000-$7FFF) (0: present; 1: not present)
    // +------ 0: Board has no bus conflicts; 1: Board has bus conflicts
    let tv_system = match flag10.value & 0b0000_1100 >> 2 {
        0 => TvSystem::NTSC,
        2 => TvSystem::PAL,
        _ => TvSystem::DualCompatible,
    };

    // 11-15: Unused padding (should be filled with zero, but some rippers put their name across bytes 7-15)

    Ok(Header {
        prg_rom_banks,
        prg_rom_bytes,
        character_rom_banks,
        character_rom_bytes,
        mirroring,
        persistent_memory,
        has_trainer,
        four_screen_vram,
        mapping_number,
        vs_unisystem,
        playchoice_10,
        nes_2_0,
        prg_ram_size,
        tv_system_rarely_used,
        tv_system,
    })
}

fn read_bytes(file: &mut File, size: usize) -> Result<Vec<u8>, io::Error> {
    let mut vec = Vec::new();
    let read_bytes = file.take(size as u64).read_to_end(&mut vec)?;
    assert_eq!(size, read_bytes);
    Ok(vec)
}
