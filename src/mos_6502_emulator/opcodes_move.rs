use crate::mos_6502_emulator::*;

/// Load the value into register A
/// Function: A:={adr}
/// Flags: N Z
pub fn lda(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (_address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.a = operand;
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Store register A at address
/// Function: {adr}:=A
/// Flags:
pub fn sta(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, _operand) = cpu.get_operand(mode, extra_cycle);
  cpu.bus.set_u8(address, cpu.a);
}

/// Load register X with the value
/// Function: X:={adr}
/// Flags: N Z
pub fn ldx(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (_address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.x = operand;
  cpu.update_zero_and_negative_flag(cpu.x);
}

/// Store register X at address
/// Function: {adr}:=X
/// Flags:
pub fn stx(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, _operand) = cpu.get_operand(mode, extra_cycle);
  cpu.bus.set_u8(address, cpu.x);
}

/// Load register Y with the value
/// Function: Y:={adr}
/// Flags: N Z
pub fn ldy(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (_address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.y = operand;
  cpu.update_zero_and_negative_flag(cpu.y);
}

/// Store register Y at address
/// Function: {adr}:=Y
/// Flags:
pub fn sty(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, _operand) = cpu.get_operand(mode, extra_cycle);
  cpu.bus.set_u8(address, cpu.y);
}

/// Transfer A to X
/// Function: X:=A
/// Flags: N Z
pub fn tax(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.x = cpu.a;
  cpu.update_zero_and_negative_flag(cpu.x)
}

/// Transfer X to A
/// Function: A:=X
/// Flags: N Z
pub fn txa(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.a = cpu.x;
  cpu.update_zero_and_negative_flag(cpu.a)
}

/// Transfer A to Y
/// Function: Y:=A
/// Flags: N Z
pub fn tay(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.y = cpu.a;
  cpu.update_zero_and_negative_flag(cpu.y)
}

/// Transfer Y to A
/// Function: A:=Y
/// Flags: N Z
pub fn tya(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.a = cpu.y;
  cpu.update_zero_and_negative_flag(cpu.a)
}

/// Transfer S to X
/// Function: X:=S
/// Flags: N Z
pub fn tsx(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.x = cpu.s;
  cpu.update_zero_and_negative_flag(cpu.x)
}

/// Transfer X to S
/// Function: S:=X
/// Flags:
pub fn txs(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.s = cpu.x;
  cpu.update_zero_and_negative_flag(cpu.s)
}

/// Pull A
/// Function: A:=+(S)
/// Flags: N Z
pub fn pla(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.a = cpu.pull_stack_u8();
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Push A to the stack
/// Function: (S)-:=A
/// Flags:
pub fn pha(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.push_stack_u8(cpu.a);
}

/// Pull the status register from the stack
/// Function: P:=+(S)
/// Flags: N V D I Z C
pub fn plp(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.p = cpu.pull_stack_u8();
}

/// Push the status register to the stack
/// Function: (S)-:=P
/// Flags:
pub fn php(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.push_stack_u8(cpu.p);
}
