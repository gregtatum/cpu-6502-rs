use super::bus::Bus;

const CLOCK_SPEED: f64 = 1.789773; // Mhz
const CLOCK_DIVISOR: u32 = 12;  //Mhz
// Emulator authors may wish to emulate the NTSC NES/Famicom CPU at 21441960 Hz
// ((341×262-0.5)×4×60) to ensure a synchronised/stable 60 frames per second.
const MASTER_CLOCK_FREQUENCY: f64 = 21.441960; //Mhz
// This is the true frequency:
// const MASTER_CLOCK_FREQUENCY: f64 = 21.477272; //Mhz
const COLOR_SUBCARRIER_FREQUENCY: f64 = 3.57954545; // Mhz

pub enum StatusFlag {
  Carry            = 0b00000001,
  Zero             = 0b00000010,
  InterruptDisable = 0b00000100,
  Decimal          = 0b00001000,
  NoEffect1        = 0b00010000,
  NoEffect2        = 0b00100000,
  Overflow         = 0b01000000,
  Negative         = 0b10000000,
}

pub enum Mode {
  Absolute,   // ABS
  AbsoluteX, // ABX
  AbsoluteY, // ABY
  Immediate, // IMM
  IMP,
  Indirect, // IND
  IndirectX, // IZX
  IndirectY, // IZY
  REL,
  ZeroPage,  // ZP
  ZeroPageX, // ZPX
  ZeroPageY, // ZPY
  None,
}

pub enum ExtraCycle {
  None,
  PageBoundary,
  IfTaken,
}

pub struct Mos6502Cpu {
  // The bus is what
  bus: Bus,
  // "A" Supports using the status register for carrying, overflow detection, and so on
  accumulator: u8,
  // Used for several addressing modes  They can be used as loop counters easily, using
  // INC/DEC and branch instructions. Not being the accumulator, they have limited
  // addressing modes themselves when loading and saving.
  // "X"
  x_index: u8,
  y_index: u8,

  // The 2-byte program counter PC supports 65536 direct (unbanked) memory locations,
  // however not all values are sent to the cartridge. It can be accessed either by
  // allowing CPU's internal fetch logic increment the address bus, an interrupt
  // (NMI, Reset, IRQ/BRQ), and using the RTS/JMP/JSR/Branch instructions.
  // "PC"
  program_counter: u16,

  // S is byte-wide and can be accessed using interrupts, pulls, pushes, and transfers.
  stack_pointer: u8,

  // P has 6 bits used by the ALU but is byte-wide. PHP, PLP, arithmetic, testing,
  // and branch instructions can access this register.
  //
  // http://wiki.nesdev.com/w/index.php/Status_flags
  //
  //   7  bit  0
  // ---- ----
  // NVss DIZC
  // |||| ||||
  // |||| |||+- Carry
  // |||| ||+-- Zero
  // |||| |+--- Interrupt Disable
  // |||| +---- Decimal
  // ||++------ No CPU effect, see: the B flag
  // |+-------- Overflow
  // +--------- Negative

  status_register: u8,

  /// The number of cycles that were done while operating on an instruction. The
  /// emulator will then need to wait the proper amount of time after executing
  /// the commands.
  cycles: u8,
}

impl Mos6502Cpu {
  fn new(bus: Bus) -> Mos6502Cpu {
    Mos6502Cpu {
      bus,
      accumulator: 0, // A
      x_index: 0, // X
      y_index: 0, // Y
      program_counter: 0, // PC -- TODO - Figure out this value
      stack_pointer: 0xFD, // S
      status_register: 0x34, // P
      cycles: 0,
    }
  }

  fn increment_pc(&mut self, byte_count: u16) -> u16 {
    self.program_counter += byte_count;
    self.program_counter
  }

  /// Handle individual instructions
  /// https://github.com/munshkr/nesasm/blob/master/docs/cpu_inst.txt
  fn step(&mut self) {
    let opcode = self.increment_pc(1);
    match opcode {
      0x01 => self.ora(Mode::IndirectX, 6, ExtraCycle::None),
      0x05 => self.ora(Mode::ZeroPage, 3, ExtraCycle::None),
      0x09 => self.ora(Mode::Immediate, 2, ExtraCycle::None),
      0x0d => self.ora(Mode::Absolute, 4, ExtraCycle::None),
      0x11 => self.ora(Mode::IndirectY, 5, ExtraCycle::PageBoundary),
      0x15 => self.ora(Mode::ZeroPageX, 4, ExtraCycle::None),
      0x19 => self.ora(Mode::AbsoluteY, 4, ExtraCycle::PageBoundary),
      0x1d => self.ora(Mode::AbsoluteX, 4, ExtraCycle::PageBoundary),
      _ => panic!("Unhandled opcode {}", opcode),
    }
  }

  fn read16(&mut self, mode: Mode, extra_cycle: ExtraCycle) -> u16 {
    match mode {
      Mode::Absolute => {
        match extra_cycle {
          ExtraCycle::PageBoundary => panic!("opcode attempted to set an extra cycle, but it wasn't handled."),
          _ => {}
        };
        self.increment_pc(2);
        self.bus.read_word(self.program_counter)
      },
      Mode::AbsoluteX => {
        let address_1 = self.increment_pc(2) + self.x_index as u16;

        // Check to see if a page boundary is being crossed, if it is, add to the cycles
        // taken if needed.
        match extra_cycle {
          ExtraCycle::PageBoundary => {
            let address_2 = address_1 + 1;
            if 0xff00 & address_1 != 0xff00 & address_2 {
              self.cycles += 1;
            }
          },
          _ => {}
        };
        self.bus.read_word(address_1)
      },
      Mode::AbsoluteY => {
        let address_1 = self.increment_pc(2) + self.y_index as u16;

        // Check to see if a page boundary is being crossed, if it is, add to the cycles
        // taken if needed.
        match extra_cycle {
          ExtraCycle::PageBoundary => {
            let address_2 = address_1 + 1;
            if 0xff00 & address_1 != 0xff00 & address_2 {
              self.cycles += 1;
            }
          },
          _ => {}
        };
        self.bus.read_word(address_1)
      },
      Mode::Immediate => 0,
      Mode::IMP => 0,
      Mode::Indirect => 0,
      Mode::IndirectX => 0,
      Mode::IndirectY => 0,
      Mode::REL => 0,
      Mode::ZeroPage => 0,
      Mode::ZeroPageX => 0,
      Mode::ZeroPageY => 0,
      Mode::None => panic!("Attempting to read with no mode set."),
    }
  }

  /// Apply the logical "or" operator on the accumulator.
  /// A:=A or {adr}
  /// Flags: NZ
  fn ora(&mut self, mode: Mode, cycles: u8, extra_cycle: ExtraCycle) {
    self.cycles += cycles;
    self.accumulator |= self.read16(mode, extra_cycle) as u8;
    self.set_status_flag(StatusFlag::Zero, self.accumulator == 0);
    self.set_status_flag(StatusFlag::Negative, self.accumulator == 0b1000_0000);
  }

  fn set_status_flag(&mut self, status_flag: StatusFlag, value: bool) {
    if value {
      self.status_register |= status_flag as u8;
    } else {
      self.status_register &= !(status_flag as u8);
    }
  }
}
