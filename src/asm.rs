use crate::opcodes::{instruction_mode_to_op_code, match_instruction, Instruction, TokenMode};
use colored::*;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Instruction(Instruction),
    Mode(TokenMode),
    U8(u8),
    U16(u16),
    Label(StringIndex),
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

pub type StringIndex = usize;
pub type ByteOffset = usize;

/// This struct is a string table that will hold a unique reference to a string.
/// This makers it easy to use simple numeric indexes to refer to a string rather
/// that duplicating a string and worrying about ownership. In addition there
/// is no duplication of strings.
///
/// In addition, it provides a mechanism for labeling the byte address of the label.
pub struct LabelTable {
    strings: Vec<String>,
    addresses: Option<Vec<ByteOffset>>,
    pub addresses_to_label: Vec<(StringIndex, ByteOffset)>,
}

impl LabelTable {
    pub fn new() -> LabelTable {
        LabelTable {
            strings: Vec::new(),
            addresses: None,
            addresses_to_label: Vec::new(),
        }
    }

    pub fn take_string(&mut self, string: String) -> StringIndex {
        match self.strings.iter().position(|s| *s == string) {
            Some(index) => index,
            None => {
                let index = self.strings.len();
                self.strings.push(string);
                index
            }
        }
    }

    pub fn index(&mut self, string: &String) -> StringIndex {
        match self.strings.iter().position(|s| s == string) {
            Some(index) => index,
            None => {
                let index = self.strings.len();
                self.strings.push(string.to_string());
                index
            }
        }
    }

    pub fn string(&self, index: StringIndex) -> Option<&String> {
        self.strings.get(index)
    }

    pub fn set_address(&mut self, address: usize, index: StringIndex) {
        match &self.addresses {
            Some(addresses) => {
                debug_assert_eq!(
                    addresses.len(),
                    self.strings.len(),
                    "Expected the StringTable to not changes size with computing addresses"
                );
            }
            None => {
                let addresses = vec![0; self.strings.len()];
                self.addresses = Some(addresses);
            }
        };
        match self.addresses {
            Some(ref mut addresses) => addresses[index] = address,
            None => panic!("self.addresses not found"),
        }
    }

    pub fn get_address(&self, index: StringIndex) -> Result<usize, String> {
        match self.addresses {
            Some(ref addresses) => {
                match addresses.get(index) {
                    Some(address) => Ok(*address),
                    None => Err(format!(
                        "Unable to find the address for the label {}",
                        self.strings.get(index).unwrap()
                    )),
                }
            },
            None => panic!("Attempted to look up the byte address of a string, but the addresses were not initialized")
        }
    }
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
pub struct ParseError {
    message: String,
    nice_message: String,
    column: u64,
    row: u64,
}

impl ParseError {
    fn new(message: String, parser: &AsmLexer) -> ParseError {
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

    pub fn panic_nicely(self) {
        panic!(self.nice_message);
    }
}

pub struct AsmLexer<'a> {
    text: &'a str,
    lines: std::str::Lines<'a>,
    characters: std::iter::Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    labels: LabelTable,
    row: u64,
    column: u64,
}

impl<'a> AsmLexer<'a> {
    pub fn new(text: &'a str) -> AsmLexer {
        AsmLexer {
            text,
            characters: IntoIterator::into_iter("".chars()).peekable(),
            lines: IntoIterator::into_iter(text.lines()),
            tokens: Vec::new(),
            labels: LabelTable::new(),
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

    /// For now the asm is simple enough to parse in one pass. Once I get to labels
    /// and other features I'll change it up to an AST.
    pub fn parse(&mut self) -> Result<(), ParseError> {
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
                            }
                            None => {
                                self.expect_next_character(':')?;
                                let label = Token::Label(self.labels.take_string(word));
                                self.tokens.push(label);
                            }
                        }
                    }
                    Character::Value('.') => match self.get_word(None)?.as_ref() {
                        "byte" => loop {
                            self.skip_whitespace();
                            let value = self.next_characters_u8()?;
                            self.tokens.push(Token::U8(value));
                            if !self.find_comma()? {
                                // No comma was found, and we skipped to the end of the line.
                                break;
                            }
                        },
                        "word" => loop {
                            self.skip_whitespace();
                            let value = self.next_characters_u16()?;
                            self.tokens.push(Token::U16(value));
                            if !self.find_comma()? {
                                // No comma was found, and we skipped to the end of the line.
                                break;
                            }
                        },
                        pragma => return Err(format!("Unknown pragma \".{}\"", pragma)),
                    },
                    _ => return Err(format!("Unknown next token. {}", character)),
                },
                None => return Ok(()),
            }
        }
    }

    pub fn to_tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub fn to_bytes(mut self) -> Result<Vec<u8>, String> {
        let mut bytes = self.to_bytes_before_labels()?;
        for (string_index, byte_offset) in self.labels.addresses_to_label.iter() {
            let label_value_u16 = self.labels.get_address(*string_index)? as u16;
            let [low, high] = label_value_u16.to_le_bytes();
            bytes[*byte_offset] = low;
            bytes[*byte_offset + 1] = high;
        }
        Ok(bytes)
    }

    fn to_bytes_before_labels(&mut self) -> Result<Vec<u8>, String> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut tokens = self.tokens.iter().peekable();
        loop {
            match tokens.next() {
                Some(token) => match token {
                    Token::Instruction(instruction) => match tokens.peek() {
                        Some(Token::Label(string_index)) => {
                            bytes.push(instruction_mode_to_op_code(
                                instruction,
                                &TokenMode::Absolute,
                            )? as u8);

                            // Go back and fill this label in.
                            self.labels
                                .addresses_to_label
                                .push((*string_index, bytes.len()));

                            // Push on a u16 address which will be filled in later.
                            bytes.push(0);
                            bytes.push(0);
                        }
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
                                            format!("Expected a u16 to be the operand of an operation, but found a: {:#x?}", token)
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
                                        Some(token) => return Err(format!("Expected a u8 to be the operand of an operation, but found a: {:#x?}", token)),
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
                    Token::Label(string_index) => {
                        self.labels.set_address(bytes.len(), *string_index);
                    }
                    Token::U8(value) => bytes.push(*value),
                    Token::U16(value) => {
                        let [a, b] = value.to_le_bytes();
                        bytes.push(a);
                        bytes.push(b);
                    }
                    token => {
                        return Err(format!("Unexpected token at the root level: {:#x?}", token))
                    }
                },
                None => break,
            }
        }
        Ok(bytes)
    }

    /// Attempts to find a comma after a number. Returns true on success, or false
    /// if the end of the line is reached
    fn find_comma(&mut self) -> Result<bool, String> {
        self.skip_whitespace();
        iter_peek_match!(self.characters, character => {
            Character::Value(',') => {
                // Skip past the comma and any whitespace.
                self.next_character();
                self.skip_whitespace();
                // A comma was found!
                return Ok(true)
            },
            Character::Value(';') => {
                self.continue_to_end_of_line()?;
            },
            value => return Err(format!("Unknown character when expecting a comma or semi-colon \"{:?}\"", value))
        });
        return Ok(false);
    }

    fn skip_whitespace(&mut self) {
        iter_peek_match!(self.characters, character => {
            Character::Whitespace => {
                self.next_character();
            },
            _ => return
        });
    }

    fn next_characters_u8(&mut self) -> Result<u8, String> {
        match self.next_character_or_err()? {
            '$' => {
                // e.g. $33
                let string = self.get_word(None)?;
                match u8::from_str_radix(&string, 16) {
                    Err(err) => Err(format!("Unable to parse hex string as number {:?}", err)),
                    Ok(value) => Ok(value),
                }
            }
            '%' => {
                // e.g. %11110000
                let string = self.get_word(None)?;
                match u8::from_str_radix(&string, 2) {
                    Err(err) => Err(format!("Unable to parse binary string as number {:?}", err)),
                    Ok(value) => Ok(value),
                }
            }
            character => {
                let number = self.get_word(Some(&character))?;
                match u8::from_str_radix(&number, 10) {
                    Ok(number) => Ok(number),
                    Err(_) => Err(format!("Unable to parse as integer \"{}\"", number)),
                }
            }
        }
    }

    fn next_characters_u16(&mut self) -> Result<u16, String> {
        match self.next_character_or_err()? {
            '$' => {
                // e.g. $3344
                let string = self.get_word(None)?;
                match u16::from_str_radix(&string, 16) {
                    Ok(value) => Ok(value),
                    Err(_) => Err(format!(
                        "Unable to parse hex string as integer \"${}\"",
                        string
                    )),
                }
            }
            '%' => {
                // e.g. %11110000111100000
                let string = self.get_word(None)?;
                match u16::from_str_radix(&string, 2) {
                    Err(err) => Err(format!(
                        "Unable to parse binary string as number \"%{:?}\"",
                        err
                    )),
                    Ok(value) => Ok(value),
                }
            }
            character => {
                let number = self.get_word(Some(&character))?;
                match u16::from_str_radix(&number, 10) {
                    Ok(number) => Ok(number),
                    Err(_) => Err(format!("Unable to parse as integer \"{}\"", number)),
                }
            }
        }
    }

    fn next_characters_u8_or_u16(&mut self) -> Result<U8OrU16, String> {
        match self.next_character_or_err()? {
            '$' => {
                // e.g. $33
                let word = self.get_word(None)?;
                match word.len() {
                    2 => match u8::from_str_radix(&word, 16) {
                        Err(err) => Err(format!("Unable to parse hex string as number {:?}", err)),
                        Ok(number) => Ok(U8OrU16::U8(number)),
                    },
                    4 => match u16::from_str_radix(&word, 16) {
                        Err(err) => Err(format!("Unable to parse hex string as number {:?}", err)),
                        Ok(number) => Ok(U8OrU16::U16(number)),
                    },
                    _ => Err("This hex number must be either 2 or 4 digits long.".to_string()),
                }
            }
            '%' => {
                // e.g. %11110000
                let word = self.get_word(None)?;
                match word.len() {
                    8 => match u8::from_str_radix(&word, 2) {
                        Err(err) => {
                            Err(format!("Unable to parse binary string as number {:?}", err))
                        }
                        Ok(number) => Ok(U8OrU16::U8(number)),
                    },
                    16 => match u16::from_str_radix(&word, 2) {
                        Err(err) => {
                            Err(format!("Unable to parse binary string as number {:?}", err))
                        }
                        Ok(number) => Ok(U8OrU16::U16(number)),
                    },
                    _ => Err("This binary number must be either 8 or 16 digits long.".to_string()),
                }
            }
            character => {
                // TODO - Is it possible to differentiate U8 or U16 here? For now assume
                // that it's u8.
                let number = self.get_word(Some(&character))?;
                match u8::from_str_radix(&number, 10) {
                    Ok(number) => Ok(U8OrU16::U8(number)),
                    Err(_) => Err(format!("Unable to parse as integer \"{}\"", number)),
                }
            }
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
        iter_peek_match!(self.characters, character => {
            Character::Whitespace => {
                self.next_character()
            },
            Character::Alpha => {
                let word = self.get_word(None)?;
                let label = Token::Label(self.labels.take_string(word));
                self.tokens.push(label);
                return self.continue_to_end_of_line();
            }
            Character::Value(';') => {
                // Check operand.
                self.verify_instruction_needs_no_operand(instruction)?;
                return self.continue_to_end_of_line();
            }
            Character::Value('#') => {
                // Immediate mode, match #$00.
                self.next_character();
                self.tokens.push(Token::Mode(TokenMode::Immediate));
                let value = self.next_characters_u8()?;
                self.tokens.push(Token::U8(value));
                return self.continue_to_end_of_line();
            }
            Character::Value('$')
            | Character::Value('%')
            | Character::Numeric => {
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
                self.next_character();
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
        });
        self.verify_instruction_needs_no_operand(instruction)
    }

    fn verify_instruction_needs_no_operand(&self, instruction: Instruction) -> Result<(), String> {
        match instruction {
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
            Instruction::KIL => Ok(()),
            _ => Err(format!("Instruction {:?} expected an operand", instruction).to_string()),
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

    /// Run this method when the line is expected to contain nothing except whitespace
    /// or a comment.
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
            let mut parser = AsmLexer::new($text);

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
    fn test_various_codes_individually() {
        assert_program!("jmp ($1234)", [JMP_ind, 0x34, 0x12]);
        assert_program!("lda #$22", [LDA_imm, 0x22]);
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

                kil;
            ",
            [
                LDA_imm, 0x66, ORA_abs, 0x34, 0x12, ASL_abx, 0x34, 0x12, EOR_aby, 0x34, 0x12,
                BPL_rel, 0x03, STY_zp, 0x4, STA_zpx, 0x05, STX_zpy, 0x06, JMP_ind, 0x34, 0x12,
                AND_izx, 0xaa, AND_izy, 0xbb, KIL
            ]
        );
    }

    #[test]
    fn test_all_modes_binary() {
        assert_program!(
            "
                lda #%11110000    ; immediate

                ora %1111000101010101   ; absolute
                asl %1111001001010101,x ; absolute indexed X
                eor %1111001101010101,y ; absolute indexed Y

                bpl %11110100     ; relative
                sty %11110101     ; zero page
                sta %11110110,x   ; zero page indexed X
                stx %11110111,y   ; zero page indexed Y

                jmp (%1111100001010101) ; indirect
                and (%11111001,X) ; indirect indexed x
                and (%11111010),Y ; indirect indexed y
            ",
            [
                LDA_imm, 0b11110000, ORA_abs, 0b01010101, 0b11110001, ASL_abx, 0b01010101,
                0b11110010, EOR_aby, 0b01010101, 0b11110011, BPL_rel, 0b11110100, STY_zp,
                0b11110101, STA_zpx, 0b11110110, STX_zpy, 0b11110111, JMP_ind, 0b01010101,
                0b11111000, AND_izx, 0b11111001, AND_izy, 0b11111010
            ]
        );
    }

    #[test]
    fn test_u8_digits() {
        assert_program!(
            "
                lda #123    ; immediate
                bpl 234     ; relative
            ",
            [LDA_imm, 123, BPL_rel, 234]
        );
    }

    #[test]
    fn test_labels() {
        assert_program!(
            "
                jmp mylabel
                lda #$11
                mylabel: ; This is address 0x0005
                lda #$22
            ",
            [JMP_abs, 0x05, 0x00, LDA_imm, 0x11, LDA_imm, 0x22]
        );
    }

    #[test]
    fn test_pragmas() {
        assert_program!(
            "
                             jmp mylabel
                            .byte $11
                            .byte $22, $33
                mylabel:    .word $5544      ; This is address 0x0006
            ",
            [JMP_abs, 0x06, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55]
        );
    }

    #[test]
    fn test_numbers() {
        assert_program!(
            "
                .byte 5
                .byte 255
                .byte %10101010
                .byte %10
                .word $ff
                .word %1111000011110000
            ",
            [
                0x05,
                0xff,
                0b1010_1010,
                0x2,
                0xff,
                0x00,
                0b11110000,
                0b11110000
            ]
        );
    }
}
