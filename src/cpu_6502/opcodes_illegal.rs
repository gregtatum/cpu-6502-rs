use crate::cpu_6502::*;

/// Function: {adr}:={adr}*2 A:=A or {adr}
/// Flags: N Z C
pub fn slo(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    let result_u16 = operand as u16 * 2;
    let result_u8 = result_u16 as u8;
    cpu.bus.borrow_mut().set_u8(address, result_u8);
    cpu.a |= result_u8;
    cpu.update_zero_and_negative_flag(result_u8);
    cpu.update_carry_flag(result_u16);
}

/// Function: {adr}:={adr}rol A:=A and {adr}
/// Flags: N Z C
#[allow(unused)]
pub fn rla(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:={adr}/2 A:=A exor {adr}
/// Flags: N Z C
#[allow(unused)]
pub fn sre(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:={adr}ror A:=A adc {adr}
/// Flags: N V Z C
#[allow(unused)]
pub fn rra(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:=A&X
/// Flags:
#[allow(unused)]
pub fn sax(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A,X:={adr}
/// Flags: N Z
#[allow(unused)]
pub fn lax(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:={adr}-1 A-{adr}
/// Flags: N Z C
#[allow(unused)]
pub fn dcp(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:={adr}+1 A:=A-{adr}
/// Flags: N V Z C
#[allow(unused)]
pub fn isc(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=A&#{imm}
/// Flags: N Z C
#[allow(unused)]
pub fn anc(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=(A&#{imm})/2
/// Flags: N Z C
#[allow(unused)]
pub fn alr(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=(A&#{imm})/2
/// Flags: N V Z C
#[allow(unused)]
pub fn arr(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A:=X&#{imm}
/// Flags: N Z
#[allow(unused)]
pub fn xaa(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: X:=A&X-#{imm}
/// Flags: N Z C
#[allow(unused)]
pub fn axs(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:=A&X&H
/// Flags:
#[allow(unused)]
pub fn ahx(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:=Y&H
/// Flags:
#[allow(unused)]
pub fn shy(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: {adr}:=X&H
/// Flags:
#[allow(unused)]
pub fn shx(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: S:=A&X {adr}:=S&H
/// Flags:
#[allow(unused)]
pub fn tas(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: A,X,S:={adr}&S
/// Flags: N Z
#[allow(unused)]
pub fn las(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}

/// Function: halts the CPU. the data bus will be set to #$FF
/// Flags: N Z
#[allow(unused)]
pub fn kil(cpu: &mut Cpu6502, mode: Mode, extra_cycle: u8) {
    // TODO
    let (address, operand) = cpu.get_address_and_operand(mode, extra_cycle);
    cpu.update_zero_and_negative_flag(cpu.a);
}
