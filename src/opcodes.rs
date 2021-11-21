use crate::cpu_6502::opcodes_illegal::*;
use crate::cpu_6502::opcodes_jump::*;
use crate::cpu_6502::opcodes_logical::*;
use crate::cpu_6502::opcodes_move::*;
use crate::cpu_6502::Cpu6502;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Absolute,         // abs
    AbsoluteIndexedX, // abx
    AbsoluteIndexedY, // aby
    Immediate,        // imm
    Implied,          // imp
    Indirect,         // ind
    IndirectX,        // izx
    IndirectY,        // izy
    Relative,         // rel
    RegisterA,        // a
    ZeroPage,         // zp
    ZeroPageX,        // zpx
    ZeroPageY,        // zpy
    None,             // non - This last one is fake.
}

/**
 * Tokens don't necessarily have enough information to know the mode.
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenMode {
    Absolute,           // abs
    AbsoluteIndexedX,   // abx
    AbsoluteIndexedY,   // aby
    Immediate,          // imm
    Implied,            // imp
    Indirect,           // ind
    IndirectX,          // izx
    IndirectY,          // izy
    RegisterA,          // a
    Relative,           // rel
    ZeroPageOrRelative, // zp,rel
    ZeroPageX,          // zpx
    ZeroPageY,          // zpy
    None,               // non - This last one is fake.
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Logical and arithmetic commands:
    ORA,
    AND,
    EOR,
    ADC,
    SBC,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    INC,
    INX,
    INY,
    ASL,
    ROL,
    LSR,
    ROR,
    // Move commands
    LDA,
    STA,
    LDX,
    STX,
    LDY,
    STY,
    TAX,
    TXA,
    TAY,
    TYA,
    TSX,
    TXS,
    PLA,
    PHA,
    PLP,
    PHP,
    // Jump / Flag commands
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,
    BRK,
    RTI,
    JSR,
    RTS,
    JMP,
    BIT,
    CLC,
    SEC,
    CLD,
    SED,
    CLI,
    SEI,
    CLV,
    NOP,
    // Illegal
    SLO,
    RLA,
    SRE,
    RRA,
    SAX,
    LAX,
    DCP,
    ISC,
    ANC,
    ALR,
    ARR,
    XAA,
    AXS,
    AHX,
    SHY,
    SHX,
    TAS,
    LAS,
    KIL,
}

pub fn match_instruction(string: &str) -> Option<Instruction> {
    let instruction = match string.to_lowercase().as_ref() {
        "ora" => Instruction::ORA,
        "and" => Instruction::AND,
        "eor" => Instruction::EOR,
        "adc" => Instruction::ADC,
        "sbc" => Instruction::SBC,
        "cmp" => Instruction::CMP,
        "cpx" => Instruction::CPX,
        "cpy" => Instruction::CPY,
        "dec" => Instruction::DEC,
        "dex" => Instruction::DEX,
        "dey" => Instruction::DEY,
        "inc" => Instruction::INC,
        "inx" => Instruction::INX,
        "iny" => Instruction::INY,
        "asl" => Instruction::ASL,
        "rol" => Instruction::ROL,
        "lsr" => Instruction::LSR,
        "ror" => Instruction::ROR,
        "lda" => Instruction::LDA,
        "sta" => Instruction::STA,
        "ldx" => Instruction::LDX,
        "stx" => Instruction::STX,
        "ldy" => Instruction::LDY,
        "sty" => Instruction::STY,
        "tax" => Instruction::TAX,
        "txa" => Instruction::TXA,
        "tay" => Instruction::TAY,
        "tya" => Instruction::TYA,
        "tsx" => Instruction::TSX,
        "txs" => Instruction::TXS,
        "pla" => Instruction::PLA,
        "pha" => Instruction::PHA,
        "plp" => Instruction::PLP,
        "php" => Instruction::PHP,
        "bpl" => Instruction::BPL,
        "bmi" => Instruction::BMI,
        "bvc" => Instruction::BVC,
        "bvs" => Instruction::BVS,
        "bcc" => Instruction::BCC,
        "bcs" => Instruction::BCS,
        "bne" => Instruction::BNE,
        "beq" => Instruction::BEQ,
        "brk" => Instruction::BRK,
        "rti" => Instruction::RTI,
        "jsr" => Instruction::JSR,
        "rts" => Instruction::RTS,
        "jmp" => Instruction::JMP,
        "bit" => Instruction::BIT,
        "clc" => Instruction::CLC,
        "sec" => Instruction::SEC,
        "cld" => Instruction::CLD,
        "sed" => Instruction::SED,
        "cli" => Instruction::CLI,
        "sei" => Instruction::SEI,
        "clv" => Instruction::CLV,
        "nop" => Instruction::NOP,
        "kil" => Instruction::KIL,
        _ => return None,
    };
    Some(instruction)
}

#[allow(non_camel_case_types)]
pub enum OpCode {
    BRK = 0x00,
    ORA_izx = 0x01,
    KIL = 0x02,
    SLO_izx = 0x03,
    NOP_zp = 0x04,
    ORA_zp = 0x05,
    ASL_zp = 0x06,
    SLO_zp = 0x07,
    PHP = 0x08,
    ORA_imm = 0x09,
    ASL_a = 0x0a,
    ANC_imm = 0x0b,
    NOP_abs = 0x0c,
    ORA_abs = 0x0d,
    ASL_abs = 0x0e,
    SLO_abs = 0x0f,
    BPL_rel = 0x10,
    ORA_izy = 0x11,
    KIL1 = 0x12,
    SLO_izy = 0x13,
    NOP_zpx = 0x14,
    ORA_zpx = 0x15,
    ASL_zpx = 0x16,
    SLO_zpx = 0x17,
    CLC = 0x18,
    ORA_aby = 0x19,
    NOP = 0x1a,
    SLO_aby = 0x1b,
    NOP_abx = 0x1c,
    ORA_abx = 0x1d,
    ASL_abx = 0x1e,
    SLO_abx = 0x1f,
    JSR_abs = 0x20,
    AND_izx = 0x21,
    KIL2 = 0x22,
    RLA_izx = 0x23,
    BIT_zp = 0x24,
    AND_zp = 0x25,
    ROL_zp = 0x26,
    RLA_zp = 0x27,
    PLP = 0x28,
    AND_imm = 0x29,
    ROL_a = 0x2a,
    ANC_imm1 = 0x2b,
    BIT_abs = 0x2c,
    AND_abs = 0x2d,
    ROL_abs = 0x2e,
    RLA_abs = 0x2f,
    BMI_rel = 0x30,
    AND_izy = 0x31,
    KIL3 = 0x32,
    RLA_izy = 0x33,
    NOP_zpx1 = 0x34,
    AND_zpx = 0x35,
    ROL_zpx = 0x36,
    RLA_zpx = 0x37,
    SEC = 0x38,
    AND_aby = 0x39,
    NOP4 = 0x3a,
    RLA_aby = 0x3b,
    NOP_abx1 = 0x3c,
    AND_abx = 0x3d,
    ROL_abx = 0x3e,
    RLA_abx = 0x3f,
    RTI = 0x40,
    EOR_izx = 0x41,
    KIL4 = 0x42,
    SRE_izx = 0x43,
    NOP_zp1 = 0x44,
    EOR_zp = 0x45,
    LSR_zp = 0x46,
    SRE_zp = 0x47,
    PHA = 0x48,
    EOR_imm = 0x49,
    LSR_a = 0x4a,
    ALR_imm = 0x4b,
    JMP_abs = 0x4c,
    EOR_abs = 0x4d,
    LSR_abs = 0x4e,
    SRE_abs = 0x4f,
    BVC_rel = 0x50,
    EOR_izy = 0x51,
    KIL5 = 0x52,
    SRE_izy = 0x53,
    NOP_zpx2 = 0x54,
    EOR_zpx = 0x55,
    LSR_zpx = 0x56,
    SRE_zpx = 0x57,
    CLI = 0x58,
    EOR_aby = 0x59,
    NOP3 = 0x5a,
    SRE_aby = 0x5b,
    NOP_abx2 = 0x5c,
    EOR_abx = 0x5d,
    LSR_abx = 0x5e,
    SRE_abx = 0x5f,
    RTS = 0x60,
    ADC_izx = 0x61,
    KIL6 = 0x62,
    RRA_izx = 0x63,
    NOP_zp2 = 0x64,
    ADC_zp = 0x65,
    ROR_zp = 0x66,
    RRA_zp = 0x67,
    PLA = 0x68,
    ADC_imm = 0x69,
    ROR_a = 0x6a,
    ARR_imm = 0x6b,
    JMP_ind = 0x6c,
    ADC_abs = 0x6d,
    ROR_abs = 0x6e,
    RRA_abs = 0x6f,
    BVS_rel = 0x70,
    ADC_izy = 0x71,
    KIL7 = 0x72,
    RRA_izy = 0x73,
    NOP_zpx3 = 0x74,
    ADC_zpx = 0x75,
    ROR_zpx = 0x76,
    RRA_zpx = 0x77,
    SEI = 0x78,
    ADC_aby = 0x79,
    NOP8 = 0x7a,
    RRA_aby = 0x7b,
    NOP_abx3 = 0x7c,
    ADC_abx = 0x7d,
    ROR_abx = 0x7e,
    RRA_abx = 0x7f,
    NOP_imm = 0x80,
    STA_izx = 0x81,
    NOP_imm1 = 0x82,
    SAX_izx = 0x83,
    STY_zp = 0x84,
    STA_zp = 0x85,
    STX_zp = 0x86,
    SAX_zp = 0x87,
    DEY = 0x88,
    NOP_imm2 = 0x89,
    TXA = 0x8a,
    XAA_imm = 0x8b,
    STY_abs = 0x8c,
    STA_abs = 0x8d,
    STX_abs = 0x8e,
    SAX_abs = 0x8f,
    BCC_rel = 0x90,
    STA_izy = 0x91,
    KIL8 = 0x92,
    AHX_izy = 0x93,
    STY_zpx = 0x94,
    STA_zpx = 0x95,
    STX_zpy = 0x96,
    SAX_zpy = 0x97,
    TYA = 0x98,
    STA_aby = 0x99,
    TXS = 0x9a,
    TAS_aby = 0x9b,
    SHY_abx = 0x9c,
    STA_abx = 0x9d,
    SHX_aby = 0x9e,
    AHX_aby = 0x9f,
    LDY_imm = 0xa0,
    LDA_izx = 0xa1,
    LDX_imm = 0xa2,
    LAX_izx = 0xa3,
    LDY_zp = 0xa4,
    LDA_zp = 0xa5,
    LDX_zp = 0xa6,
    LAX_zp = 0xa7,
    TAY = 0xa8,
    LDA_imm = 0xa9,
    TAX = 0xaa,
    LAX_imm = 0xab,
    LDY_abs = 0xac,
    LDA_abs = 0xad,
    LDX_abs = 0xae,
    LAX_abs = 0xaf,
    BCS_rel = 0xb0,
    LDA_izy = 0xb1,
    KIL9 = 0xb2,
    LAX_izy = 0xb3,
    LDY_zpx = 0xb4,
    LDA_zpx = 0xb5,
    LDX_zpy = 0xb6,
    LAX_zpy = 0xb7,
    CLV = 0xb8,
    LDA_aby = 0xb9,
    TSX = 0xba,
    LAS_aby = 0xbb,
    LDY_abx = 0xbc,
    LDA_abx = 0xbd,
    LDX_aby = 0xbe,
    LAX_aby = 0xbf,
    CPY_imm = 0xc0,
    CMP_izx = 0xc1,
    NOP_imm3 = 0xc2,
    DCP_izx = 0xc3,
    CPY_zp = 0xc4,
    CMP_zp = 0xc5,
    DEC_zp = 0xc6,
    DCP_zp = 0xc7,
    INY = 0xc8,
    CMP_imm = 0xc9,
    DEX = 0xca,
    AXS_imm = 0xcb,
    CPY_abs = 0xcc,
    CMP_abs = 0xcd,
    DEC_abs = 0xce,
    DCP_abs = 0xcf,
    BNE_rel = 0xd0,
    CMP_izy = 0xd1,
    KIL10 = 0xd2,
    DCP_izy = 0xd3,
    NOP_zpx4 = 0xd4,
    CMP_zpx = 0xd5,
    DEC_zpx = 0xd6,
    DCP_zpx = 0xd7,
    CLD = 0xd8,
    CMP_aby = 0xd9,
    NOP1 = 0xda,
    DCP_aby = 0xdb,
    NOP_abx4 = 0xdc,
    CMP_abx = 0xdd,
    DEC_abx = 0xde,
    DCP_abx = 0xdf,
    CPX_imm = 0xe0,
    SBC_izx = 0xe1,
    NOP_imm4 = 0xe2,
    ISC_izx = 0xe3,
    CPX_zp = 0xe4,
    SBC_zp = 0xe5,
    INC_zp = 0xe6,
    ISC_zp = 0xe7,
    INX = 0xe8,
    SBC_imm = 0xe9,
    NOP5 = 0xea,
    SBC_imm1 = 0xeb,
    CPX_abs = 0xec,
    SBC_abs = 0xed,
    INC_abs = 0xee,
    ISC_abs = 0xef,
    BEQ_rel = 0xf0,
    SBC_izy = 0xf1,
    KIL11 = 0xf2,
    ISC_izy = 0xf3,
    NOP_zpx5 = 0xf4,
    SBC_zpx = 0xf5,
    INC_zpx = 0xf6,
    ISC_zpx = 0xf7,
    SED = 0xf8,
    SBC_aby = 0xf9,
    NOP6 = 0xfa,
    ISC_aby = 0xfb,
    NOP_abx5 = 0xfc,
    SBC_abx = 0xfd,
    INC_abx = 0xfe,
    ISC_abx = 0xff,
}

pub fn instruction_mode_to_op_code(
    instruction: &Instruction,
    mode: &TokenMode,
) -> Result<OpCode, String> {
    Ok(match (instruction, mode) {
        (Instruction::ADC, TokenMode::Absolute) => OpCode::ADC_abs,
        (Instruction::ADC, TokenMode::AbsoluteIndexedX) => OpCode::ADC_abx,
        (Instruction::ADC, TokenMode::AbsoluteIndexedY) => OpCode::ADC_aby,
        (Instruction::ADC, TokenMode::Immediate) => OpCode::ADC_imm,
        (Instruction::ADC, TokenMode::IndirectX) => OpCode::ADC_izx,
        (Instruction::ADC, TokenMode::IndirectY) => OpCode::ADC_izy,
        (Instruction::ADC, TokenMode::ZeroPageOrRelative) => OpCode::ADC_zp,
        (Instruction::ADC, TokenMode::ZeroPageX) => OpCode::ADC_zpx,
        (Instruction::AHX, TokenMode::AbsoluteIndexedY) => OpCode::AHX_aby,
        (Instruction::AHX, TokenMode::IndirectY) => OpCode::AHX_izy,
        (Instruction::ALR, TokenMode::Immediate) => OpCode::ALR_imm,
        (Instruction::ANC, TokenMode::Immediate) => OpCode::ANC_imm,
        (Instruction::AND, TokenMode::Absolute) => OpCode::AND_abs,
        (Instruction::AND, TokenMode::AbsoluteIndexedX) => OpCode::AND_abx,
        (Instruction::AND, TokenMode::AbsoluteIndexedY) => OpCode::AND_aby,
        (Instruction::AND, TokenMode::Immediate) => OpCode::AND_imm,
        (Instruction::AND, TokenMode::IndirectX) => OpCode::AND_izx,
        (Instruction::AND, TokenMode::IndirectY) => OpCode::AND_izy,
        (Instruction::AND, TokenMode::ZeroPageOrRelative) => OpCode::AND_zp,
        (Instruction::AND, TokenMode::ZeroPageX) => OpCode::AND_zpx,
        (Instruction::ARR, TokenMode::Immediate) => OpCode::ARR_imm,
        (Instruction::ASL, TokenMode::None) => OpCode::ASL_a,
        (Instruction::ASL, TokenMode::RegisterA) => OpCode::ASL_a,
        (Instruction::ASL, TokenMode::Absolute) => OpCode::ASL_abs,
        (Instruction::ASL, TokenMode::AbsoluteIndexedX) => OpCode::ASL_abx,
        (Instruction::ASL, TokenMode::ZeroPageOrRelative) => OpCode::ASL_zp,
        (Instruction::ASL, TokenMode::ZeroPageX) => OpCode::ASL_zpx,
        (Instruction::AXS, TokenMode::Immediate) => OpCode::AXS_imm,
        (Instruction::BCC, TokenMode::ZeroPageOrRelative) => OpCode::BCC_rel,
        (Instruction::BCC, TokenMode::Relative) => OpCode::BCC_rel,
        (Instruction::BCS, TokenMode::ZeroPageOrRelative) => OpCode::BCS_rel,
        (Instruction::BCS, TokenMode::Relative) => OpCode::BCS_rel,
        (Instruction::BEQ, TokenMode::ZeroPageOrRelative) => OpCode::BEQ_rel,
        (Instruction::BEQ, TokenMode::Relative) => OpCode::BEQ_rel,
        (Instruction::BIT, TokenMode::Absolute) => OpCode::BIT_abs,
        (Instruction::BIT, TokenMode::ZeroPageOrRelative) => OpCode::BIT_zp,
        (Instruction::BMI, TokenMode::ZeroPageOrRelative) => OpCode::BMI_rel,
        (Instruction::BMI, TokenMode::Relative) => OpCode::BMI_rel,
        (Instruction::BNE, TokenMode::ZeroPageOrRelative) => OpCode::BNE_rel,
        (Instruction::BNE, TokenMode::Relative) => OpCode::BNE_rel,
        (Instruction::BPL, TokenMode::ZeroPageOrRelative) => OpCode::BPL_rel,
        (Instruction::BPL, TokenMode::Relative) => OpCode::BPL_rel,
        (Instruction::BRK, TokenMode::None) => OpCode::BRK,
        (Instruction::BVC, TokenMode::ZeroPageOrRelative) => OpCode::BVC_rel,
        (Instruction::BVC, TokenMode::Relative) => OpCode::BVC_rel,
        (Instruction::BVS, TokenMode::ZeroPageOrRelative) => OpCode::BVS_rel,
        (Instruction::BVS, TokenMode::Relative) => OpCode::BVS_rel,
        (Instruction::CLC, TokenMode::None) => OpCode::CLC,
        (Instruction::CLD, TokenMode::None) => OpCode::CLD,
        (Instruction::CLI, TokenMode::None) => OpCode::CLI,
        (Instruction::CLV, TokenMode::None) => OpCode::CLV,
        (Instruction::CMP, TokenMode::Absolute) => OpCode::CMP_abs,
        (Instruction::CMP, TokenMode::AbsoluteIndexedX) => OpCode::CMP_abx,
        (Instruction::CMP, TokenMode::AbsoluteIndexedY) => OpCode::CMP_aby,
        (Instruction::CMP, TokenMode::Immediate) => OpCode::CMP_imm,
        (Instruction::CMP, TokenMode::IndirectX) => OpCode::CMP_izx,
        (Instruction::CMP, TokenMode::IndirectY) => OpCode::CMP_izy,
        (Instruction::CMP, TokenMode::ZeroPageOrRelative) => OpCode::CMP_zp,
        (Instruction::CMP, TokenMode::ZeroPageX) => OpCode::CMP_zpx,
        (Instruction::CPX, TokenMode::Absolute) => OpCode::CPX_abs,
        (Instruction::CPX, TokenMode::Immediate) => OpCode::CPX_imm,
        (Instruction::CPX, TokenMode::ZeroPageOrRelative) => OpCode::CPX_zp,
        (Instruction::CPY, TokenMode::Absolute) => OpCode::CPY_abs,
        (Instruction::CPY, TokenMode::Immediate) => OpCode::CPY_imm,
        (Instruction::CPY, TokenMode::ZeroPageOrRelative) => OpCode::CPY_zp,
        (Instruction::DCP, TokenMode::Absolute) => OpCode::DCP_abs,
        (Instruction::DCP, TokenMode::AbsoluteIndexedX) => OpCode::DCP_abx,
        (Instruction::DCP, TokenMode::AbsoluteIndexedY) => OpCode::DCP_aby,
        (Instruction::DCP, TokenMode::IndirectX) => OpCode::DCP_izx,
        (Instruction::DCP, TokenMode::IndirectY) => OpCode::DCP_izy,
        (Instruction::DCP, TokenMode::ZeroPageOrRelative) => OpCode::DCP_zp,
        (Instruction::DCP, TokenMode::ZeroPageX) => OpCode::DCP_zpx,
        (Instruction::DEC, TokenMode::Absolute) => OpCode::DEC_abs,
        (Instruction::DEC, TokenMode::AbsoluteIndexedX) => OpCode::DEC_abx,
        (Instruction::DEC, TokenMode::ZeroPageOrRelative) => OpCode::DEC_zp,
        (Instruction::DEC, TokenMode::ZeroPageX) => OpCode::DEC_zpx,
        (Instruction::DEX, TokenMode::None) => OpCode::DEX,
        (Instruction::DEY, TokenMode::None) => OpCode::DEY,
        (Instruction::EOR, TokenMode::Absolute) => OpCode::EOR_abs,
        (Instruction::EOR, TokenMode::AbsoluteIndexedX) => OpCode::EOR_abx,
        (Instruction::EOR, TokenMode::AbsoluteIndexedY) => OpCode::EOR_aby,
        (Instruction::EOR, TokenMode::Immediate) => OpCode::EOR_imm,
        (Instruction::EOR, TokenMode::IndirectX) => OpCode::EOR_izx,
        (Instruction::EOR, TokenMode::IndirectY) => OpCode::EOR_izy,
        (Instruction::EOR, TokenMode::ZeroPageOrRelative) => OpCode::EOR_zp,
        (Instruction::EOR, TokenMode::ZeroPageX) => OpCode::EOR_zpx,
        (Instruction::INC, TokenMode::Absolute) => OpCode::INC_abs,
        (Instruction::INC, TokenMode::AbsoluteIndexedX) => OpCode::INC_abx,
        (Instruction::INC, TokenMode::ZeroPageOrRelative) => OpCode::INC_zp,
        (Instruction::INC, TokenMode::ZeroPageX) => OpCode::INC_zpx,
        (Instruction::INX, TokenMode::None) => OpCode::INX,
        (Instruction::INY, TokenMode::None) => OpCode::INY,
        (Instruction::ISC, TokenMode::Absolute) => OpCode::ISC_abs,
        (Instruction::ISC, TokenMode::AbsoluteIndexedX) => OpCode::ISC_abx,
        (Instruction::ISC, TokenMode::AbsoluteIndexedY) => OpCode::ISC_aby,
        (Instruction::ISC, TokenMode::IndirectX) => OpCode::ISC_izx,
        (Instruction::ISC, TokenMode::IndirectY) => OpCode::ISC_izy,
        (Instruction::ISC, TokenMode::ZeroPageOrRelative) => OpCode::ISC_zp,
        (Instruction::ISC, TokenMode::ZeroPageX) => OpCode::ISC_zpx,
        (Instruction::JMP, TokenMode::Absolute) => OpCode::JMP_abs,
        (Instruction::JMP, TokenMode::Indirect) => OpCode::JMP_ind,
        (Instruction::JSR, TokenMode::Absolute) => OpCode::JSR_abs,
        (Instruction::KIL, TokenMode::None) => OpCode::KIL,
        (Instruction::LAS, TokenMode::AbsoluteIndexedY) => OpCode::LAS_aby,
        (Instruction::LAX, TokenMode::Absolute) => OpCode::LAX_abs,
        (Instruction::LAX, TokenMode::AbsoluteIndexedY) => OpCode::LAX_aby,
        (Instruction::LAX, TokenMode::Immediate) => OpCode::LAX_imm,
        (Instruction::LAX, TokenMode::IndirectX) => OpCode::LAX_izx,
        (Instruction::LAX, TokenMode::IndirectY) => OpCode::LAX_izy,
        (Instruction::LAX, TokenMode::ZeroPageOrRelative) => OpCode::LAX_zp,
        (Instruction::LAX, TokenMode::ZeroPageY) => OpCode::LAX_zpy,
        (Instruction::LDA, TokenMode::Absolute) => OpCode::LDA_abs,
        (Instruction::LDA, TokenMode::AbsoluteIndexedX) => OpCode::LDA_abx,
        (Instruction::LDA, TokenMode::AbsoluteIndexedY) => OpCode::LDA_aby,
        (Instruction::LDA, TokenMode::Immediate) => OpCode::LDA_imm,
        (Instruction::LDA, TokenMode::IndirectX) => OpCode::LDA_izx,
        (Instruction::LDA, TokenMode::IndirectY) => OpCode::LDA_izy,
        (Instruction::LDA, TokenMode::ZeroPageOrRelative) => OpCode::LDA_zp,
        (Instruction::LDA, TokenMode::ZeroPageX) => OpCode::LDA_zpx,
        (Instruction::LDX, TokenMode::Absolute) => OpCode::LDX_abs,
        (Instruction::LDX, TokenMode::AbsoluteIndexedY) => OpCode::LDX_aby,
        (Instruction::LDX, TokenMode::Immediate) => OpCode::LDX_imm,
        (Instruction::LDX, TokenMode::ZeroPageOrRelative) => OpCode::LDX_zp,
        (Instruction::LDX, TokenMode::ZeroPageY) => OpCode::LDX_zpy,
        (Instruction::LDY, TokenMode::Absolute) => OpCode::LDY_abs,
        (Instruction::LDY, TokenMode::AbsoluteIndexedX) => OpCode::LDY_abx,
        (Instruction::LDY, TokenMode::Immediate) => OpCode::LDY_imm,
        (Instruction::LDY, TokenMode::ZeroPageOrRelative) => OpCode::LDY_zp,
        (Instruction::LDY, TokenMode::ZeroPageX) => OpCode::LDY_zpx,
        (Instruction::LSR, TokenMode::None) => OpCode::LSR_a,
        (Instruction::LSR, TokenMode::RegisterA) => OpCode::LSR_a,
        (Instruction::LSR, TokenMode::Absolute) => OpCode::LSR_abs,
        (Instruction::LSR, TokenMode::AbsoluteIndexedX) => OpCode::LSR_abx,
        (Instruction::LSR, TokenMode::ZeroPageOrRelative) => OpCode::LSR_zp,
        (Instruction::LSR, TokenMode::ZeroPageX) => OpCode::LSR_zpx,
        (Instruction::NOP, TokenMode::None) => OpCode::NOP,
        (Instruction::NOP, TokenMode::Absolute) => OpCode::NOP_abs,
        (Instruction::NOP, TokenMode::AbsoluteIndexedX) => OpCode::NOP_abx,
        (Instruction::NOP, TokenMode::Immediate) => OpCode::NOP_imm,
        (Instruction::NOP, TokenMode::ZeroPageOrRelative) => OpCode::NOP_zp,
        (Instruction::NOP, TokenMode::ZeroPageX) => OpCode::NOP_zpx,
        (Instruction::ORA, TokenMode::Absolute) => OpCode::ORA_abs,
        (Instruction::ORA, TokenMode::AbsoluteIndexedX) => OpCode::ORA_abx,
        (Instruction::ORA, TokenMode::AbsoluteIndexedY) => OpCode::ORA_aby,
        (Instruction::ORA, TokenMode::Immediate) => OpCode::ORA_imm,
        (Instruction::ORA, TokenMode::IndirectX) => OpCode::ORA_izx,
        (Instruction::ORA, TokenMode::IndirectY) => OpCode::ORA_izy,
        (Instruction::ORA, TokenMode::ZeroPageOrRelative) => OpCode::ORA_zp,
        (Instruction::ORA, TokenMode::ZeroPageX) => OpCode::ORA_zpx,
        (Instruction::PHA, TokenMode::None) => OpCode::PHA,
        (Instruction::PHP, TokenMode::None) => OpCode::PHP,
        (Instruction::PLA, TokenMode::None) => OpCode::PLA,
        (Instruction::PLP, TokenMode::None) => OpCode::PLP,
        (Instruction::RLA, TokenMode::Absolute) => OpCode::RLA_abs,
        (Instruction::RLA, TokenMode::AbsoluteIndexedX) => OpCode::RLA_abx,
        (Instruction::RLA, TokenMode::AbsoluteIndexedY) => OpCode::RLA_aby,
        (Instruction::RLA, TokenMode::IndirectX) => OpCode::RLA_izx,
        (Instruction::RLA, TokenMode::IndirectY) => OpCode::RLA_izy,
        (Instruction::RLA, TokenMode::ZeroPageOrRelative) => OpCode::RLA_zp,
        (Instruction::RLA, TokenMode::ZeroPageX) => OpCode::RLA_zpx,
        (Instruction::ROL, TokenMode::None) => OpCode::ROL_a,
        (Instruction::ROL, TokenMode::RegisterA) => OpCode::ROL_a,
        (Instruction::ROL, TokenMode::Absolute) => OpCode::ROL_abs,
        (Instruction::ROL, TokenMode::AbsoluteIndexedX) => OpCode::ROL_abx,
        (Instruction::ROL, TokenMode::ZeroPageOrRelative) => OpCode::ROL_zp,
        (Instruction::ROL, TokenMode::ZeroPageX) => OpCode::ROL_zpx,
        (Instruction::ROR, TokenMode::None) => OpCode::ROR_a,
        (Instruction::ROR, TokenMode::RegisterA) => OpCode::ROR_a,
        (Instruction::ROR, TokenMode::Absolute) => OpCode::ROR_abs,
        (Instruction::ROR, TokenMode::AbsoluteIndexedX) => OpCode::ROR_abx,
        (Instruction::ROR, TokenMode::ZeroPageOrRelative) => OpCode::ROR_zp,
        (Instruction::ROR, TokenMode::ZeroPageX) => OpCode::ROR_zpx,
        (Instruction::RRA, TokenMode::Absolute) => OpCode::RRA_abs,
        (Instruction::RRA, TokenMode::AbsoluteIndexedX) => OpCode::RRA_abx,
        (Instruction::RRA, TokenMode::AbsoluteIndexedY) => OpCode::RRA_aby,
        (Instruction::RRA, TokenMode::IndirectX) => OpCode::RRA_izx,
        (Instruction::RRA, TokenMode::IndirectY) => OpCode::RRA_izy,
        (Instruction::RRA, TokenMode::ZeroPageOrRelative) => OpCode::RRA_zp,
        (Instruction::RRA, TokenMode::ZeroPageX) => OpCode::RRA_zpx,
        (Instruction::RTI, TokenMode::None) => OpCode::RTI,
        (Instruction::RTS, TokenMode::None) => OpCode::RTS,
        (Instruction::SAX, TokenMode::Absolute) => OpCode::SAX_abs,
        (Instruction::SAX, TokenMode::IndirectX) => OpCode::SAX_izx,
        (Instruction::SAX, TokenMode::ZeroPageOrRelative) => OpCode::SAX_zp,
        (Instruction::SAX, TokenMode::ZeroPageY) => OpCode::SAX_zpy,
        (Instruction::SBC, TokenMode::Absolute) => OpCode::SBC_abs,
        (Instruction::SBC, TokenMode::AbsoluteIndexedX) => OpCode::SBC_abx,
        (Instruction::SBC, TokenMode::AbsoluteIndexedY) => OpCode::SBC_aby,
        (Instruction::SBC, TokenMode::Immediate) => OpCode::SBC_imm,
        (Instruction::SBC, TokenMode::IndirectX) => OpCode::SBC_izx,
        (Instruction::SBC, TokenMode::IndirectY) => OpCode::SBC_izy,
        (Instruction::SBC, TokenMode::ZeroPageOrRelative) => OpCode::SBC_zp,
        (Instruction::SBC, TokenMode::ZeroPageX) => OpCode::SBC_zpx,
        (Instruction::SEC, TokenMode::None) => OpCode::SEC,
        (Instruction::SED, TokenMode::None) => OpCode::SED,
        (Instruction::SEI, TokenMode::None) => OpCode::SEI,
        (Instruction::SHX, TokenMode::AbsoluteIndexedY) => OpCode::SHX_aby,
        (Instruction::SHY, TokenMode::AbsoluteIndexedX) => OpCode::SHY_abx,
        (Instruction::SLO, TokenMode::Absolute) => OpCode::SLO_abs,
        (Instruction::SLO, TokenMode::AbsoluteIndexedX) => OpCode::SLO_abx,
        (Instruction::SLO, TokenMode::AbsoluteIndexedY) => OpCode::SLO_aby,
        (Instruction::SLO, TokenMode::IndirectX) => OpCode::SLO_izx,
        (Instruction::SLO, TokenMode::IndirectY) => OpCode::SLO_izy,
        (Instruction::SLO, TokenMode::ZeroPageOrRelative) => OpCode::SLO_zp,
        (Instruction::SLO, TokenMode::ZeroPageX) => OpCode::SLO_zpx,
        (Instruction::SRE, TokenMode::Absolute) => OpCode::SRE_abs,
        (Instruction::SRE, TokenMode::AbsoluteIndexedX) => OpCode::SRE_abx,
        (Instruction::SRE, TokenMode::AbsoluteIndexedY) => OpCode::SRE_aby,
        (Instruction::SRE, TokenMode::IndirectX) => OpCode::SRE_izx,
        (Instruction::SRE, TokenMode::IndirectY) => OpCode::SRE_izy,
        (Instruction::SRE, TokenMode::ZeroPageOrRelative) => OpCode::SRE_zp,
        (Instruction::SRE, TokenMode::ZeroPageX) => OpCode::SRE_zpx,
        (Instruction::STA, TokenMode::Absolute) => OpCode::STA_abs,
        (Instruction::STA, TokenMode::AbsoluteIndexedX) => OpCode::STA_abx,
        (Instruction::STA, TokenMode::AbsoluteIndexedY) => OpCode::STA_aby,
        (Instruction::STA, TokenMode::IndirectX) => OpCode::STA_izx,
        (Instruction::STA, TokenMode::IndirectY) => OpCode::STA_izy,
        (Instruction::STA, TokenMode::ZeroPageOrRelative) => OpCode::STA_zp,
        (Instruction::STA, TokenMode::ZeroPageX) => OpCode::STA_zpx,
        (Instruction::STX, TokenMode::Absolute) => OpCode::STX_abs,
        (Instruction::STX, TokenMode::ZeroPageOrRelative) => OpCode::STX_zp,
        (Instruction::STX, TokenMode::ZeroPageY) => OpCode::STX_zpy,
        (Instruction::STY, TokenMode::Absolute) => OpCode::STY_abs,
        (Instruction::STY, TokenMode::ZeroPageOrRelative) => OpCode::STY_zp,
        (Instruction::STY, TokenMode::ZeroPageX) => OpCode::STY_zpx,
        (Instruction::TAS, TokenMode::AbsoluteIndexedY) => OpCode::TAS_aby,
        (Instruction::TAX, TokenMode::None) => OpCode::TAX,
        (Instruction::TAY, TokenMode::None) => OpCode::TAY,
        (Instruction::TSX, TokenMode::None) => OpCode::TSX,
        (Instruction::TXA, TokenMode::None) => OpCode::TXA,
        (Instruction::TXS, TokenMode::None) => OpCode::TXS,
        (Instruction::TYA, TokenMode::None) => OpCode::TYA,
        (Instruction::XAA, TokenMode::Immediate) => OpCode::XAA_imm,
        _ => {
            return Err(format!(
                "Unable to match the opcode {:?} {:?}",
                instruction, mode
            ))
        }
    })
}

pub const CYCLES_TABLE: [u8; 256] = [
    7, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, 2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7,
    4, 4, 7, 7, 6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, 2, 5, 0, 8, 4, 4, 6, 6,
    2, 4, 2, 7, 4, 4, 7, 7, 6, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6, 2, 5, 0, 8,
    4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, 6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, 2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2,
    4, 4, 4, 4, 2, 6, 0, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5, 2, 6, 2, 6, 3, 3, 3, 3,
    2, 2, 2, 2, 4, 4, 4, 4, 2, 5, 0, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4, 2, 6, 2, 8,
    3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7,
    4, 4, 7, 7,
];

// TODO
pub const EXTRA_CYCLES_TABLE: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];

pub const ADDRESSING_MODE_TABLE: [Mode; 256] = [
    Mode::None,
    Mode::IndirectX,
    Mode::None,
    Mode::IndirectX,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::None,
    Mode::Immediate,
    Mode::RegisterA,
    Mode::Immediate,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Relative,
    Mode::IndirectY,
    Mode::None,
    Mode::IndirectY,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::Absolute,
    Mode::IndirectX,
    Mode::None,
    Mode::IndirectX,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::None,
    Mode::Immediate,
    Mode::RegisterA,
    Mode::Immediate,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Relative,
    Mode::IndirectY,
    Mode::None,
    Mode::IndirectY,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::None,
    Mode::IndirectX,
    Mode::None,
    Mode::IndirectX,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::None,
    Mode::Immediate,
    Mode::RegisterA,
    Mode::Immediate,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Relative,
    Mode::IndirectY,
    Mode::None,
    Mode::IndirectY,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::None,
    Mode::IndirectX,
    Mode::None,
    Mode::IndirectX,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::None,
    Mode::Immediate,
    Mode::RegisterA,
    Mode::Immediate,
    Mode::Indirect,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Relative,
    Mode::IndirectY,
    Mode::None,
    Mode::IndirectY,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::Immediate,
    Mode::IndirectX,
    Mode::Immediate,
    Mode::IndirectX,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::None,
    Mode::Immediate,
    Mode::None,
    Mode::Immediate,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Relative,
    Mode::IndirectY,
    Mode::None,
    Mode::IndirectY,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageY,
    Mode::ZeroPageY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedY,
    Mode::Immediate,
    Mode::IndirectX,
    Mode::Immediate,
    Mode::IndirectX,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::None,
    Mode::Immediate,
    Mode::None,
    Mode::Immediate,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Relative,
    Mode::IndirectY,
    Mode::None,
    Mode::IndirectY,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageY,
    Mode::ZeroPageY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedY,
    Mode::Immediate,
    Mode::IndirectX,
    Mode::Immediate,
    Mode::IndirectX,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::None,
    Mode::Immediate,
    Mode::None,
    Mode::Immediate,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Relative,
    Mode::IndirectY,
    Mode::None,
    Mode::IndirectY,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::Immediate,
    Mode::IndirectX,
    Mode::Immediate,
    Mode::IndirectX,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::ZeroPage,
    Mode::None,
    Mode::Immediate,
    Mode::None,
    Mode::Immediate,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Absolute,
    Mode::Relative,
    Mode::IndirectY,
    Mode::None,
    Mode::IndirectY,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::ZeroPageX,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::None,
    Mode::AbsoluteIndexedY,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
    Mode::AbsoluteIndexedX,
];

pub const OPCODE_STRING_TABLE: [&str; 256] = [
    "brk", "ora", "kil", "slo", "nop", "ora", "asl", "slo", "php", "ora", "asl", "anc",
    "nop", "ora", "asl", "slo", "bpl", "ora", "kil", "slo", "nop", "ora", "asl", "slo",
    "clc", "ora", "nop", "slo", "nop", "ora", "asl", "slo", "jsr", "and", "kil", "rla",
    "bit", "and", "rol", "rla", "plp", "and", "rol", "anc", "bit", "and", "rol", "rla",
    "bmi", "and", "kil", "rla", "nop", "and", "rol", "rla", "sec", "and", "nop", "rla",
    "nop", "and", "rol", "rla", "rti", "eor", "kil", "sre", "nop", "eor", "lsr", "sre",
    "pha", "eor", "lsr", "alr", "jmp", "eor", "lsr", "sre", "bvc", "eor", "kil", "sre",
    "nop", "eor", "lsr", "sre", "cli", "eor", "nop", "sre", "nop", "eor", "lsr", "sre",
    "rts", "adc", "kil", "rra", "nop", "adc", "ror", "rra", "pla", "adc", "ror", "arr",
    "jmp", "adc", "ror", "rra", "bvs", "adc", "kil", "rra", "nop", "adc", "ror", "rra",
    "sei", "adc", "nop", "rra", "nop", "adc", "ror", "rra", "nop", "sta", "nop", "sax",
    "sty", "sta", "stx", "sax", "dey", "nop", "txa", "xaa", "sty", "sta", "stx", "sax",
    "bcc", "sta", "kil", "ahx", "sty", "sta", "stx", "sax", "tya", "sta", "txs", "tas",
    "shy", "sta", "shx", "ahx", "ldy", "lda", "ldx", "lax", "ldy", "lda", "ldx", "lax",
    "tay", "lda", "tax", "lax", "ldy", "lda", "ldx", "lax", "bcs", "lda", "kil", "lax",
    "ldy", "lda", "ldx", "lax", "clv", "lda", "tsx", "las", "ldy", "lda", "ldx", "lax",
    "cpy", "cmp", "nop", "dcp", "cpy", "cmp", "dec", "dcp", "iny", "cmp", "dex", "axs",
    "cpy", "cmp", "dec", "dcp", "bne", "cmp", "kil", "dcp", "nop", "cmp", "dec", "dcp",
    "cld", "cmp", "nop", "dcp", "nop", "cmp", "dec", "dcp", "cpx", "sbc", "nop", "isc",
    "cpx", "sbc", "inc", "isc", "inx", "sbc", "nop", "sbc", "cpx", "sbc", "inc", "isc",
    "beq", "sbc", "kil", "isc", "nop", "sbc", "inc", "isc", "sed", "sbc", "nop", "isc",
    "nop", "sbc", "inc", "isc",
];

type OperationFn = fn(&mut Cpu6502, Mode, u8);

pub const OPERATION_FN_TABLE: [OperationFn; 256] = [
    brk, ora, kil, slo, nop, ora, asl, slo, php, ora, asl, anc, nop, ora, asl, slo, bpl,
    ora, kil, slo, nop, ora, asl, slo, clc, ora, nop, slo, nop, ora, asl, slo, jsr, and,
    kil, rla, bit, and, rol, rla, plp, and, rol, anc, bit, and, rol, rla, bmi, and, kil,
    rla, nop, and, rol, rla, sec, and, nop, rla, nop, and, rol, rla, rti, eor, kil, sre,
    nop, eor, lsr, sre, pha, eor, lsr, alr, jmp, eor, lsr, sre, bvc, eor, kil, sre, nop,
    eor, lsr, sre, cli, eor, nop, sre, nop, eor, lsr, sre, rts, adc, kil, rra, nop, adc,
    ror, rra, pla, adc, ror, arr, jmp, adc, ror, rra, bvs, adc, kil, rra, nop, adc, ror,
    rra, sei, adc, nop, rra, nop, adc, ror, rra, nop, sta, nop, sax, sty, sta, stx, sax,
    dey, nop, txa, xaa, sty, sta, stx, sax, bcc, sta, kil, ahx, sty, sta, stx, sax, tya,
    sta, txs, tas, shy, sta, shx, ahx, ldy, lda, ldx, lax, ldy, lda, ldx, lax, tay, lda,
    tax, lax, ldy, lda, ldx, lax, bcs, lda, kil, lax, ldy, lda, ldx, lax, clv, lda, tsx,
    las, ldy, lda, ldx, lax, cpy, cmp, nop, dcp, cpy, cmp, dec, dcp, iny, cmp, dex, axs,
    cpy, cmp, dec, dcp, bne, cmp, kil, dcp, nop, cmp, dec, dcp, cld, cmp, nop, dcp, nop,
    cmp, dec, dcp, cpx, sbc, nop, isc, cpx, sbc, inc, isc, inx, sbc, nop, sbc, cpx, sbc,
    inc, isc, beq, sbc, kil, isc, nop, sbc, inc, isc, sed, sbc, nop, isc, nop, sbc, inc,
    isc,
];
