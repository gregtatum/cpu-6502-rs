use crate::cpu_6502::*;

/// Apply the logical "or" operator on the accumulator.
/// Function: A:=A or {adr}
/// Flags: N Z
pub fn ora(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.a |= operand;
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Apply the logical "and" operator on the accumulator.
/// Function: A:=A&{adr}
/// Flags: N Z
pub fn and(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.a &= operand;
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Logical Exclusive OR
/// Function: A:=A exor {adr}
/// Flags: N Z
pub fn eor(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.a ^= operand;
  cpu.update_zero_and_negative_flag(cpu.a);
}

fn add_impl(cpu: &mut Cpu6502, operand: u8) {
  // Translating to u16 means that the values won't wrap, so wrapping
  // add is not needed.
  let result_u16 =
    // Get the carry from the previous operation, and carry it over
    // into this one, but operate in the u16 space as to not overflow.
    cpu.get_carry() as u16 + // Either 0x00 or 0x01
    cpu.a as u16 +
    operand as u16;

  let result_u8 = result_u16 as u8;

  cpu.update_zero_and_negative_flag(result_u8);
  // Take the 0x100 value here, and set it to the register. This can then carry
  // over into the next byte of a number.
  cpu.update_carry_flag(result_u16);
  cpu.update_overflow_flag(operand, result_u8);
  cpu.a = result_u8;
}

/// Add with Carry
/// Function: A:=A+{adr}+C
/// Flags: N V Z C
pub fn adc(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  add_impl(cpu, operand);
}

/// Subtract with Carry
/// Function: A:=A-{adr}+C
/// Flags: N V Z C
pub fn sbc(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  // Signed numbers range: -128 to 127
  // 0b0000_0000, 0
  // 0b0000_0001, 1
  // 0b0000_0010, 1
  // ...
  // 0b0111_1111, 127
  // 0b1000_0000, -128
  // 0b1000_0001, -127
  // ...
  // 0b1111_1111, -1

  let (_, operand) = cpu.get_operand(mode, extra_cycle);

  // In order to properly subtract we need the two's complement of the operand.
  // Normally this would be accomplished by;
  // `let twos_complement = !operand + 0x1;`
  //
  // However, in this CPU, this is done by inverting the operand here, and letting
  // the carry flag be the + 1.
  //
  // Because of this, it's assumed the assembly will run SEC before running sbc.
  add_impl(cpu, !operand);
}

/// Compare A with source
/// http://6502.org/tutorials/compare_instructions.html
/// Function: A-{adr}
/// Flags: N Z C
pub fn cmp(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a.wrapping_sub(operand));
  cpu.set_status_flag(StatusFlag::Carry, cpu.a >= operand);
}

/// Compare X with source
/// http://6502.org/tutorials/compare_instructions.html
/// Function: X-{adr}
/// Flags: N Z C
pub fn cpx(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.x.wrapping_sub(operand));
  cpu.set_status_flag(StatusFlag::Carry, cpu.x >= operand);
}

/// Compare Y with source
/// http://6502.org/tutorials/compare_instructions.html
/// Function: Y-{adr}
/// Flags: N Z C
pub fn cpy(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.y.wrapping_sub(operand));
  cpu.set_status_flag(StatusFlag::Carry, cpu.y >= operand);
}

/// Decrement at an address
/// Function: {adr}:={adr}-1
/// Flags: N Z
pub fn dec(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  let result = operand.wrapping_sub(1);
  cpu.update_zero_and_negative_flag(result);
  cpu.bus.set_u8(address, result);
}

/// Decrement X
/// Function: X:=X-1
/// Flags: N Z
pub fn dex(cpu: &mut Cpu6502, _mode: Mode, _extra_cycle: u8) {
  cpu.x = cpu.x.wrapping_sub(1);
  cpu.update_zero_and_negative_flag(cpu.x);
}

/// Decrement Y
/// Function: Y:=Y-1
/// Flags: N Z
pub fn dey(cpu: &mut Cpu6502, _mode: Mode, _extra_cycle: u8) {
  cpu.y = cpu.y.wrapping_sub(1);
  cpu.update_zero_and_negative_flag(cpu.x);
}

/// Increment the address
/// Function: {adr}:={adr}+1
/// Flags: N Z
pub fn inc(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  let result = operand.wrapping_add(1);
  cpu.update_zero_and_negative_flag(result);
  cpu.bus.set_u8(address, result);
}

/// Increment X
/// Function: X:=X+1
/// Flags: N Z
pub fn inx(cpu: &mut Cpu6502, _mode: Mode, _extra_cycle: u8) {
  cpu.x = cpu.x.wrapping_add(1);
  cpu.update_zero_and_negative_flag(cpu.x);
}

/// Increment Y
/// Function: Y:=Y+1
/// Flags: N Z
pub fn iny(cpu: &mut Cpu6502, _mode: Mode, _extra_cycle: u8) {
  cpu.y = cpu.y.wrapping_add(1);
  cpu.update_zero_and_negative_flag(cpu.y);
}

/// Arithmetic shift left
/// Function: {adr}:={adr}*2
/// Flags: N Z C
pub fn asl(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  let result = operand << 1;
  cpu.update_zero_and_negative_flag(result);
  // The Carry flag contains the bit that was shifted out:
  cpu.set_status_flag(StatusFlag::Carry, operand & 0b1000_0000 != 0);
  cpu.bus.set_u8(address, result);
}

/// Rotate left
/// Function: {adr}:={adr}*2+C
/// Flags: N Z C
pub fn rol(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  let result = (operand << 1) | cpu.get_carry();
  cpu.update_zero_and_negative_flag(result);
  // The Carry flag contains the bit that was shifted out:
  cpu.set_status_flag(StatusFlag::Carry, operand & 0b1000_0000 != 0);
  cpu.bus.set_u8(address, result);
}

/// Logical shift right
/// Function: {adr}:={adr}/2
/// Flags: N Z C
pub fn lsr(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  let result = operand >> 1;
  cpu.update_zero_and_negative_flag(result);
  // The Carry flag contains the bit that was shifted out:
  cpu.set_status_flag(StatusFlag::Carry, operand & 0b0000_0001 != 0);
  cpu.bus.set_u8(address, result);
}

/// Rotate right
/// Function: {adr}:={adr}/2+C*128
/// Flags: N Z C
pub fn ror(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);

  let result =
    // Shift the operand, {adr}/2
    (operand >> 1) |
    // Move the carry bit to the beginning 0b0000_0001 -> 0b10000_000
    // C*128
    (cpu.get_carry() << 7);

  cpu.update_zero_and_negative_flag(result);
  // The Carry flag contains the bit that was shifted out:
  cpu.set_status_flag(StatusFlag::Carry, operand & 0b0000_0001 != 0);
  cpu.bus.set_u8(address, result);
}
