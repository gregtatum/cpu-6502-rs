use crate::constants::{memory_range, InterruptVectors};
use crate::opcodes::{Mode, OpCode, OPCODE_STRING_TABLE};
use crate::{bus::SharedBus, opcodes};
pub mod opcodes_illegal;
pub mod opcodes_jump;
pub mod opcodes_logical;
pub mod opcodes_move;

#[cfg(test)]
mod test_helpers;

// Test must be after test_helpers, rust format tries to move things around.
#[cfg(test)]
mod test;

pub const RESET_STATUS_FLAG: u8 = 0b00110100;

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

/// This struct implements the MOS Technology 6502 central processing unit.
///
/// http://www.6502.org/
/// https://en.wikipedia.org/wiki/MOS_Technology_6502
/// http://wiki.nesdev.com/w/index.php/CPU
pub struct Cpu6502 {
    // The bus is what holds all the memory access for the program.
    pub bus: SharedBus,
    // "A" register - The accumulator. Typical results of operations are stored here.
    // In combination with the status register, supports using the status register for
    // carrying, overflow detection, and so on.
    pub a: u8,
    /// "X" register.
    /// Used for several addressing modes  They can be used as loop counters easily, using
    /// INC/DEC and branch instructions. Not being the accumulator, they have limited
    /// addressing modes themselves when loading and saving.
    pub x: u8,
    /// "Y" register.
    pub y: u8,

    /// "PC" - Program counter.
    /// The 2-byte program counter PC supports 65536 direct (unbanked) memory locations,
    /// however not all values are sent to the cartridge. It can be accessed either by
    /// allowing CPU's internal fetch logic increment the address bus, an interrupt
    /// (NMI, Reset, IRQ/BRQ), and using the RTS/JMP/JSR/Branch instructions.
    /// "PC"
    pub pc: u16,

    /// "S" - Stack pointer
    ///
    /// The 6502 has hardware support for a stack implemented using a 256-byte array
    /// whose location is hardcoded at page 0x01 (0x0100-0x01FF), using the S register
    /// for a stack pointer.
    ///
    /// The 6502 uses a descending stack (it grows downwards)
    /// https://wiki.nesdev.com/w/index.php/Stack
    pub s: u8,

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
    pub p: u8,

    /// The number of cycles that were done while operating on an instruction. The
    /// emulator will then need to wait the proper amount of time after executing
    /// the commands.
    pub cycles: u8,

    pub tick_count: u64,

    // Stop running the CPU after so many ticks. Useful for testing.
    pub max_ticks: Option<u64>,
}

impl Cpu6502 {
    pub fn new(bus: SharedBus) -> Cpu6502 {
        // Go ahead and read the first instruction from the reset vector. If the reset
        // vector is set again, the program will end.
        let pc = bus.borrow().read_u16(InterruptVectors::ResetVector as u16);

        Cpu6502 {
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
            tick_count: 0,
            max_ticks: None,
        }
    }

    /// Read the PC without incrementing.
    fn peek_u8(&mut self) -> u8 {
        self.bus.borrow().read_u8(self.pc)
    }

    /// Increment the program counter and read the next u8 value following
    /// the current pc.
    fn next_u8(&mut self) -> u8 {
        let value = self.bus.borrow().read_u8(self.pc);
        self.pc += 1;
        value
    }

    /// Increment the program counter and read the next u16 value following
    /// the current pc.
    fn next_u16(&mut self) -> u16 {
        let value = self.bus.borrow().read_u16(self.pc);
        self.pc += 2;
        value
    }

    /// This function is useful for testing the emulator. It will only run while the
    /// predicate is true.
    pub fn run_until<F>(&mut self, predicate: F)
    where
        F: Fn(&Cpu6502) -> bool,
    {
        while !predicate(self) {
            self.tick();
        }
    }

    /// Run the emulator until the "KIL" or "BRK" command is issued.
    pub fn run(&mut self) {
        while self.peek_u8() != OpCode::KIL as u8 && self.peek_u8() != OpCode::BRK as u8 {
            self.tick();

            // If there is a max ticks counter, respect it.
            if let Some(max_ticks) = self.max_ticks {
                if self.tick_count > max_ticks {
                    break;
                }
            }
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
            Mode::Absolute => self.next_u16(),
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
                self.incur_extra_cycle_on_page_boundary(
                    base_address,
                    offset_address,
                    page_boundary_cycle,
                );
                offset_address
            }
            Mode::AbsoluteIndexedY => {
                let base_address = self.next_u16();
                let offset_address = base_address + self.y as u16;
                self.incur_extra_cycle_on_page_boundary(
                    base_address,
                    offset_address,
                    page_boundary_cycle,
                );
                offset_address
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
                address
            }
            // In an implied instruction, the data and/or destination is mandatory for
            // the instruction. For example, the CLC instruction is implied, it is going
            // to clear the processor's Carry flag.
            Mode::Implied => {
                panic!("Attempting to get the operand address for an implied mode.")
            }
            Mode::RegisterA => {
                panic!("Register A has no address.")
            }
            // The indirect addressing mode is similar to the absolute mode, but the
            // next u16 is actually a pointer to another address. Use this next address
            // for the operation.
            Mode::Indirect => {
                let address = self.next_u16();
                return self.bus.borrow().read_u16(address);
            }
            Mode::IndirectX => {
                let zero_page_address = self.next_u8().wrapping_add(self.x) as u16;
                self.bus.borrow().read_u16(zero_page_address)
            }
            Mode::IndirectY => {
                let zero_page_address = self.next_u8() as u16;
                self.bus.borrow().read_u16(zero_page_address) + self.y as u16
            }
            // Relative addressing on the 6502 is only used for branch operations. The byte
            // after the opcode is the branch offset. If the branch is taken, the new address
            // will the the current PC plus the offset. The offset is a signed byte, so it can
            // jump a maximum of 127 bytes forward, or 128 bytes backward.
            //
            // For more info about signed numbers, check here:
            // http://www.emulator101.com/more-about-binary-numbers.html
            Mode::Relative => {
                let relative_offset = self.next_u8() as i8;
                // We already read the instruction and operand, which incremented the
                // pc by 2 bytes. Get the base address by moving backwards 2.
                let base_address = self.pc - 2;

                // Due to the nature of binary representaion of numbers, just adding the
                // negative number will result in it being subtract. It will wrap,
                // hence allow the wrapping operation.
                let offset_address = base_address.wrapping_add(relative_offset as u16);

                self.incur_extra_cycle_on_page_boundary(
                    base_address,
                    offset_address,
                    page_boundary_cycle,
                );
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
            // For some reason NOP is being called with this. Originally it was
            // a panic.
            Mode::None => 0,
        }
    }

    fn get_address_and_maybe_operand(
        &mut self,
        mode: Mode,
        extra_cycle: u8,
    ) -> (Option<u16>, u8) {
        if mode == Mode::RegisterA {
            return (None, self.a);
        }
        let address = self.get_operand_address(mode, extra_cycle);
        let value = self.bus.borrow().read_u8(address);
        (Some(address), value)
    }

    fn get_address_and_operand(&mut self, mode: Mode, extra_cycle: u8) -> (u16, u8) {
        let address = self.get_operand_address(mode, extra_cycle);
        let value = self.bus.borrow().read_u8(address);
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

    /// Does one operational tick of the CPU. Returns true if there are more
    /// instructions, and false if a KIL operation was encountered.
    pub fn tick(&mut self) -> bool {
        self.tick_count += 1;
        self.cycles = 0;
        let opcode = self.next_u8();

        if opcode == OpCode::KIL as u8 {
            return false;
        }
        let opcode_index = opcode as usize;

        // The operations are all contained in tables that match up the opcode to its
        // particular implementation details.
        self.cycles += opcodes::CYCLES_TABLE[opcode_index];
        let operation_fn = opcodes::OPERATION_FN_TABLE[opcode_index];
        let mode = opcodes::ADDRESSING_MODE_TABLE[opcode_index];
        let extra_cycles = opcodes::EXTRA_CYCLES_TABLE[opcode_index];

        operation_fn(self, mode, extra_cycles);

        true
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

    /// Overflow for ADC and SBC indicates if we overflow from bit 6 to bit 7 of the u8,
    /// and change the meaning of a number from being negative or positive.
    /// e.g. 0b0111_1111 + 0b0000_0001 = 0b1000_0000
    ///        |             |             |
    ///        positive      positive      negative result
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
        self.bus.borrow_mut().set_u8(address, value);
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
        self.bus.borrow().read_u8(address)
    }

    /// This function implements pushing to the stack.
    /// See the "S" register for more details.
    fn push_stack_u16(&mut self, value: u16) {
        let address = u16::from_le_bytes([self.s, memory_range::STACK_PAGE]);
        // The stack points to the next available memory.
        self.bus.borrow_mut().set_u16(
            // An additional byte is needed to store a u16. Subtract since the stack
            // grows down.
            address.wrapping_sub(1),
            value,
        );
        // Grow down only after setting the memory.
        self.s = self.s.wrapping_sub(2);
    }

    /// This function implements pulling to the stack.
    /// See the "S" register for more details.
    fn pull_stack_u16(&mut self) -> u16 {
        // The current stack pointer points at available memory, decrement it first.
        self.s = self.s.wrapping_add(1);
        // Now read out the memory that is being pulled.
        let address = u16::from_le_bytes([self.s, memory_range::STACK_PAGE]);
        self.s = self.s.wrapping_add(1);
        self.bus.borrow().read_u16(address)
    }

    /// This feature was never hooked up to any code, but it's valid (but untested)
    /// behavior.
    #[allow(dead_code)]
    fn handle_irq(&mut self) {
        self.push_stack_u16(self.pc);
        self.push_stack_u8(self.p);
        self.pc = InterruptVectors::ResetVector as u16;
        self.set_status_flag(StatusFlag::InterruptDisable, true);
        self.cycles += 7;
    }
}
