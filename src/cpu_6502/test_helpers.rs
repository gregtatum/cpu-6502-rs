#![macro_use]

use crate::bus::Bus;
use crate::cpu_6502::*;
use crate::{
    asm::{AsmLexer, BytesLabels},
    mappers::SimpleProgram,
};

pub const P: u8 = RESET_STATUS_FLAG;
pub const C: u8 = StatusFlag::Carry as u8;
pub const Z: u8 = StatusFlag::Zero as u8;
pub const I: u8 = StatusFlag::InterruptDisable as u8;
pub const D: u8 = StatusFlag::Decimal as u8;
pub const B: u8 = StatusFlag::Break as u8;
pub const T: u8 = StatusFlag::Push as u8;
pub const V: u8 = StatusFlag::Overflow as u8;
pub const N: u8 = StatusFlag::Negative as u8;

pub fn run_program(text: &str) -> Cpu6502 {
    let mut lexer = AsmLexer::new(text);

    match lexer.parse() {
        Ok(_) => {
            let BytesLabels { mut bytes, .. } = lexer.into_bytes().unwrap();
            bytes.push(OpCode::KIL as u8);
            let mut cpu =
                Cpu6502::new(Bus::new_shared_bus(Box::new(SimpleProgram::load(&bytes))));

            cpu.run();
            cpu
        }
        Err(parse_error) => {
            parse_error.panic_nicely();
            panic!("");
        }
    }
}

/// Run two's complement on a u8.
pub fn negative(n: u8) -> u8 {
    !n + 1
}

pub fn assert_register_a(text: &str, value: u8, status: u8) {
    let cpu = run_program(text);
    if cpu.a != value {
        panic!(
            "\n{}\nExpected register A to be {:#x} ({:#b}) but it was {:#x} ({:#b})",
            text, value, value, cpu.a, cpu.a
        );
    }
    assert_status(&cpu, status);
}

pub fn assert_register_x(text: &str, value: u8, status: u8) {
    let cpu = run_program(text);
    if cpu.x != value {
        panic!(
            "\n{}\nExpected register X to be {:#x} ({:#b}) but it was {:#x} ({:#b})",
            text, value, value, cpu.x, cpu.x
        );
    }
    assert_status(&cpu, status);
}

pub fn assert_register_y(text: &str, value: u8, status: u8) {
    let cpu = run_program(text);
    if cpu.y != value {
        panic!(
            "\n{}\nExpected register X to be {:#x} ({:#b}) but it was {:#x} ({:#b})",
            text, value, value, cpu.x, cpu.x
        );
    }
    assert_status(&cpu, status);
}

pub fn assert_status(cpu: &Cpu6502, value: u8) {
    let mut result = String::new();

    let expected_carry = value & StatusFlag::Carry as u8 == StatusFlag::Carry as u8;
    let expected_zero = value & StatusFlag::Zero as u8 == StatusFlag::Zero as u8;
    let expected_interruptdisable =
        value & StatusFlag::InterruptDisable as u8 == StatusFlag::InterruptDisable as u8;
    let expected_decimal = value & StatusFlag::Decimal as u8 == StatusFlag::Decimal as u8;
    let expected_break = value & StatusFlag::Break as u8 == StatusFlag::Break as u8;
    let expected_push = value & StatusFlag::Push as u8 == StatusFlag::Push as u8;
    let expected_overflow =
        value & StatusFlag::Overflow as u8 == StatusFlag::Overflow as u8;
    let expected_negative =
        value & StatusFlag::Negative as u8 == StatusFlag::Negative as u8;

    let actual_carry = cpu.is_status_flag_set(StatusFlag::Carry);
    let actual_zero = cpu.is_status_flag_set(StatusFlag::Zero);
    let actual_interruptdisable = cpu.is_status_flag_set(StatusFlag::InterruptDisable);
    let actual_decimal = cpu.is_status_flag_set(StatusFlag::Decimal);
    let actual_break = cpu.is_status_flag_set(StatusFlag::Break);
    let actual_push = cpu.is_status_flag_set(StatusFlag::Push);
    let actual_overflow = cpu.is_status_flag_set(StatusFlag::Overflow);
    let actual_negative = cpu.is_status_flag_set(StatusFlag::Negative);

    if expected_carry != actual_carry {
        result.push_str(&format!(
            "Expected StatusFlag::Carry to be {} but received {}\n",
            expected_carry, actual_carry
        ));
    }
    if expected_zero != actual_zero {
        result.push_str(&format!(
            "Expected StatusFlag::Zero to be {} but received {}\n",
            expected_zero, actual_zero
        ));
    }
    if expected_interruptdisable != actual_interruptdisable {
        result.push_str(&format!(
            "Expected StatusFlag::InterruptDisable to be {} but received {}\n",
            expected_interruptdisable, actual_interruptdisable
        ));
    }
    if expected_decimal != actual_decimal {
        result.push_str(&format!(
            "Expected StatusFlag::Decimal to be {} but received {}\n",
            expected_decimal, actual_decimal
        ));
    }
    if expected_break != actual_break {
        result.push_str(&format!(
            "Expected StatusFlag::Break to be {} but received {}\n",
            expected_break, actual_break
        ));
    }
    if expected_push != actual_push {
        result.push_str(&format!(
            "Expected StatusFlag::Push to be {} but received {}\n",
            expected_push, actual_push
        ));
    }
    if expected_overflow != actual_overflow {
        result.push_str(&format!(
            "Expected StatusFlag::Overflow to be {} but received {}\n",
            expected_overflow, actual_overflow
        ));
    }
    if expected_negative != actual_negative {
        result.push_str(&format!(
            "Expected StatusFlag::Negative to be {} but received {}\n",
            expected_negative, actual_negative
        ));
    }

    if cpu.p != value {
        panic!(
            "\nExpected cpu status 0b{:08b} to match 0b{:08b}\n{}",
            cpu.p, value, result
        );
    }
}

#[macro_export]
macro_rules! register_a {
    ($name:ident, $a:expr, $p:expr, $text:expr) => {
        #[test]
        pub fn $name() {
            assert_register_a($text, $a, $p);
        }
    };
}

#[macro_export]
macro_rules! register_x {
    ($name:ident, $x:expr, $p:expr, $text:expr) => {
        #[test]
        fn $name() {
            assert_register_x($text, $x, $p);
        }
    };
}

#[macro_export]
macro_rules! register_y {
    ($name:ident, $y:expr, $p:expr, $text:expr) => {
        #[test]
        fn $name() {
            assert_register_y($text, $y, $p);
        }
    };
}

#[macro_export]
macro_rules! zero_page {
  ($name:ident, [$addr:expr, $expected:expr], $text:expr) => {
    #[test]
    fn $name() {
      let cpu = run_program($text);
      let actual = cpu.bus.read_u8($addr);
      if actual != $expected {
        panic!(
          "\n{}\nExpected zero page address {:#x} to contain {:#x} ({:#b}) but it was {:#x} ({:#b})",
          text, $addr, $expected, $expected, actual, actual
        );
      }
    }
  };
}

#[macro_export]
macro_rules! status {
    ($name:ident, $p:expr, $text:expr) => {
        #[test]
        fn $name() {
            let cpu = run_program($text);
            assert_status(&cpu, $p);
        }
    };
}
