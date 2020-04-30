use super::opcodes::OpCode;

#[derive(Debug, Clone)]
pub enum Token {
    Op(OpCode),
    Return,
    If,
    Else,
    Identifier(StringIndex),
    Number(f64),
    Char(char),
}

#[derive(Debug, Clone, Copy)]
pub enum Character {
    Whitespace,
    Alpha,
    Numeric,
    NewLine,
    Value(char),
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


pub type StringIndex = usize;

pub struct StringTable {
    strings: Vec<String>
}

impl StringTable {
    pub fn new() -> StringTable {
        StringTable {
            strings: Vec::new()
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
}

pub struct AsmToTokens<'a> {
    characters: std::iter::Peekable<
        std::str::Chars<'a>
    >,
    tokens: Vec<Token>,
    string_table: StringTable,
}

impl<'a> AsmToTokens<'a> {
    fn new (text: &'a str) -> AsmToTokens {
        AsmToTokens {
            characters: IntoIterator::into_iter(text.chars()).peekable(),
            binary: Vec::new(),
            string_table: StringTable::new()
        }
    }

    fn parse(&mut self) {

    }

    fn parse_instruction(&mut self) {
        let op_ident: Vec<char> = Vec::with_capacity(3);
        let mode_ident: Vec<char> = Vec::with_capacity(3);

        loop {
            match self.characters.next() {
                Some(character) => {
                    match char_to_enum(&character) {
                        Character::Whitespace => continue,
                        Character::Value('\n') => {},
                        Character::Value(';') => self.skip_to_end_of_line_if_comment(),
                        Character::Numeric => tokens.push(
                            Token::Number(get_number(&mut characters, character))
                        ),
                        Character::Alpha => tokens.push({
                            let word = get_word(&mut characters, &character);
                            if word == "function" {
                                Token::Function
                            } else if word == "if" {
                                Token::If
                            } else if word == "else" {
                                Token::Else
                            } else if word == "return" {
                                Token::Return
                            } else {
                                Token::Identifier(string_table.take_string(word))
                            }
                        }),
                        Character::Value(value) => tokens.push(Token::Char(value))
                    }
                },
                None => break,
            }
        }
    }

    fn skip_to_end_of_line_if_comment(&mut self) {
        loop {
            match self.characters.next() {
                Some('\n') => break,
                Some(_) => continue,
                None => break,
            }
        }
    }

    fn get_word(&mut self, starting_char: &char) -> String {
        let mut word = String::new();
        word.push(*starting_char);

        iter_peek_match!(self.characters, character => {
            Character::Alpha | Character::Numeric => {
                word.push(character);
                self.characters.next();
            },
            _ => break,
        });

        return word;
    }

}
