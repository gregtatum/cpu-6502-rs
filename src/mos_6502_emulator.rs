use super::bus::Bus;
use super::constants::InterruptVectors;

const CLOCK_SPEED: f64 = 1.789773; // Mhz
const CLOCK_DIVISOR: u32 = 12; //Mhz
                               // Emulator authors may wish to emulate the NTSC NES/Famicom CPU at 21441960 Hz
                               // ((341×262-0.5)×4×60) to ensure a synchronised/stable 60 frames per second.
const MASTER_CLOCK_FREQUENCY: f64 = 21.441960; //Mhz
                                               // This is the true frequency:
                                               // const MASTER_CLOCK_FREQUENCY: f64 = 21.477272; //Mhz
const COLOR_SUBCARRIER_FREQUENCY: f64 = 3.57954545; // Mhz

pub enum StatusFlag {
  Carry = 0b00000001,
  Zero = 0b00000010,
  InterruptDisable = 0b00000100,
  Decimal = 0b00001000,
  NoEffect1 = 0b00010000,
  NoEffect2 = 0b00100000,
  Overflow = 0b01000000,
  Negative = 0b10000000,
}

pub enum Mode {
  Absolute,         // ABS
  AbsoluteIndexedX, // ABX
  AbsoluteIndexedY, // ABY
  Immediate,        // IMM
  Implied,          // IMP
  Indirect,         // IND
  IndirectX,        // IZX
  IndirectY,        // IZY
  Relative,         // REL
  ZeroPage,         // ZP
  ZeroPageX,        // ZPX
  ZeroPageY,        // ZPY
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
    // Go ahead and read the first instruction from the reset vector. If the reset
    // vector is set again, the program will end.
    let program_counter = bus.read_u16(InterruptVectors::ResetVector as u16);

    Mos6502Cpu {
      bus,
      accumulator: 0, // A
      x_index: 0,     // X
      y_index: 0,     // Y
      //
      program_counter,
      stack_pointer: 0xFD,   // S
      status_register: 0x34, // P
      cycles: 0,
    }
  }

  fn next_u8(&mut self) -> u8 {
    let value = self.bus.read_u8(self.program_counter);
    self.program_counter += 1;
    value
  }

  fn next_u16(&mut self) -> u16 {
    let value = self.bus.read_u16(self.program_counter);
    self.program_counter += 2;
    value
  }

  /// Handle individual instructions
  /// https://github.com/munshkr/nesasm/blob/master/docs/cpu_inst.txt
  fn tick(&mut self) {
    let opcode = self.next_u8();
    match opcode {
      0x01 => self.ora(Mode::IndirectX, 6, false),
      0x05 => self.ora(Mode::ZeroPage, 3, false),
      0x09 => self.ora(Mode::Immediate, 2, false),
      0x0d => self.ora(Mode::Absolute, 4, false),
      0x11 => self.ora(Mode::IndirectY, 5, true),
      0x15 => self.ora(Mode::ZeroPageX, 4, false),
      0x19 => self.ora(Mode::AbsoluteIndexedY, 4, true),
      0x1d => self.ora(Mode::AbsoluteIndexedX, 4, true),

      0xa1 => self.lda(Mode::IndirectX, 6, false),
      0xa5 => self.lda(Mode::ZeroPage, 3, false),
      0xa9 => self.lda(Mode::Immediate, 2, false),
      0xad => self.lda(Mode::Absolute, 4, false),
      0xb1 => self.lda(Mode::IndirectY, 5, true),
      0xb5 => self.lda(Mode::ZeroPageX, 4, false),
      0xb9 => self.lda(Mode::AbsoluteIndexedY, 4, true),
      0xbd => self.lda(Mode::AbsoluteIndexedX, 4, true),

      _ => panic!("Unhandled opcode {}", opcode),
    }
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

  /// The source for the comments on the modes is coming from:
  /// http://www.emulator101.com/6502-addressing-modes.html
  fn get_operand_address(&mut self, mode: Mode, _incur_extra_cycle_on_page_boundary: bool) -> u16 {
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
        return self.next_u16() + self.x_index as u16;
      }
      Mode::AbsoluteIndexedY => {
        return self.next_u16() + self.y_index as u16;
      }
      // These instructions have their data defined as the next byte after the
      // opcode. ORA #$B2 will perform a logical (also called bitwise) of the
      // value B2 with the accumulator. Remember that in assembly when you see
      // a # sign, it indicates an immediate value. If $B2 was written without
      // a #, it would indicate an address or offset.
      Mode::Immediate => {
        // Return the current program counter as the address, but also increment
        // the program counter.
        let address = self.program_counter;
        self.program_counter += 1;
        return address;
      }
      // In an implied instruction, the data and/or destination is mandatory for
      // the instruction. For example, the CLC instruction is implied, it is going
      // to clear the processor's Carry flag.
      Mode::Implied => panic!("An implied mode should never be directly activated."),
      Mode::Indirect => panic!("Unhandled mode."),
      Mode::IndirectX => panic!("Unhandled mode."),
      Mode::IndirectY => panic!("Unhandled mode."),
      // Relative addressing on the 6502 is only used for branch operations. The byte
      // after the opcode is the branch offset. If the branch is taken, the new address
      // will the the current PC plus the offset. The offset is a signed byte, so it can
      // jump a maximum of 127 bytes forward, or 128 bytes backward.
      //
      // For more info about signed numbers, check here:
      // http://www.emulator101.com/more-about-binary-numbers.html
      Mode::Relative => panic!("Unhandled mode."),
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
      Mode::ZeroPageX => (self.next_u8() + self.x_index) as u16,
      Mode::ZeroPageY => (self.next_u8() + self.y_index) as u16,
    }
  }

  /// Apply the logical "or" operator on the accumulator.
  /// A:=A or {adr}
  /// Flags: NZ
  fn ora(&mut self, mode: Mode, cycles: u8, incur_extra_cycle_on_page_boundary: bool) {
    self.cycles += cycles;
    let address = self.get_operand_address(mode, incur_extra_cycle_on_page_boundary);
    self.accumulator |= self.bus.read_u8(address);
    self.update_zero_flag(self.accumulator);
    self.update_negative_flag(self.accumulator);
  }

  /// Load the value into register A
  /// A:={adr}
  /// Flags: NZ
  fn lda(&mut self, mode: Mode, cycles: u8, incur_extra_cycle_on_page_boundary: bool) {
    self.cycles += cycles;
    let address = self.get_operand_address(mode, incur_extra_cycle_on_page_boundary);
    self.accumulator = self.bus.read_u8(address);
    self.update_zero_flag(self.accumulator);
    self.update_negative_flag(self.accumulator);
  }

  fn update_zero_flag(&mut self, value: u8) {
    self.set_status_flag(StatusFlag::Zero, value == 0);
  }

  fn update_negative_flag(&mut self, value: u8) {
    self.set_status_flag(StatusFlag::Zero, value & 0b1000_0000 == 0b1000_0000);
  }

  fn set_status_flag(&mut self, status_flag: StatusFlag, value: bool) {
    if value {
      self.status_register |= status_flag as u8;
    } else {
      self.status_register &= !(status_flag as u8);
    }
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
      |cpu| cpu.accumulator == 0x66,
    );

    assert_eq!(cpu.accumulator, 0x66, "cpu.accumulator");
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
      |cpu| cpu.accumulator == result,
    );

    assert_eq!(cpu.accumulator, result, "cpu.accumulator");
    assert_eq!(cpu.cycles, 4, "cpu.cycles");
  }
}
