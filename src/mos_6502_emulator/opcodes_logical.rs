use crate::mos_6502_emulator::*;

/// Apply the logical "or" operator on the accumulator.
/// Function: A:=A or {adr}
/// Flags: N Z
pub fn ora(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  cpu.accumulator |= cpu.get_operand(mode, extra_cycle).1;
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Apply the logical "and" operator on the accumulator.
/// Function: A:=A&{adr}
/// Flags: N Z
pub fn and(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  cpu.accumulator &= cpu.get_operand(mode, extra_cycle).1;
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Logical Exclusive OR
/// Function: A:=A exor {adr}
/// Flags: N Z
pub fn eor(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  cpu.accumulator ^= cpu.get_operand(mode, extra_cycle).1;
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Add with Carry
/// Function: A:=A+{adr}
/// Flags: N V Z C
pub fn adc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  let result = cpu.get_carry() as u16 + cpu.accumulator as u16 + operand as u16;
  cpu.update_zero_and_negative_flag(cpu.accumulator);
  cpu.update_carry_and_overflow_flag(operand, result);
  cpu.accumulator = result as u8;
}

/// Substract with Carry
/// Function: A:=A-{adr}
/// Flags: N V Z C
pub fn sbc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let operand = !cpu.get_operand(mode, extra_cycle).1;
  let result = cpu.get_carry() as u16 + cpu.accumulator as u16 + operand as u16;
  cpu.update_zero_and_negative_flag(cpu.accumulator);
  cpu.update_carry_and_overflow_flag(operand, result);
  cpu.accumulator = result as u8;
}

/// Compare A with source
/// Function: A-{adr}
/// Flags: N Z C
pub fn cmp(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator.wrapping_sub(operand));
  cpu.set_status_flag(StatusFlag::Carry, cpu.accumulator >= operand);
}

/// Function: X-{adr}
/// Flags: N Z C
pub fn cpx(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.x_index.wrapping_sub(operand));
  cpu.set_status_flag(StatusFlag::Carry, cpu.x_index >= operand);
}

/// Function: Y-{adr}
/// Flags: N Z C
pub fn cpy(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (_, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.y_index.wrapping_sub(operand));
  cpu.set_status_flag(StatusFlag::Carry, cpu.y_index >= operand);
}

/// Function: {adr}:={adr}-1
/// Flags: N Z
pub fn dec(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  let result = operand.wrapping_sub(1);
  cpu.update_zero_and_negative_flag(result);
  cpu.bus.set_u8(address, result);
}

/// Function: X:=X-1
/// Flags: N Z
pub fn dex(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.x_index = cpu.x_index.wrapping_sub(1);
  cpu.update_zero_and_negative_flag(cpu.x_index);
}

/// Function: Y:=Y-1
/// Flags: N Z
pub fn dey(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.y_index = cpu.y_index.wrapping_sub(1);
  cpu.update_zero_and_negative_flag(cpu.x_index);
}

/// Increment the address
/// Function: {adr}:={adr}+1
/// Flags: N Z
pub fn inc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  let result = operand.wrapping_add(1);
  cpu.update_zero_and_negative_flag(result);
  cpu.bus.set_u8(address, result);
}

/// Increment X
/// Function: X:=X+1
/// Flags: N Z
pub fn inx(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.x_index = cpu.x_index.wrapping_add(1);
  cpu.update_zero_and_negative_flag(cpu.x_index);
}

/// Increment Y
/// Function: Y:=Y+1
/// Flags: N Z
pub fn iny(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.y_index = cpu.y_index.wrapping_add(1);
  cpu.update_zero_and_negative_flag(cpu.y_index);
}

/// Arithmetic shift left
/// Function: {adr}:={adr}*2
/// Flags: N Z C
pub fn asl(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
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
pub fn rol(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
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
pub fn lsr(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
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
pub fn ror(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
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
