/// The PPU is a picture processing unit. It generates 240 lines of pixels.
/// It has its own address space, consisting of 10kb of memory (possibly more with
/// memory mappers). 8 kilobytes of ROM or RAM on the Game Pak, that contained tiles.
/// Then 2kb for maps and other things.
use crate::bus::SharedBus;

enum PpuRegister {
  /// PPU control register - Write only
  Ctrl = 0x2000,
  /// PPU mask register - Write only
  Mask = 0x2001,
  /// PPU status register - Read only
  Status = 0x2002,
  /// OAM address port - Write
  Oam = 0x2003,
  /// OAM read/write
  OamData = 0x2004,
  /// Scroll - write x2
  Scroll = 0x2005,
  /// Address - write x2
  Address = 0x2006,
  /// VRAM read/write data register
  Data = 0x2007,
}

/// PPU control register
/// Controller ($2000) > write
///
/// https://wiki.nesdev.com/w/index.php/PPU_registers#PPUCTRL
///
/// VPHB SINN
/// |||| ||||
/// |||| ||++- Base nametable address
/// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
/// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
/// |||| |     (0: add 1, going across; 1: add 32, going down)
/// |||| +---- Sprite pattern table address for 8x8 sprites
/// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
/// |||+------ Background pattern table address (0: $0000; 1: $1000)
/// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
/// |+-------- PPU master/slave select
/// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
/// +--------- Generate an NMI at the start of the
///            vertical blanking interval (0: off; 1: on)
enum PpuCtrl {
  V = 0b1000_0000,
  P = 0b0100_0000,
  H = 0b0010_0000,
  B = 0b0001_0000,
  S = 0b0000_1000,
  I = 0b0000_0100,
  N = 0b0000_0011,
}

/// This register controls the rendering of sprites and backgrounds, as well as colour effects.
///
/// https://wiki.nesdev.com/w/index.php/PPU_registers#PPUMASK
///
/// 7  bit  0
/// ---- ----
/// BGRs bMmG
/// |||| ||||
/// |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
/// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
/// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
/// |||| +---- 1: Show background
/// |||+------ 1: Show sprites
/// ||+------- Emphasize red
/// |+-------- Emphasize green
/// +--------- Emphasize blue
enum PpuMask {
  Blue = 0b1000_0000,
  Green = 0b0100_0000,
  Red = 0b0010_0000,
  ShowSprites = 0b0001_0000,
  ShowBackground = 0b0000_1000,
  ShowLeftmostSprites = 0b0000_0100,
  ShowLeftmostBackground = 0b0000_0010,
  GrayScale = 0b0000_0001,
}

/// This register reflects the state of various functions inside the PPU. It is often
/// used for determining timing. To determine when the PPU has reached a given pixel of
/// the screen, put an opaque (non-transparent) pixel of sprite 0 there.
///
/// 7  bit  0
/// ---- ----
/// VSO. ....
/// |||| ||||
/// |||+-++++- Least significant bits previously written into a PPU register
/// |||        (due to register not being updated for this address)
/// ||+------- Sprite overflow. The intent was for this flag to be set
/// ||         whenever more than eight sprites appear on a scanline, but a
/// ||         hardware bug causes the actual behavior to be more complicated
/// ||         and generate false positives as well as false negatives; see
/// ||         PPU sprite evaluation. This flag is set during sprite
/// ||         evaluation and cleared at dot 1 (the second dot) of the
/// ||         pre-render line.
/// |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
/// |          a nonzero background pixel; cleared at dot 1 of the pre-render
/// |          line.  Used for raster timing.
/// +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
///            Set at dot 1 of line 241 (the line *after* the post-render
///            line); cleared after reading $2002 and at dot 1 of the
///            pre-render line.
enum PpuStatus {
  VerticalBlank = 0b1000_0000,
  SpriteHit = 0b0100_0000,
  SpriteOverflow = 0b0010_0000,
}

pub struct Ppu {
  bus: SharedBus,
}

impl Ppu {
  pub fn new(bus: SharedBus) -> Ppu {
    Ppu { bus }
  }

  fn get_register(&self, register: PpuRegister) -> u8 {
    self.bus.borrow().read_u8(register as u16)
  }

  fn set_register(&self, register: PpuRegister, value: u8) {
    self.bus.borrow_mut().set_u8(register as u16, value);
  }

  fn get_register_flag(&self, register: PpuRegister, flag: u8) -> bool {
    self.get_register(register) & flag == flag
  }

  fn get_base_name_table(&self) -> u16 {
    match self.get_register(PpuRegister::Ctrl) & (PpuCtrl::N as u8) {
      0 => 0x2000,
      1 => 0x2400,
      2 => 0x2800,
      3 => 0x2C00,
      _ => panic!("Getting the base name table failed."),
    }
  }
}
