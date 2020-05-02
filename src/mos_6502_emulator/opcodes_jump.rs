use crate::mos_6502_emulator::*;

/// Function: branch on N=0
/// Flags:
pub fn bpl(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: branch on N=1
/// Flags:
pub fn bmi(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: branch on V=0
/// Flags:
pub fn bvc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: branch on V=1
/// Flags:
pub fn bvs(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: branch on C=0
/// Flags:
pub fn bcc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: branch on C=1
/// Flags:
pub fn bcs(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: branch on Z=0
/// Flags:
pub fn bne(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: branch on Z=1
/// Flags:
pub fn beq(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: (S)-:=PC,P PC:=($FFFE)
/// Flags: B I
pub fn brk(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: P,PC:=+(S)
/// Flags: N V D I Z C
pub fn rti(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: (S)-:=PC PC:={adr}
/// Flags:
pub fn jsr(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: PC:=+(S)
/// Flags:
pub fn rts(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: PC:={adr}
/// Flags:
pub fn jmp(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: N:=b7 V:=b6 Z:=A&{adr}
/// Flags: N V Z
pub fn bit(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: C:=0
/// Flags: C
pub fn clc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: C:=1
/// Flags: C
pub fn sec(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: D:=0
/// Flags: D
pub fn cld(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: D:=1
/// Flags: D
pub fn sed(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: I:=0
/// Flags: I
pub fn cli(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: I:=1
/// Flags: I
pub fn sei(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function: V:=0
/// Flags: V
pub fn clv(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}

/// Function:
/// Flags:
pub fn nop(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.accumulator);
}
