use crate::mos_6502_emulator::*;

/// Load the value into register A
/// Function: A:={adr}
/// Flags: N Z
pub fn lda(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.a = cpu.bus.read_u8(address);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Store register A at address
/// Function: {adr}:=A
/// Flags:
pub fn sta(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, _) = cpu.get_operand(mode, extra_cycle);
  cpu.bus.set_u8(address, cpu.a);
}

/// Load register X with the value
/// Function: X:={adr}
/// Flags: N Z
pub fn ldx(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.x = operand;
  cpu.update_zero_and_negative_flag(cpu.x);
}

/// Store register X at address
/// Function: {adr}:=X
/// Flags:
pub fn stx(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, _) = cpu.get_operand(mode, extra_cycle);
  cpu.bus.set_u8(address, cpu.x);
}

/// Load register Y with the value
/// Function: Y:={adr}
/// Flags: N Z
pub fn ldy(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.y = operand;
  cpu.update_zero_and_negative_flag(cpu.y);
}

/// Function: {adr}:=Y
/// Flags:
pub fn sty(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: X:=A
/// Flags: N Z
pub fn tax(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=X
/// Flags: N Z
pub fn txa(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: Y:=A
/// Flags: N Z
pub fn tay(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=Y
/// Flags: N Z
pub fn tya(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: X:=S
/// Flags: N Z
pub fn tsx(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: S:=X
/// Flags:
pub fn txs(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=+(S)
/// Flags: N Z
pub fn pla(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: (S)-:=A
/// Flags:
pub fn pha(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: P:=+(S)
/// Flags: N V D I Z C
pub fn plp(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: (S)-:=P
/// Flags:
pub fn php(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}
