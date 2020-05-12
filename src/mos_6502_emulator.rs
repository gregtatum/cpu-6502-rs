use crate::bus::Bus;
use crate::constants::{memory_range, InterruptVectors};
use crate::opcodes::{Mode, OpCode};
mod opcodes_illegal;
mod opcodes_jump;
mod opcodes_logical;
mod opcodes_move;

use opcodes_illegal::*;
use opcodes_jump::*;
use opcodes_logical::*;
use opcodes_move::*;

// Mhz
const CLOCK_SPEED: f64 = 1.789773;
// Mhz
const CLOCK_DIVISOR: u32 = 12;
// Emulator authors may wish to emulate the NTSC NES/Famicom CPU at 21441960 Hz
// ((341×262-0.5)×4×60) to ensure a synchronised/stable 60 frames per second.
// Mhz
const MASTER_CLOCK_FREQUENCY: f64 = 21.441960;
// This is the true frequency:
// const MASTER_CLOCK_FREQUENCY: f64 = 21.477272;
// Mhz
const COLOR_SUBCARRIER_FREQUENCY: f64 = 3.57954545;

const RESET_STATUS_FLAG: u8 = 0b00110100;

#[rustfmt::skip]
pub enum StatusFlag {
  Carry            = 0b00000001,
  Zero             = 0b00000010,
  InterruptDisable = 0b00000100,
  Decimal          = 0b00001000,
  Break            = 0b00010000,
  Push             = 0b00100000,
  Overflow         = 0b01000000,
  Negative         = 0b10000000,
}

pub enum ExtraCycle {
  None,
  PageBoundary,
  IfTaken,
}

pub struct Mos6502Cpu {
  // The bus is what
  bus: Bus,
  // "A" register - The accumulator. Typical results of operations are stored here.
  // In combination with the status register, supports using the status register for
  // carrying, overflow detection, and so on.
  a: u8,
  /// "X" register.
  /// Used for several addressing modes  They can be used as loop counters easily, using
  /// INC/DEC and branch instructions. Not being the accumulator, they have limited
  /// addressing modes themselves when loading and saving.
  x: u8,
  /// "Y" register.
  y: u8,

  /// "PC" - Program counter.
  /// The 2-byte program counter PC supports 65536 direct (unbanked) memory locations,
  /// however not all values are sent to the cartridge. It can be accessed either by
  /// allowing CPU's internal fetch logic increment the address bus, an interrupt
  /// (NMI, Reset, IRQ/BRQ), and using the RTS/JMP/JSR/Branch instructions.
  /// "PC"
  pc: u16,

  /// "S" - Stack pointer
  ///
  /// The 6502 has hardware support for a stack implemented using a 256-byte array
  /// whose location is hardcoded at page 0x01 (0x0100-0x01FF), using the S register
  /// for a stack pointer.
  ///
  /// The 6502 uses a descending stack (it grows downwards)
  /// https://wiki.nesdev.com/w/index.php/Stack
  s: u8,

  /// "P" - Status register.
  /// P has 6 bits used by the ALU but is byte-wide. PHP, PLP, arithmetic, testing,
  /// and branch instructions can access this register.
  ///
  /// http://wiki.nesdev.com/w/index.php/Status_flags
  ///
  ///   7  bit  0
  /// ---- ----
  /// NVss DIZC
  /// |||| ||||
  /// |||| |||+- Carry
  /// |||| ||+-- Zero
  /// |||| |+--- Interrupt Disable
  /// |||| +---- Decimal
  /// ||++------ No CPU effect, see: the B flag
  /// |+-------- Overflow
  /// +--------- Negative
  p: u8,

  /// The number of cycles that were done while operating on an instruction. The
  /// emulator will then need to wait the proper amount of time after executing
  /// the commands.
  cycles: u8,
}

macro_rules! mode_to_type {
  (abs) => {
    Mode::Absolute
  };
  (abx) => {
    Mode::AbsoluteIndexedX
  };
  (aby) => {
    Mode::AbsoluteIndexedY
  };
  (imm) => {
    Mode::Immediate
  };
  (imp) => {
    Mode::Implied
  };
  (ind) => {
    Mode::Indirect
  };
  (izx) => {
    Mode::IndirectX
  };
  (izy) => {
    Mode::IndirectY
  };
  (rel) => {
    Mode::Relative
  };
  (zp) => {
    Mode::ZeroPage
  };
  (zpx) => {
    Mode::ZeroPageX
  };
  (zpy) => {
    Mode::ZeroPageY
  };
  (non) => {
    Mode::None
  };
}

/// Usage:
/// match_opcode!(opcode, [
///   { 0x00, BRK, non, 7, 0 },
/// ]);
macro_rules! match_opcode {
  (
    $self:expr,
    $opcode:expr,
    [
      $({
        $byte:expr,
        $op_name:ident,
        $addressing_mode:ident,
        $cycles:expr,
        $extra_cycles:expr
      }),*
    ]
  ) => {
      {
          match $opcode {
            $(
                // 0x01 => ora(&mut self, Mode::IndirectX, 6, 0),
                $byte => {
                  $self.cycles += $cycles;
                  $op_name($self, mode_to_type!($addressing_mode), $extra_cycles)
                },
            )*
          }
      }
  };
}

impl Mos6502Cpu {
  fn new(bus: Bus) -> Mos6502Cpu {
    // Go ahead and read the first instruction from the reset vector. If the reset
    // vector is set again, the program will end.
    let pc = bus.read_u16(InterruptVectors::ResetVector as u16);

    Mos6502Cpu {
      bus,
      // Accumulator
      a: 0,
      // X & Y Registers.
      x: 0,
      y: 0,
      // The program counter.
      pc,
      // Stack pointer - It grows down, so initialize it at the top.
      s: 0xFF,
      // Status register
      p: 0b0011_0100,
      cycles: 0,
    }
  }

  /// Read the PC without incrementing.
  fn peek_u8(&mut self) -> u8 {
    self.bus.read_u8(self.pc)
  }

  /// Increment the program counter and read the next u8 value following
  /// the current pc.
  fn next_u8(&mut self) -> u8 {
    let value = self.bus.read_u8(self.pc);
    self.pc += 1;
    value
  }

  /// Increment the program counter and read the next u16 value following
  /// the current pc.
  fn next_u16(&mut self) -> u16 {
    let value = self.bus.read_u16(self.pc);
    self.pc += 2;
    value
  }

  /// This function is useful for testing the emulator. It will only run while the
  /// predicate is true.
  fn run_until<F>(&mut self, predicate: F)
  where
    F: Fn(&Mos6502Cpu) -> bool,
  {
    while !predicate(self) {
      self.tick();
    }
  }

  /// Run the emulator until the "KIL" command is issued.
  fn run(&mut self) {
    while self.peek_u8() != OpCode::KIL as u8 {
      self.tick()
    }
  }

  /// The source for the comments on the modes is coming from:
  /// http://www.emulator101.com/6502-addressing-modes.html
  fn get_operand_address(&mut self, mode: Mode, page_boundary_cycle: u8) -> u16 {
    match mode {
      // Absolute addressing specifies the memory location explicitly in the two bytes
      // following the opcode. So JMP $4032 will set the PC to $4032. The hex for
      // this is 4C 32 40, here 4C is the opcode. The 6502 is a little endian machine,
      // so any 16 bit (2 byte) value is stored with the LSB first. All instructions
      // that use absolute addressing are 3 bytes including the opcode.
      Mode::Absolute => {
        return self.next_u16();
      }
      // Absolute indexing gets the target address by adding the contents of the X or Y
      // register to an absolute address. For example, this 6502 code can be used
      // to fill 10 bytes with $FF starting at address $1009, counting down to
      // address $1000.
      //
      //    LDA #$FF    ; Load 0xff into the A register
      //    LDY #$09    ; Load 0x09 ito the Y register
      //    loop:       ; Create a label
      //    STA $1000,Y ; Store 0xff at address 0x1000 + Y
      //    DEY         ; Decrement Y
      //    BPL loop    ; Loop until Y is 0
      Mode::AbsoluteIndexedX => {
        let base_address = self.next_u16();
        let offset_address = base_address + self.x as u16;
        self.incur_extra_cycle_on_page_boundary(base_address, offset_address, page_boundary_cycle);
        return offset_address;
      }
      Mode::AbsoluteIndexedY => {
        let base_address = self.next_u16();
        let offset_address = base_address + self.y as u16;
        self.incur_extra_cycle_on_page_boundary(base_address, offset_address, page_boundary_cycle);
        return offset_address;
      }
      // These instructions have their data defined as the next byte after the
      // opcode. ORA #$B2 will perform a logical (also called bitwise) of the
      // value B2 with the accumulator. Remember that in assembly when you see
      // a # sign, it indicates an immediate value. If $B2 was written without
      // a #, it would indicate an address or offset.
      Mode::Immediate => {
        // Return the current program counter as the address, but also increment
        // the program counter.
        let address = self.pc;
        self.pc += 1;
        return address;
      }
      // In an implied instruction, the data and/or destination is mandatory for
      // the instruction. For example, the CLC instruction is implied, it is going
      // to clear the processor's Carry flag.
      Mode::Implied => panic!("An implied mode should never be directly activated."),
      // The indirect addressing mode is similar to the absolute mode, but the
      // next u16 is actually a pointer to another address. Use this next address
      // for the operation.
      Mode::Indirect => {
        let address = self.next_u16();
        return self.bus.read_u16(address);
      }
      Mode::IndirectX => self.next_u8().wrapping_add(self.x) as u16,
      Mode::IndirectY => self.next_u8().wrapping_add(self.y) as u16,
      // Relative addressing on the 6502 is only used for branch operations. The byte
      // after the opcode is the branch offset. If the branch is taken, the new address
      // will the the current PC plus the offset. The offset is a signed byte, so it can
      // jump a maximum of 127 bytes forward, or 128 bytes backward.
      //
      // For more info about signed numbers, check here:
      // http://www.emulator101.com/more-about-binary-numbers.html
      Mode::Relative => {
        let relative_offset = self.next_u8() as i8;
        // Due to the nature of binary representaion of numbers, just adding the
        // negative number will result in it being subtract. It will wrap,
        // hence allow the wrapping operation.
        let base_address = self.pc;
        let offset_address = self.pc.wrapping_add(relative_offset as u16);
        self.incur_extra_cycle_on_page_boundary(base_address, offset_address, page_boundary_cycle);
        offset_address
      }
      // Zero-Page is an addressing mode that is only capable of addressing the
      // first 256 bytes of the CPU's memory map. You can think of it as absolute
      // addressing for the first 256 bytes. The instruction LDA $35 will put the
      // value stored in memory location $35 into A. The advantage of zero-page are
      // two - the instruction takes one less byte to specify, and it executes in
      // less CPU cycles. Most programs are written to store the most frequently
      // used variables in the first 256 memory locations so they can take advantage
      // of zero page addressing.
      Mode::ZeroPage => self.next_u8() as u16,
      // This works just like absolute indexed, but the target address is limited to
      // the first 0xFF bytes. The target address will wrap around and will always
      // be in the zero page. If the instruction is LDA $C0,X, and X is $60, then
      // the target address will be $20. $C0+$60 = $120, but the carry is discarded
      // in the calculation of the target address.
      //
      // 6502 bug: Zeropage index will not leave zeropage when page boundary is crossed.
      //           Make sure and do a wrapping add in u8 space.
      Mode::ZeroPageX => (self.next_u8().wrapping_add(self.x)) as u16,
      Mode::ZeroPageY => (self.next_u8().wrapping_add(self.y)) as u16,
      Mode::None => panic!("Mode::None is attempting to be used."),
    }
  }

  fn get_operand(&mut self, mode: Mode, extra_cycle: u8) -> (u16, u8) {
    let address = self.get_operand_address(mode, extra_cycle);
    let value = self.bus.read_u8(address);
    (address, value)
  }

  fn incur_extra_cycle_on_page_boundary(
    &mut self,
    base_address: u16,
    offset_address: u16,
    extra_cycles: u8,
  ) {
    let [_, base_page] = base_address.to_le_bytes();
    let [_, offset_page] = offset_address.to_le_bytes();
    if base_page != offset_page {
      self.cycles += extra_cycles;
    }
  }

  fn tick(&mut self) {
    let opcode = self.next_u8();

    match_opcode!(self, opcode, [
      { 0x00, brk, non, 7, 0 },
      { 0x01, ora, izx, 6, 0 },
      { 0x02, kil, non, 0, 0 },
      { 0x03, slo, izx, 8, 0 },
      { 0x04, nop, zp,  3, 0 },
      { 0x05, ora, zp,  3, 0 },
      { 0x06, asl, zp,  5, 0 },
      { 0x07, slo, zp,  5, 0 },
      { 0x08, php, non, 3, 0 },
      { 0x09, ora, imm, 2, 0 },
      { 0x0a, asl, non, 2, 0 },
      { 0x0b, anc, imm, 2, 0 },
      { 0x0c, nop, abs, 4, 0 },
      { 0x0d, ora, abs, 4, 0 },
      { 0x0e, asl, abs, 6, 0 },
      { 0x0f, slo, abs, 6, 0 },
      { 0x10, bpl, rel, 2, 0 },
      { 0x11, ora, izy, 5, 0 },
      { 0x12, kil, non, 0, 0 },
      { 0x13, slo, izy, 8, 0 },
      { 0x14, nop, zpx, 4, 0 },
      { 0x15, ora, zpx, 4, 0 },
      { 0x16, asl, zpx, 6, 0 },
      { 0x17, slo, zpx, 6, 0 },
      { 0x18, clc, non, 2, 0 },
      { 0x19, ora, aby, 4, 0 },
      { 0x1a, nop, non, 2, 0 },
      { 0x1b, slo, aby, 7, 0 },
      { 0x1c, nop, abx, 4, 0 },
      { 0x1d, ora, abx, 4, 0 },
      { 0x1e, asl, abx, 7, 0 },
      { 0x1f, slo, abx, 7, 0 },
      { 0x20, jsr, abs, 6, 0 },
      { 0x21, and, izx, 6, 0 },
      { 0x22, kil, non, 0, 0 },
      { 0x23, rla, izx, 8, 0 },
      { 0x24, bit, zp,  3, 0 },
      { 0x25, and, zp,  3, 0 },
      { 0x26, rol, zp,  5, 0 },
      { 0x27, rla, zp,  5, 0 },
      { 0x28, plp, non, 4, 0 },
      { 0x29, and, imm, 2, 0 },
      { 0x2a, rol, non, 2, 0 },
      { 0x2b, anc, imm, 2, 0 },
      { 0x2c, bit, abs, 4, 0 },
      { 0x2d, and, abs, 4, 0 },
      { 0x2e, rol, abs, 6, 0 },
      { 0x2f, rla, abs, 6, 0 },
      { 0x30, bmi, rel, 2, 0 },
      { 0x31, and, izy, 5, 0 },
      { 0x32, kil, non, 0, 0 },
      { 0x33, rla, izy, 8, 0 },
      { 0x34, nop, zpx, 4, 0 },
      { 0x35, and, zpx, 4, 0 },
      { 0x36, rol, zpx, 6, 0 },
      { 0x37, rla, zpx, 6, 0 },
      { 0x38, sec, non, 2, 0 },
      { 0x39, and, aby, 4, 0 },
      { 0x3a, nop, non, 2, 0 },
      { 0x3b, rla, aby, 7, 0 },
      { 0x3c, nop, abx, 4, 0 },
      { 0x3d, and, abx, 4, 0 },
      { 0x3e, rol, abx, 7, 0 },
      { 0x3f, rla, abx, 7, 0 },
      { 0x40, rti, non, 6, 0 },
      { 0x41, eor, izx, 6, 0 },
      { 0x42, kil, non, 0, 0 },
      { 0x43, sre, izx, 8, 0 },
      { 0x44, nop, zp,  3, 0 },
      { 0x45, eor, zp,  3, 0 },
      { 0x46, lsr, zp,  5, 0 },
      { 0x47, sre, zp,  5, 0 },
      { 0x48, pha, non, 3, 0 },
      { 0x49, eor, imm, 2, 0 },
      { 0x4a, lsr, non, 2, 0 },
      { 0x4b, alr, imm, 2, 0 },
      { 0x4c, jmp, abs, 3, 0 },
      { 0x4d, eor, abs, 4, 0 },
      { 0x4e, lsr, abs, 6, 0 },
      { 0x4f, sre, abs, 6, 0 },
      { 0x50, bvc, rel, 2, 0 },
      { 0x51, eor, izy, 5, 0 },
      { 0x52, kil, non, 0, 0 },
      { 0x53, sre, izy, 8, 0 },
      { 0x54, nop, zpx, 4, 0 },
      { 0x55, eor, zpx, 4, 0 },
      { 0x56, lsr, zpx, 6, 0 },
      { 0x57, sre, zpx, 6, 0 },
      { 0x58, cli, non, 2, 0 },
      { 0x59, eor, aby, 4, 0 },
      { 0x5a, nop, non, 2, 0 },
      { 0x5b, sre, aby, 7, 0 },
      { 0x5c, nop, abx, 4, 0 },
      { 0x5d, eor, abx, 4, 0 },
      { 0x5e, lsr, abx, 7, 0 },
      { 0x5f, sre, abx, 7, 0 },
      { 0x60, rts, non, 6, 0 },
      { 0x61, adc, izx, 6, 0 },
      { 0x62, kil, non, 0, 0 },
      { 0x63, rra, izx, 8, 0 },
      { 0x64, nop, zp,  3, 0 },
      { 0x65, adc, zp,  3, 0 },
      { 0x66, ror, zp,  5, 0 },
      { 0x67, rra, zp,  5, 0 },
      { 0x68, pla, non, 4, 0 },
      { 0x69, adc, imm, 2, 0 },
      { 0x6a, ror, non, 2, 0 },
      { 0x6b, arr, imm, 2, 0 },
      { 0x6c, jmp, ind, 5, 0 },
      { 0x6d, adc, abs, 4, 0 },
      { 0x6e, ror, abs, 6, 0 },
      { 0x6f, rra, abs, 6, 0 },
      { 0x70, bvs, rel, 2, 0 },
      { 0x71, adc, izy, 5, 0 },
      { 0x72, kil, non, 0, 0 },
      { 0x73, rra, izy, 8, 0 },
      { 0x74, nop, zpx, 4, 0 },
      { 0x75, adc, zpx, 4, 0 },
      { 0x76, ror, zpx, 6, 0 },
      { 0x77, rra, zpx, 6, 0 },
      { 0x78, sei, non, 2, 0 },
      { 0x79, adc, aby, 4, 0 },
      { 0x7a, nop, non, 2, 0 },
      { 0x7b, rra, aby, 7, 0 },
      { 0x7c, nop, abx, 4, 0 },
      { 0x7d, adc, abx, 4, 0 },
      { 0x7e, ror, abx, 7, 0 },
      { 0x7f, rra, abx, 7, 0 },
      { 0x80, nop, imm, 2, 0 },
      { 0x81, sta, izx, 6, 0 },
      { 0x82, nop, imm, 2, 0 },
      { 0x83, sax, izx, 6, 0 },
      { 0x84, sty, zp,  3, 0 },
      { 0x85, sta, zp,  3, 0 },
      { 0x86, stx, zp,  3, 0 },
      { 0x87, sax, zp,  3, 0 },
      { 0x88, dey, non, 2, 0 },
      { 0x89, nop, imm, 2, 0 },
      { 0x8a, txa, non, 2, 0 },
      { 0x8b, xaa, imm, 2, 0 },
      { 0x8c, sty, abs, 4, 0 },
      { 0x8d, sta, abs, 4, 0 },
      { 0x8e, stx, abs, 4, 0 },
      { 0x8f, sax, abs, 4, 0 },
      { 0x90, bcc, rel, 2, 0 },
      { 0x91, sta, izy, 6, 0 },
      { 0x92, kil, non, 0, 0 },
      { 0x93, ahx, izy, 6, 0 },
      { 0x94, sty, zpx, 4, 0 },
      { 0x95, sta, zpx, 4, 0 },
      { 0x96, stx, zpy, 4, 0 },
      { 0x97, sax, zpy, 4, 0 },
      { 0x98, tya, non, 2, 0 },
      { 0x99, sta, aby, 5, 0 },
      { 0x9a, txs, non, 2, 0 },
      { 0x9b, tas, aby, 5, 0 },
      { 0x9c, shy, abx, 5, 0 },
      { 0x9d, sta, abx, 5, 0 },
      { 0x9e, shx, aby, 5, 0 },
      { 0x9f, ahx, aby, 5, 0 },
      { 0xa0, ldy, imm, 2, 0 },
      { 0xa1, lda, izx, 6, 0 },
      { 0xa2, ldx, imm, 2, 0 },
      { 0xa3, lax, izx, 6, 0 },
      { 0xa4, ldy, zp,  3, 0 },
      { 0xa5, lda, zp,  3, 0 },
      { 0xa6, ldx, zp,  3, 0 },
      { 0xa7, lax, zp,  3, 0 },
      { 0xa8, tay, non, 2, 0 },
      { 0xa9, lda, imm, 2, 0 },
      { 0xaa, tax, non, 2, 0 },
      { 0xab, lax, imm, 2, 0 },
      { 0xac, ldy, abs, 4, 0 },
      { 0xad, lda, abs, 4, 0 },
      { 0xae, ldx, abs, 4, 0 },
      { 0xaf, lax, abs, 4, 0 },
      { 0xb0, bcs, rel, 2, 0 },
      { 0xb1, lda, izy, 5, 0 },
      { 0xb2, kil, non, 0, 0 },
      { 0xb3, lax, izy, 5, 0 },
      { 0xb4, ldy, zpx, 4, 0 },
      { 0xb5, lda, zpx, 4, 0 },
      { 0xb6, ldx, zpy, 4, 0 },
      { 0xb7, lax, zpy, 4, 0 },
      { 0xb8, clv, non, 2, 0 },
      { 0xb9, lda, aby, 4, 0 },
      { 0xba, tsx, non, 2, 0 },
      { 0xbb, las, aby, 4, 0 },
      { 0xbc, ldy, abx, 4, 0 },
      { 0xbd, lda, abx, 4, 0 },
      { 0xbe, ldx, aby, 4, 0 },
      { 0xbf, lax, aby, 4, 0 },
      { 0xc0, cpy, imm, 2, 0 },
      { 0xc1, cmp, izx, 6, 0 },
      { 0xc2, nop, imm, 2, 0 },
      { 0xc3, dcp, izx, 8, 0 },
      { 0xc4, cpy, zp,  3, 0 },
      { 0xc5, cmp, zp,  3, 0 },
      { 0xc6, dec, zp,  5, 0 },
      { 0xc7, dcp, zp,  5, 0 },
      { 0xc8, iny, non, 2, 0 },
      { 0xc9, cmp, imm, 2, 0 },
      { 0xca, dex, non, 2, 0 },
      { 0xcb, axs, imm, 2, 0 },
      { 0xcc, cpy, abs, 4, 0 },
      { 0xcd, cmp, abs, 4, 0 },
      { 0xce, dec, abs, 6, 0 },
      { 0xcf, dcp, abs, 6, 0 },
      { 0xd0, bne, rel, 2, 0 },
      { 0xd1, cmp, izy, 5, 0 },
      { 0xd2, kil, non, 0, 0 },
      { 0xd3, dcp, izy, 8, 0 },
      { 0xd4, nop, zpx, 4, 0 },
      { 0xd5, cmp, zpx, 4, 0 },
      { 0xd6, dec, zpx, 6, 0 },
      { 0xd7, dcp, zpx, 6, 0 },
      { 0xd8, cld, non, 2, 0 },
      { 0xd9, cmp, aby, 4, 0 },
      { 0xda, nop, non, 2, 0 },
      { 0xdb, dcp, aby, 7, 0 },
      { 0xdc, nop, abx, 4, 0 },
      { 0xdd, cmp, abx, 4, 0 },
      { 0xde, dec, abx, 7, 0 },
      { 0xdf, dcp, abx, 7, 0 },
      { 0xe0, cpx, imm, 2, 0 },
      { 0xe1, sbc, izx, 6, 0 },
      { 0xe2, nop, imm, 2, 0 },
      { 0xe3, isc, izx, 8, 0 },
      { 0xe4, cpx, zp,  3, 0 },
      { 0xe5, sbc, zp,  3, 0 },
      { 0xe6, inc, zp,  5, 0 },
      { 0xe7, isc, zp,  5, 0 },
      { 0xe8, inx, non, 2, 0 },
      { 0xe9, sbc, imm, 2, 0 },
      { 0xea, nop, non, 2, 0 },
      { 0xeb, sbc, imm, 2, 0 },
      { 0xec, cpx, abs, 4, 0 },
      { 0xed, sbc, abs, 4, 0 },
      { 0xee, inc, abs, 6, 0 },
      { 0xef, isc, abs, 6, 0 },
      { 0xf0, beq, rel, 2, 0 },
      { 0xf1, sbc, izy, 5, 0 },
      { 0xf2, kil, non, 0, 0 },
      { 0xf3, isc, izy, 8, 0 },
      { 0xf4, nop, zpx, 4, 0 },
      { 0xf5, sbc, zpx, 4, 0 },
      { 0xf6, inc, zpx, 6, 0 },
      { 0xf7, isc, zpx, 6, 0 },
      { 0xf8, sed, non, 2, 0 },
      { 0xf9, sbc, aby, 4, 0 },
      { 0xfa, nop, non, 2, 0 },
      { 0xfb, isc, aby, 7, 0 },
      { 0xfc, nop, abx, 4, 0 },
      { 0xfd, sbc, abx, 4, 0 },
      { 0xfe, inc, abx, 7, 0 },
      { 0xff, isc, abx, 7, 0 }
    ]);
  }

  /// These flags are commonly set together.
  fn update_zero_and_negative_flag(&mut self, value: u8) {
    // Numbers can be interpreted as signed or unsigned. The negative flag only
    // cares if the most-significant bit is 1 or 0.
    let negative = 0b1000_0000;
    self.set_status_flag(StatusFlag::Zero, value == 0);
    self.set_status_flag(StatusFlag::Negative, value & negative == negative);
  }

  /// ADC and SBC operate on 9 bits. 8 of them are the register A, while the last bit
  /// is the carry flag. Store this 9th bit onto the status flag.
  fn update_carry_flag(&mut self, result: u16) {
    let carry = 0b1_0000_0000;
    self.set_status_flag(StatusFlag::Carry, result & carry == carry);
  }

  /// Overflow for ADC and SBC indicates if we overflow the most significant bit
  /// of the u8, and change the meaning of a number from being negative or positive.
  fn update_overflow_flag(&mut self, operand: u8, result: u8) {
    let bit_7_mask = 0b1000_0000;

    let does_overflow = (
      // Only look at bit 7, the most significant bit (MSB)
      bit_7_mask &
        // A and operand have the same MSB.
        !(self.a ^ operand) &
        // A and result have a different MSB
        (self.a ^ result)
    ) == bit_7_mask; // Are both conditions correct as commented above?

    self.set_status_flag(StatusFlag::Overflow, does_overflow);
  }

  fn set_status_flag(&mut self, status_flag: StatusFlag, value: bool) {
    if value {
      self.p |= status_flag as u8;
    } else {
      self.p &= !(status_flag as u8);
    }
  }

  fn get_carry(&self) -> u8 {
    self.p & (StatusFlag::Carry as u8)
  }

  fn is_status_flag_set(&self, status_flag: StatusFlag) -> bool {
    let flag = status_flag as u8;
    self.p & (flag as u8) == flag as u8
  }

  /// This function implements pushing to the stack.
  /// See the "S" register for more details.
  fn push_stack_u8(&mut self, value: u8) {
    // The stack page is hard coded.
    let address = u16::from_le_bytes([self.s, memory_range::STACK_PAGE]);
    // The stack points to the next available memory.
    self.bus.set_u8(address, value);
    // Grow down only after setting the memory.
    self.s = self.s.wrapping_sub(1);
  }

  /// This function implements pulling to the stack.
  /// See the "S" register for more details.
  fn pull_stack_u8(&mut self) -> u8 {
    // The current stack pointer points at available memory, decrement it first.
    self.s = self.s.wrapping_add(1);
    // Now read out the memory that is being pulled.
    let address = u16::from_le_bytes([self.s, memory_range::STACK_PAGE]);
    self.bus.read_u8(address)
  }

  /// This function implements pushing to the stack.
  /// See the "S" register for more details.
  fn push_stack_u16(&mut self, value: u16) {
    let address = u16::from_le_bytes([self.s, memory_range::STACK_PAGE]);
    // The stack points to the next available memory.
    self.bus.set_u16(address, value);
    // Grow down only after setting the memory.
    self.s = self.s.wrapping_sub(2);
  }

  /// This function implements pulling to the stack.
  /// See the "S" register for more details.
  fn pull_stack_u16(&mut self) -> u16 {
    // The current stack pointer points at available memory, decrement it first.
    self.s = self.s.wrapping_add(2);
    // Now read out the memory that is being pulled.
    let stack_page = 0x01;
    let address = u16::from_le_bytes([self.s, stack_page]);
    self.bus.read_u16(address)
  }
}

#[cfg(test)]
mod test {
  use super::super::opcodes::*;
  use super::*;

  fn run_program_until<F>(program: Vec<u8>, condition: F) -> Mos6502Cpu
  where
    F: Fn(&Mos6502Cpu) -> bool,
  {
    let mut cpu = Mos6502Cpu::new({
      let mut bus = Bus::new();

      // This will load the value into the accu
      bus.load_program(&program);
      bus
    });

    cpu.run_until(condition);
    cpu
  }

  #[test]
  fn load_value_into_accumulator_using_immediate_addressing() {
    let cpu = run_program_until(
      vec![
        OpCode::LDA_imm as u8, // Load a value into the A register.
        0x66,                  // Here is the value, which is 1 byte.
      ],
      |cpu| cpu.a == 0x66,
    );

    assert_eq!(cpu.a, 0x66, "cpu.a");
    assert_eq!(cpu.cycles, 2, "cpu.cycles");
  }

  #[test]
  fn run_logical_or_on_the_a_register() {
    let a = 0b1010_1010;
    let b = 0b1111_0000;
    let result = 0b1111_1010;

    assert!(a | b == result, "Check out the assumption on the values");

    let cpu = run_program_until(
      vec![
        OpCode::LDA_imm as u8, // Load a value into the A register.
        a,                     // Here is the value, which is 1 byte.
        OpCode::ORA_imm as u8, // The | operator
        b,                     // Now the operand for the operation
      ],
      |cpu| cpu.a == result,
    );

    assert_eq!(cpu.a, result, "cpu.a");
    assert_eq!(cpu.cycles, 4, "cpu.cycles");
  }

  #[test]
  fn run_logical_and_on_the_a_register() {
    let a = 0b1010_1010;
    let b = 0b1111_0000;
    let result = 0b1010_0000;

    assert!(a & b == result, "Check out the assumption on the values");

    let cpu = run_program_until(
      vec![
        OpCode::LDA_imm as u8, // Load a value into the A register.
        a,                     // Here is the value, which is 1 byte.
        OpCode::AND_imm as u8, // The & operator
        b,                     // Now the operand for the operation
      ],
      |cpu| cpu.a == result,
    );

    assert_eq!(cpu.a, result, "cpu.a");
    assert_eq!(cpu.cycles, 4, "cpu.cycles");
  }

  use crate::asm::AsmLexer;

  fn run_program(text: &str) -> Mos6502Cpu {
    let mut lexer = AsmLexer::new(text);

    match lexer.parse() {
      Ok(_) => {
        let mut program = lexer.to_bytes().unwrap();
        program.push(OpCode::KIL as u8);
        let mut cpu = Mos6502Cpu::new({
          let mut bus = Bus::new();

          // This will load the value into the accu
          bus.load_program(&program);
          bus
        });

        cpu.run();
        cpu
      }
      Err(parse_error) => {
        parse_error.panic_nicely();
        panic!("");
      }
    }
  }

  fn assert_register_a(text: &str, value: u8, status: u8) {
    let cpu = run_program(text);
    if cpu.a != value {
      panic!(
        "\n{}\nExpected register A to be {:#x} ({:#b}) but it was {:#x} ({:#b})",
        text, value, value, cpu.a, cpu.a
      );
    }
    assert_status(&cpu, status);
  }

  fn assert_register_x(text: &str, value: u8, status: u8) {
    let cpu = run_program(text);
    if cpu.x != value {
      panic!(
        "\n{}\nExpected register X to be {:#x} ({:#b}) but it was {:#x} ({:#b})",
        text, value, value, cpu.x, cpu.x
      );
    }
    assert_status(&cpu, status);
  }

  fn assert_register_y(text: &str, value: u8, status: u8) {
    let cpu = run_program(text);
    if cpu.y != value {
      panic!(
        "\n{}\nExpected register X to be {:#x} ({:#b}) but it was {:#x} ({:#b})",
        text, value, value, cpu.x, cpu.x
      );
    }
    assert_status(&cpu, status);
  }

  fn assert_status(cpu: &Mos6502Cpu, value: u8) {
    let mut result = String::new();

    let expected_carry = value & StatusFlag::Carry as u8 == StatusFlag::Carry as u8;
    let expected_zero = value & StatusFlag::Zero as u8 == StatusFlag::Zero as u8;
    let expected_interruptdisable =
      value & StatusFlag::InterruptDisable as u8 == StatusFlag::InterruptDisable as u8;
    let expected_decimal = value & StatusFlag::Decimal as u8 == StatusFlag::Decimal as u8;
    let expected_break = value & StatusFlag::Break as u8 == StatusFlag::Break as u8;
    let expected_push = value & StatusFlag::Push as u8 == StatusFlag::Push as u8;
    let expected_overflow = value & StatusFlag::Overflow as u8 == StatusFlag::Overflow as u8;
    let expected_negative = value & StatusFlag::Negative as u8 == StatusFlag::Negative as u8;

    let actual_carry = cpu.is_status_flag_set(StatusFlag::Carry);
    let actual_zero = cpu.is_status_flag_set(StatusFlag::Zero);
    let actual_interruptdisable = cpu.is_status_flag_set(StatusFlag::InterruptDisable);
    let actual_decimal = cpu.is_status_flag_set(StatusFlag::Decimal);
    let actual_break = cpu.is_status_flag_set(StatusFlag::Break);
    let actual_push = cpu.is_status_flag_set(StatusFlag::Push);
    let actual_overflow = cpu.is_status_flag_set(StatusFlag::Overflow);
    let actual_negative = cpu.is_status_flag_set(StatusFlag::Negative);

    if expected_carry != actual_carry {
      result.push_str(&format!(
        "Expected StatusFlag::Carry to be {} but received {}\n",
        expected_carry, actual_carry
      ));
    }
    if expected_zero != actual_zero {
      result.push_str(&format!(
        "Expected StatusFlag::Zero to be {} but received {}\n",
        expected_zero, actual_zero
      ));
    }
    if expected_interruptdisable != actual_interruptdisable {
      result.push_str(&format!(
        "Expected StatusFlag::InterruptDisable to be {} but received {}\n",
        expected_interruptdisable, actual_interruptdisable
      ));
    }
    if expected_decimal != actual_decimal {
      result.push_str(&format!(
        "Expected StatusFlag::Decimal to be {} but received {}\n",
        expected_decimal, actual_decimal
      ));
    }
    if expected_break != actual_break {
      result.push_str(&format!(
        "Expected StatusFlag::Break to be {} but received {}\n",
        expected_break, actual_break
      ));
    }
    if expected_push != actual_push {
      result.push_str(&format!(
        "Expected StatusFlag::Push to be {} but received {}\n",
        expected_push, actual_push
      ));
    }
    if expected_overflow != actual_overflow {
      result.push_str(&format!(
        "Expected StatusFlag::Overflow to be {} but received {}\n",
        expected_overflow, actual_overflow
      ));
    }
    if expected_negative != actual_negative {
      result.push_str(&format!(
        "Expected StatusFlag::Negative to be {} but received {}\n",
        expected_negative, actual_negative
      ));
    }

    if cpu.p != value {
      panic!(
        "\nExpected cpu status 0b{:08b} to match 0b{:08b}\n{}",
        cpu.p, value, result
      );
    }
  }

  macro_rules! register_a {
    ($name:ident, $text:expr, $a:expr, $p:expr) => {
      #[test]
      fn $name() {
        assert_register_a($text, $a, $p);
      }
    };
  }

  macro_rules! register_x {
    ($name:ident, $text:expr, $x:expr, $p:expr) => {
      #[test]
      fn $name() {
        assert_register_x($text, $x, $p);
      }
    };
  }

  macro_rules! register_y {
    ($name:ident, $text:expr, $y:expr, $p:expr) => {
      #[test]
      fn $name() {
        assert_register_y($text, $y, $p);
      }
    };
  }

  #[rustfmt::skip]
  mod imm {
    use super::*;
    use StatusFlag::*;
    const P: u8 = RESET_STATUS_FLAG;

    const C: u8 = Carry as u8;
    const Z: u8 = Zero as u8;
    const I: u8 = InterruptDisable as u8;
    const D: u8 = Decimal as u8;
    const B: u8 = Break as u8;
    const T: u8 = Push as u8;
    const V: u8 = Overflow as u8;
    const N: u8 = Negative as u8;

    // These test the CPU using a macro, in order to tersely test the system.
    // For instance this command will run the test:
    //
    // `cargo test mos_6502_emulator::test::imm::test_adc1`
    //
    //             TestName   Program               Register Result
    //             |          |                     |     Status Register
    //             |          |                     |     |
    // register_a!(test_adc1, "lda #$22\nadc #$11", 0x33, P);

    // This first test shows: 0x22 + 0x11 == 0x33.
    // P is the default "P" or status register values.
    register_a!(test_adc1, "lda #$22\nadc #$11", 0x33, P);
    // This add doesn't do anything, but the N, or negative flag is set since the most
    // significant bit is 1.
    register_a!(test_adc2, "lda #$FF\nadc #$00", 0xff, P | N);
    // Here we overflow the u8.
    register_a!(
      test_adc3,
      "
        lda #$FF  ; 255 signed, or -1 unsigned
        adc #$01  ;   1 signed, or 1 unsigned
      ",
      0x00,
      P
      | C // For unsigned numbers, te carry bit is flipped, since the result carries over.
      | Z // The result is 0x00 (with the carry only in the status register)
    );
    // This is a similar result as above, but the final resut is not 0.
    register_a!(test_adc4, "lda #$FF\nadc #$02", 0x01, P | C);
    // Check that this uses the carry flag.
    register_a!(
      test_adc_carry,
      "
        sec      ; Set the carry flag
        lda #$11 ; Load A with a value
        adc #$22 ; This should add all three values
                 ; = 0x01 + 0x11 + 0x22
      ",
      0x34,
      P
    );
    mod adc_overflow_carry {
      // This section tests the adc cases from:
      // http://www.6502.org/tutorials/vflag.html
      use super::*;
      register_a!(
        test_1_1,
        "
          CLC      ; 1 + 1 = 2, returns C = 0
          LDA #$01 ;            returns V = 0
          ADC #$01
        ",
        0x02,
        P
      );
      // 0b0000_0001
      // 0x1111_1111
      // 1_0000_0000
      register_a!(
        test_1_neg1,
        "
          CLC      ; 1 + -1 = 0, returns C = 1
          LDA #$01 ;                     V = 0
          ADC #$FF
        ",
        0x00,
        P | C | Z
      );

      // 0b0111_1111
      // 0b0000_0001
      // 0b1000_0000
      register_a!(
        test_127_1,
        "
          CLC      ; 127 + 1 = 128, returns C = 0
          LDA #$7F ;                        V = 1
          ADC #$01
        ",
        0b1000_0000,
        P | V | N
      );

      // 0x80 + 0xff
      // 0b1000_0000
      // 0b1111_1111
      // 1_0111_1111
      register_a!(
        test_neg128_negative_1,
        "
          CLC      ; -128 + -1 = -129, returns C = 1
          LDA #$80 ;                           V = 1
          ADC #$FF
        ",
        0b0111_1111,
        P | C | V
      );

      // 0b0011_1111  a
      // 0b0100_0000  operand
      // 0b0000_0001  carry
      // 0b0000_0000  result
      register_a!(
        test_carry,
        "
          SEC      ; Note: SEC, not CLC
          LDA #$3F ; 63 + 64 + 1 = 128, returns V = 1
          ADC #$40
        ",
        0b1000_0000,
        P | V | N
      );
    }

    mod sbc_overflow_carry {
      // This section tests the sbc cases from:
      // http://www.6502.org/tutorials/vflag.html
      use super::*;
      // 0b0000_0000   two's comp   0b0000_0000
      // 0b0000_0001       ->       0b1111_1111
      //                            0b1111_1111
      register_a!(
        test_0_minus_1,
        "
          SEC      ; 0 - 1 = -1, returns V = 0
          LDA #$00
          SBC #$01
        ",
        negative(1),
        P | N
      );

      // 0b1000_0000    0b1000_0000
      // 0b0000_0001 -> 0b1111_1111
      //              0b1_0111_1111
      register_a!(
        test_neg128_minus_1,
        "
          SEC      ; -128 - 1 = -129, returns V = 1
          LDA #$80
          SBC #$01
        ",
        negative(129),
        P | C | V
      );

      // 0b0111_1111    0b0111_1111
      // 0b1111_1111 -> 0b0000_0001
      //                0b1000_0000
      register_a!(
        test_127_minus_neg1,
        "
          SEC      ; 127 - -1 = 128, returns V = 1
          LDA #$7F
          SBC #$FF
        ",
        128,
        P | V | N
      );

      //   0b1100_0000    0b1100_0000
      // - 0b0100_0000 => 0b1011_1111
      //                  1_0111_1111
      register_a!(
        test_clc,
        "
          CLC      ; Note: CLC, not SEC
          LDA #$C0 ; -64 - 64 - 1 = -129, returns V = 1
          SBC #$40
        ",
        negative(129),
        P | C | V
      );
    }

    // register_a!(test_, "alr #$22", 0x22, P); // illegal
    // register_a!(test_, "anc #$22", 0x22, P); // illegal
    // register_a!(test_, "axs #$22", 0x22, P); // illegal
    // register_a!(test_, "arr #$22", 0x22, P); // illegal
    // register_a!(test_, "cpx #$22", 0x22, P);
    // register_a!(test_, "cpy #$22", 0x22, P);
    // register_a!(test_, "cmp #$22", 0x22, P);
    // register_a!(test_, "eor #$22", 0x22, P);
    // register_a!(test_, "lax #$22", 0x22, P);
    register_a!(test_lda, "lda #$22", 0x22, P);
    register_x!(test_ldx, "ldx #$22", 0x22, P);
    register_y!(test_ldy, "ldy #$22", 0x22, P);
    register_a!(test_nop, "nop #$22", 0x00, P);
    register_a!(test_sbc1, "sec\nlda #$33\n sbc #$11", 0x22, P | C);
    register_a!(test_sbc2, "sec\nlda #$33\n sbc #$33", 0x00, P | Z | C);
    register_a!(test_sbc3, "sec\nlda #$33\n sbc #$34", negative(1), P | N);
    // register_a!(test_, "ora #$22", 0x22, P);
    // register_a!(test_, "xaa #$22", 0x22, P);
  }

  // Run two's complement on a u8.
  fn negative(n: u8) -> u8 {
    !n + 1
  }
}
