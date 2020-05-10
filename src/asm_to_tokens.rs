use crate::opcodes::{instruction_mode_to_op_code, match_instruction, Instruction, TokenMode};
use colored::*;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Instruction(Instruction),
    Mode(TokenMode),
    U8(u8),
    U16(u16),
}

#[derive(Debug, Clone, Copy)]
pub enum Character {
    Whitespace,
    Newline,
    Alpha,
    Numeric,
    NewLine,
    Value(char),
}

pub enum U8OrU16 {
    U8(u8),
    U16(u16),
}

fn char_to_enum(character: &char) -> Character {
    if character.is_numeric() {
        return Character::Numeric;
    }
    if character.is_alphabetic() {
        return Character::Alpha;
    }
    if character.is_whitespace() {
        return Character::Whitespace;
    }
    return Character::Value(character.clone());
}

// This is convenience sugar over peeking and matching an enum.
macro_rules! iter_peek_match {
    ($iterator:expr, $item:ident => $match_body:tt) => {
        loop {
            match $iterator.peek() {
                Some(&$item) => {
                    match char_to_enum(&$item) $match_body;
                },
                None => break,
            }
        };
    }
}

type TokenizerResult = Result<(), String>;

#[derive(Debug)]
struct ParseError {
    message: String,
    nice_message: String,
    column: u64,
    row: u64,
}

impl ParseError {
    fn new(message: String, parser: &AsmParser) -> ParseError {
        let error_row_index = parser.row as usize - 1;
        let range = 3;
        let min = (error_row_index as i64 - range).max(0) as usize;
        let max = (error_row_index as i64 + range) as usize;

        let mut nice_message = String::from("\n\n");
        for (row_index, row_text) in parser.text.lines().enumerate() {
            if row_index > max {
                break;
            }
            if row_index < min {
                continue;
            }

            // Lazypad.
            let col_string = if row_index < 9 {
                format!("   {}: ", row_index + 1)
            } else if row_index < 99 {
                format!("  {}: ", row_index + 1)
            } else if row_index < 999 {
                format!(" {}: ", row_index + 1)
            } else {
                format!("{}: ", row_index + 1)
            };
            nice_message.push_str(&format!("{}", &col_string.cyan()));

            nice_message.push_str(&format!("{}", &row_text.bright_white()));
            nice_message.push_str("\n");

            if row_index == error_row_index {
                let indent = " ".repeat((parser.column + 5) as usize);
                let error_message = &format!(
                    "^ parse error on row {} column {} ",
                    parser.row, parser.column
                );
                nice_message.push_str(&indent);
                nice_message.push_str(&format!("{}", error_message.bright_red()));
                nice_message.push_str("\n");
                nice_message.push_str(&indent);
                nice_message.push_str(&format!("{}", message.bright_red()));
                nice_message.push_str("\n");
            }
        }

        nice_message.push('\n');

        ParseError {
            message,
            nice_message,
            column: parser.column,
            row: parser.row,
        }
    }

    fn panic_nicely(self) {
        panic!(self.nice_message);
    }
}

pub struct AsmParser<'a> {
    text: &'a str,
    lines: std::str::Lines<'a>,
    characters: std::iter::Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    row: u64,
    column: u64,
}

impl<'a> AsmParser<'a> {
    fn new(text: &'a str) -> AsmParser {
        AsmParser {
            text,
            characters: IntoIterator::into_iter("".chars()).peekable(),
            lines: IntoIterator::into_iter(text.lines()),
            tokens: Vec::new(),
            column: 1,
            row: 1,
        }
    }

    fn next_character(&mut self) -> Option<char> {
        let character = self.characters.next();
        if character.is_some() {
            self.column += 1;
        }
        character
    }

    fn panic(&self, reason: &str) -> TokenizerResult {
        Err(format!("{} Location: {}:{}", reason, self.row, self.column))
    }

    fn parse(&mut self) -> Result<(), ParseError> {
        loop {
            match self.lines.next() {
                Some(line) => {
                    self.characters = IntoIterator::into_iter(line.chars()).peekable();
                    match self.parse_root_level() {
                        Err(message) => return Err(ParseError::new(message, self)),
                        _ => {}
                    }
                }
                None => {
                    return Ok(());
                }
            };
            self.row += 1;
            self.column = 0;
        }
    }

    fn parse_root_level(&mut self) -> Result<(), String> {
        loop {
            match self.next_character() {
                Some(character) => match char_to_enum(&character) {
                    Character::Whitespace => {}
                    Character::Value(';') => {
                        return self.ignore_comment_contents();
                    }
                    Character::Alpha => {
                        let word = self.get_word(Some(&character))?;
                        match match_instruction(&word) {
                            Some(instruction) => {
                                self.tokens.push(Token::Instruction(instruction.clone()));
                                self.parse_operand(instruction)?;
                            },
                            None => return Err(
                                format!("Found the word \"{}\", and not an instruction. Labels are not supported as of yet.", word)
                            ),
                        }
                    }
                    _ => return Err(format!("Unknown next token. {}", character)),
                },
                None => return Ok(()),
            }
        }
    }

    fn to_tokens(self) -> Vec<Token> {
        self.tokens
    }

    fn to_bytes(self) -> Result<Vec<u8>, String> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut tokens = self.tokens.iter().peekable();
        loop {
            match tokens.next() {
                Some(token) => match token {
                    Token::Instruction(instruction) => match tokens.peek() {
                        Some(Token::Mode(mode)) => {
                            bytes.push(instruction_mode_to_op_code(instruction, mode)? as u8);
                            tokens.next();

                            match mode {
                                TokenMode::Absolute
                                | TokenMode::AbsoluteIndexedX
                                | TokenMode::AbsoluteIndexedY
                                | TokenMode::Indirect => {
                                    match tokens.next() {
                                        Some(Token::U16(value)) => {
                                            let [a, b] = value.to_le_bytes();
                                            bytes.push(a);
                                            bytes.push(b);
                                        },
                                        Some(token) => return Err(
                                            format!("Expected a u16 to be the operand of an operation, but found a: {:?}", token)
                                        ),
                                        None => return Err(
                                            "Expected a u16 to be the operand of an operation, but found nothing".to_string()
                                        )
                                    };
                                }
                                TokenMode::ZeroPageOrRelative
                                | TokenMode::ZeroPageX
                                | TokenMode::ZeroPageY
                                | TokenMode::Immediate
                                | TokenMode::IndirectX
                                | TokenMode::IndirectY => {
                                    match tokens.next() {
                                        Some(Token::U8(value)) => bytes.push(*value),
                                        Some(token) => return Err(format!("Expected a u8 to be the operand of an operation, but found a: {:?}", token)),
                                        None => return Err("Expected a u8 to be the operand of an operation, but found nothing".to_string())
                                    };
                                }
                                TokenMode::Implied | TokenMode::None => {}
                            }
                        }
                        _ => {
                            bytes
                                .push(instruction_mode_to_op_code(instruction, &TokenMode::None)?
                                    as u8);
                        }
                    },
                    token => {
                        return Err(format!(
                            "Unexpected token. Expected an instrunction {:?}",
                            token
                        ))
                    }
                },
                None => break,
            }
        }
        Ok(bytes)
    }

    fn next_characters_u8(&mut self) -> Result<u8, String> {
        let dollar = self.next_character_or_err()?;
        self.verify_character(dollar, '$')?;

        let mut string = String::new();
        string.push(self.next_character_or_err()?);
        string.push(self.next_character_or_err()?);
        match u8::from_str_radix(&string, 16) {
            Err(err) => Err(format!("Unable to parse string as number {:?}", err)),
            Ok(value) => Ok(value),
        }
    }

    fn next_characters_u16(&mut self) -> Result<u16, String> {
        let dollar = self.next_character_or_err()?;
        self.verify_character(dollar, '$')?;

        let mut string = String::new();
        string.push(self.next_character_or_err()?);
        string.push(self.next_character_or_err()?);
        string.push(self.next_character_or_err()?);
        string.push(self.next_character_or_err()?);
        match u16::from_str_radix(&string, 16) {
            Err(err) => Err(format!("Unable to parse string as number {:?}", err)),
            Ok(value) => Ok(value),
        }
    }

    fn next_characters_u8_or_u16(&mut self) -> Result<U8OrU16, String> {
        let word = self.get_word(None)?;
        match word.len() {
            2 => match u8::from_str_radix(&word, 16) {
                Err(err) => Err(format!("Unable to parse string as number {:?}", err)),
                Ok(value) => Ok(U8OrU16::U8(value)),
            },
            4 => match u16::from_str_radix(&word, 16) {
                Err(err) => Err(format!("Unable to parse string as number {:?}", err)),
                Ok(value) => Ok(U8OrU16::U16(value)),
            },
            _ => Err("Unable to extract a number.".to_string()),
        }
    }

    fn next_character_or_err(&mut self) -> Result<char, String> {
        match self.next_character() {
            Some(character) => Ok(character),
            None => Err("Unexpected end of file.".to_string()),
        }
    }

    fn peek_is_next_character(&mut self, value: char) -> bool {
        match self.characters.peek() {
            Some(character) => *character == value,
            None => false,
        }
    }

    fn expect_next_character(&mut self, value: char) -> Result<(), String> {
        let next_char = self.next_character_or_err()?;
        if next_char == value {
            Ok(())
        } else {
            Err(format!(
                "Expected the character {} but found {}",
                value, next_char
            ))
        }
    }

    fn verify_character(&self, a: char, b: char) -> TokenizerResult {
        if a != b {
            return Err(format!("Expected character \"{}\" to be \"{}\"", a, b));
        }
        Ok(())
    }

    /// imm = #$00
    /// zp = $00
    /// zpx = $00,X
    /// zpy = $00,Y
    /// izx = ($00,X)
    /// izy = ($00),Y
    /// abs = $0000
    /// abx = $0000,X
    /// aby = $0000,Y
    /// ind = ($0000)
    /// rel = $0000 (PC-relative)
    fn parse_operand(&mut self, instruction: Instruction) -> Result<(), String> {
        loop {
            match self.next_character() {
                Some(character) => {
                    match char_to_enum(&character) {
                        Character::Whitespace => continue,
                        Character::Value(';') => {
                            return self.ignore_comment_contents();
                        }
                        Character::Value('#') => {
                            // Immediate mode, match #$00.
                            self.tokens.push(Token::Mode(TokenMode::Immediate));
                            let value = self.next_characters_u8()?;
                            self.tokens.push(Token::U8(value));
                            return self.continue_to_end_of_line();
                        }
                        Character::Value('$') => {
                            match self.next_characters_u8_or_u16()? {
                                U8OrU16::U8(value_u8) => {
                                    // Figure out the mode.
                                    if self.peek_is_next_character(',') {
                                        // Skip the ","
                                        self.next_character_or_err()?;
                                        let character = self.next_character_or_err()?;
                                        self.tokens.push(match character {
                                            'x' => Token::Mode(TokenMode::ZeroPageX),
                                            'y' => Token::Mode(TokenMode::ZeroPageY),
                                            _ => {
                                                return Err(format!(
                                                    "Unexpected index mode: {}",
                                                    character
                                                ))
                                            }
                                        });
                                    } else {
                                        self.tokens
                                            .push(Token::Mode(TokenMode::ZeroPageOrRelative));
                                    }

                                    self.tokens.push(Token::U8(value_u8));
                                }
                                U8OrU16::U16(value_u16) => {
                                    // Figure out the mode.
                                    if self.peek_is_next_character(',') {
                                        // Skip the ","
                                        self.next_character_or_err()?;
                                        let character = self.next_character_or_err()?;
                                        self.tokens.push(match character {
                                            'x' => Token::Mode(TokenMode::AbsoluteIndexedX),
                                            'y' => Token::Mode(TokenMode::AbsoluteIndexedY),
                                            _ => {
                                                return Err(format!(
                                                    "Unexpected index mode: {}",
                                                    character
                                                ))
                                            }
                                        });
                                    } else {
                                        self.tokens.push(Token::Mode(TokenMode::Absolute));
                                    }

                                    self.tokens.push(Token::U16(value_u16));
                                }
                            }
                            return self.continue_to_end_of_line();
                        }
                        Character::Value('(') => {
                            // jmp ($1234) ; indirect
                            // and ($aa,X) ; indirect indexed x
                            // and ($aa),Y ; indirect indexed y
                            self.expect_next_character('$')?;
                            match self.next_characters_u8_or_u16()? {
                                U8OrU16::U8(value_u8) => {
                                    // and ($aa,X) ; indirect indexed x
                                    // and ($aa),Y ; indirect indexed y
                                    let character = self.next_character_or_err()?;
                                    match char_to_enum(&character) {
                                        Character::Value(',') => {
                                            // and ($aa,X) ; indirect indexed x
                                            self.expect_next_character('X')?;
                                            self.expect_next_character(')')?;
                                            self.tokens.push(Token::Mode(TokenMode::IndirectX));
                                        }
                                        Character::Value(')') => {
                                            // and ($aa),Y ; indirect indexed y
                                            self.expect_next_character(',')?;
                                            self.expect_next_character('Y')?;
                                            self.tokens.push(Token::Mode(TokenMode::IndirectY));
                                        }
                                        _ => {
                                            return Err(format!(
                                                "Unexpected character {:?}",
                                                character
                                            ))
                                        }
                                    }
                                    self.tokens.push(Token::U8(value_u8));
                                }
                                U8OrU16::U16(value_u16) => {
                                    // jmp ($1234) ; indirect
                                    self.tokens.push(Token::Mode(TokenMode::Indirect));
                                    self.tokens.push(Token::U16(value_u16));
                                    self.expect_next_character(')')?;
                                }
                            }
                            return self.continue_to_end_of_line();
                        }
                        value => {
                            return Err(format!(
                                "Unknown character type when attempting to parse an operand. {:?}",
                                value
                            ))
                        }
                    }
                }
                None => {
                    return match instruction {
                        Instruction::DEX => Ok(()),
                        Instruction::DEY => Ok(()),
                        Instruction::INX => Ok(()),
                        Instruction::INY => Ok(()),
                        Instruction::BRK => Ok(()),
                        Instruction::RTI => Ok(()),
                        Instruction::RTS => Ok(()),
                        Instruction::CLC => Ok(()),
                        Instruction::SEC => Ok(()),
                        Instruction::CLD => Ok(()),
                        Instruction::SED => Ok(()),
                        Instruction::CLI => Ok(()),
                        Instruction::SEI => Ok(()),
                        Instruction::CLV => Ok(()),
                        Instruction::NOP => Ok(()),
                        Instruction::TAX => Ok(()),
                        Instruction::TXA => Ok(()),
                        Instruction::TAY => Ok(()),
                        Instruction::TYA => Ok(()),
                        Instruction::TSX => Ok(()),
                        Instruction::TXS => Ok(()),
                        Instruction::PLA => Ok(()),
                        Instruction::PHA => Ok(()),
                        Instruction::PLP => Ok(()),
                        Instruction::PHP => Ok(()),
                        _ => Err(format!("Instruction {:?} expected an operand", instruction)
                            .to_string()),
                    }
                }
            }
        }
    }

    fn ignore_comment_contents(&mut self) -> Result<(), String> {
        loop {
            // This effectively runs ".last()" without consuming the iterator.
            match self.next_character() {
                None => return Ok(()),
                _ => {}
            };
        }
    }

    fn continue_to_end_of_line(&mut self) -> TokenizerResult {
        loop {
            match self.next_character() {
                Some(character) => match char_to_enum(&character) {
                    Character::Whitespace => continue,
                    Character::Value(';') => return self.ignore_comment_contents(),
                    value => return Err(format!("Unknown character encountered \"{:?}\".", value)),
                },
                None => {
                    return Ok(());
                }
            }
        }
    }

    fn get_word(&mut self, starting_char: Option<&char>) -> Result<String, String> {
        let mut word = String::new();
        match starting_char {
            Some(starting_char) => {
                word.push(*starting_char);
            }
            None => {}
        }

        iter_peek_match!(self.characters, character => {
            Character::Alpha | Character::Numeric => {
                word.push(character);
                self.next_character();
            },
            value => {
                if word.len() == 0 {
                    return Err(format!("Expected to find an alpha-numeric value, but instead found {:?}", value));
                }
                break
            },
        });

        return Ok(word);
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::opcodes::OpCode::*;

    macro_rules! assert_program {
        ( $text:expr, [$( $bytes:expr ),*] ) => {
            let mut parser = AsmParser::new($text);

            match parser.parse() {
                Ok(_) => {
                    let bytes = parser.to_bytes().unwrap();
                    // Here's the biggest reason for the macro, this will add the `as u8`
                    // to the bytes generated.
                    assert_eq!(bytes, vec![$( $bytes as u8, )*]);
                }
                Err(parse_error) => {
                    parse_error.panic_nicely();
                }
            };
        };
    }

    #[test]
    fn test_immediate_mode() {
        assert_program!(
            "lda #$66    ; Load 0x66 into the A register",
            [LDA_imm, 0x66]
        );
    }

    #[test]
    fn test_multiple_lines() {
        assert_program!(
            "
                lda #$66    ; Load 0x66 into the A register
                adc #$55    ; Add 0x55 to it
            ",
            [LDA_imm, 0x66, ADC_imm, 0x55]
        );
    }

    #[test]
    fn test_ind_jmp() {
        assert_program!("jmp ($1234)", [JMP_ind, 0x34, 0x12]);
    }
    #[test]
    fn test_all_modes() {
        assert_program!(
            "
                lda #$66    ; immediate

                ora $1234   ; absolute
                asl $1234,x ; absolute indexed X
                eor $1234,y ; absolute indexed Y

                bpl $03     ; relative
                sty $04     ; zero page
                sta $05,x   ; zero page indexed X
                stx $06,y   ; zero page indexed Y

                jmp ($1234) ; indirect
                and ($aa,X) ; indirect indexed x
                and ($bb),Y ; indirect indexed y
            ",
            [
                LDA_imm, 0x66, ORA_abs, 0x34, 0x12, ASL_abx, 0x34, 0x12, EOR_aby, 0x34, 0x12,
                BPL_rel, 0x03, STY_zp, 0x4, STA_zpx, 0x05, STX_zpy, 0x06, JMP_ind, 0x34, 0x12,
                AND_izx, 0xaa, AND_izx, 0xbb
            ]
        );
    }
}
