use std::path::Path;

use cpu_6502::{
    asm::{AddressToLabel, AsmLexer, BytesLabels},
    bus::Bus,
    cpu_6502::Cpu6502,
    mappers::SimpleProgram,
    opcodes::OpCode,
};

pub fn load_cpu<P: AsRef<Path>>(filename: P) -> (Cpu6502, AddressToLabel) {
    let contents = std::fs::read_to_string(filename).unwrap();
    let mut lexer = AsmLexer::new(&contents);

    if let Err(parse_error) = lexer.parse() {
        parse_error.panic_nicely();
        panic!("");
    }

    let BytesLabels {
        mut bytes,
        address_to_label,
    } = lexer.into_bytes().unwrap();
    bytes.push(OpCode::KIL as u8);
    (
        Cpu6502::new(Bus::new_shared_bus(Box::new(SimpleProgram::load(&bytes)))),
        address_to_label,
    )
}
