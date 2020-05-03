use crate::mos_6502_emulator::*;

/// Function: {adr}:={adr}*2 A:=A or {adr}
/// Flags: N Z C
pub fn slo(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  let result_u16 = operand as u16 * 2;
  let result_u8 = result_u16 as u8;
  cpu.bus.set_u8(address, result_u8);
  cpu.a = cpu.a | result_u8;
  cpu.update_zero_and_negative_flag(result_u8);
  cpu.update_carry_flag(result_u16);
}

/// Function: {adr}:={adr}rol A:=A and {adr}
/// Flags: N Z C
pub fn rla(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:={adr}/2 A:=A exor {adr}
/// Flags: N Z C
pub fn sre(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:={adr}ror A:=A adc {adr}
/// Flags: N V Z C
pub fn rra(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:=A&X
/// Flags:
pub fn sax(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A,X:={adr}
/// Flags: N Z
pub fn lax(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:={adr}-1 A-{adr}
/// Flags: N Z C
pub fn dcp(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:={adr}+1 A:=A-{adr}
/// Flags: N V Z C
pub fn isc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=A&#{imm}
/// Flags: N Z C
pub fn anc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=(A&#{imm})/2
/// Flags: N Z C
pub fn alr(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=(A&#{imm})/2
/// Flags: N V Z C
pub fn arr(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=X&#{imm}
/// Flags: N Z
pub fn xaa(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: X:=A&X-#{imm}
/// Flags: N Z C
pub fn axs(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:=A&X&H
/// Flags:
pub fn ahx(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:=Y&H
/// Flags:
pub fn shy(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:=X&H
/// Flags:
pub fn shx(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: S:=A&X {adr}:=S&H
/// Flags:
pub fn tas(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A,X,S:={adr}&S
/// Flags: N Z
pub fn las(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: halts the CPU. the data bus will be set to #$FF
/// Flags: N Z
pub fn kil(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}
