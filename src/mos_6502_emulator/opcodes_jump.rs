use crate::mos_6502_emulator::*;

fn branch(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8, do_branch: bool) {
  if do_branch {
    let (address, _) = cpu.get_operand(mode, extra_cycle);
    cpu.pc = address
  } else {
    // Just move the pc forward, but ignore the extra cycles, since the memory
    // won't actually be accessed.
    cpu.get_operand(mode, 0);
  }
}

/// Branch if plus
/// Function: branch on N=0
/// Flags:
pub fn bpl(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  branch(
    cpu,
    mode,
    extra_cycle,
    !cpu.is_status_flag_set(StatusFlag::Negative),
  );
}

/// Branch if minus
/// Function: branch on N=1
/// Flags:
pub fn bmi(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  branch(
    cpu,
    mode,
    extra_cycle,
    cpu.is_status_flag_set(StatusFlag::Negative),
  );
}

/// Branch if Overflow Clear
/// Function: branch on V=0
/// Flags:
pub fn bvc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  branch(
    cpu,
    mode,
    extra_cycle,
    !cpu.is_status_flag_set(StatusFlag::Overflow),
  );
}

/// Branch if Overflow Set
/// Function: branch on V=1
/// Flags:
pub fn bvs(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  branch(
    cpu,
    mode,
    extra_cycle,
    cpu.is_status_flag_set(StatusFlag::Overflow),
  );
}

/// Branch if Carry Clear
/// Function: branch on C=0
/// Flags:
pub fn bcc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  branch(
    cpu,
    mode,
    extra_cycle,
    !cpu.is_status_flag_set(StatusFlag::Carry),
  );
}

/// Branch if Carry Set
/// Function: branch on C=1
/// Flags:
pub fn bcs(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  branch(
    cpu,
    mode,
    extra_cycle,
    cpu.is_status_flag_set(StatusFlag::Carry),
  );
}

/// Branch if Not Equal
/// Function: branch on Z=0
/// Flags:
pub fn bne(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  branch(
    cpu,
    mode,
    extra_cycle,
    !cpu.is_status_flag_set(StatusFlag::Zero),
  );
}

/// Branch if Equal
/// Function: branch on Z=1
/// Flags:
pub fn beq(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  branch(
    cpu,
    mode,
    extra_cycle,
    cpu.is_status_flag_set(StatusFlag::Zero),
  );
}

/// Break - This stops the execution of the program, and saves the PC to the stack.
///         It also sets the status flags so we know what state the CPU is in.
/// Function: (S)-:=PC,P PC:=($FFFE)
/// Flags: B I
pub fn brk(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  cpu.push_stack_u16(cpu.pc);
  cpu.push_stack_u8(cpu.p);
  cpu.pc = InterruptVectors::ResetVector as u16;
  cpu.set_status_flag(StatusFlag::Break, true);
  cpu.set_status_flag(StatusFlag::InterruptDisable, true);
}

/// Return from Interrupt
/// Function: P,PC:=+(S)
/// Flags: N V D I Z C
pub fn rti(cpu: &mut Mos6502Cpu, _mode: Mode, _extra_cycle: u8) {
  cpu.p = cpu.pull_stack_u8();
  cpu.pc = cpu.pull_stack_u16()
}

/// Jump to subroutine
/// Function: (S)-:=PC PC:={adr}
/// Flags:
pub fn jsr(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  cpu.push_stack_u16(cpu.pc);
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.pc = address;
}

/// Return from Sub Routine
/// Function: PC:=+(S)
/// Flags:
pub fn rts(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  cpu.pc = cpu.pull_stack_u16();
}

/// Jump
/// Function: PC:={adr}
/// Flags:
pub fn jmp(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.pc = address;
}

/// Function: N:=b7 V:=b6 Z:=A&{adr}
/// Flags: N V Z
pub fn bit(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: C:=0
/// Flags: C
pub fn clc(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: C:=1
/// Flags: C
pub fn sec(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: D:=0
/// Flags: D
pub fn cld(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: D:=1
/// Flags: D
pub fn sed(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: I:=0
/// Flags: I
pub fn cli(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: I:=1
/// Flags: I
pub fn sei(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: V:=0
/// Flags: V
pub fn clv(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function:
/// Flags:
pub fn nop(cpu: &mut Mos6502Cpu, mode: Mode, extra_cycle: u8) {
  // TODO
  let (address, operand) = cpu.get_operand(mode, extra_cycle);
  cpu.update_zero_and_negative_flag(cpu.a);
}
