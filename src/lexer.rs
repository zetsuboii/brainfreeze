//! # Scanning (Lexing)
//!
//! ## Brainf*ck Syntax
//! - `>`  : Go to right cell
//! - `<`  : Go to left cell
//! - `+`  : Increment
//! - `-`  : Decrement
//! - `[]` : Loop
//! - `.`  : Put char
//! - `,`  : Read char
//!
//! When the code is parsed into tokens, it is called **regular language**
//! Tokens are also called **lexemes**.

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub struct Position(u32, u32);

impl Position {
    pub fn new(line_number: u32, offset: u32) -> Self {
        Self(line_number, offset)
    }

    pub fn line_number(&self) -> u32 {
        self.0
    }

    pub fn offset(&self) -> u32 {
        self.1
    }

    pub fn increment_line_number(&mut self) {
        self.0 += 1;
        self.1 = 0;
    }

    pub fn increment_offset(&mut self) {
        self.1 += 1;
    }
}

impl Display for Position {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!(
            "line {}, offset {}",
            self.line_number() + 1,
            self.offset() + 1
        ))
    }
}

pub type LexError = (Position, String);

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
}

impl Token {
    pub fn new(kind: TokenKind, position: Position) -> Self {
        Self { kind, position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
}

#[derive(Debug)]
pub enum TokenKind {
    Right(usize),
    Left(usize),
    Increment(usize),
    Decrement(usize),
    LoopStart,
    LoopEnd,
    PutChar,
    ReadChar,
    EOF,
}

impl Display for TokenKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!(
            "{:?}",
            match self {
                TokenKind::Right(_) => ">",
                TokenKind::Left(_) => "<",
                TokenKind::Increment(_) => "+",
                TokenKind::Decrement(_) => "-",
                TokenKind::LoopStart => "[",
                TokenKind::LoopEnd => "]",
                TokenKind::PutChar => ".",
                TokenKind::ReadChar => ",",
                TokenKind::EOF => "EOF",
            }
            .to_string()
        ))
    }
}

pub struct Lexer {
    inner: String,
}

impl Lexer {
    pub fn new(inner: String) -> Self {
        Self { inner }
    }

    pub fn scan_tokens(self) -> Result<Vec<Token>, Vec<LexError>> {
        use TokenKind::*;

        let chars = self.inner.chars();
        let mut stacked: Option<(char, usize)> = None;
        let mut tokens: Vec<Token> = Vec::new();
        let mut errors: Vec<LexError> = Vec::new();

        let mut position = Position::new(0, 0);

        for char in chars {
            position.increment_offset();

            if let Some((stack_char, stack_count)) = stacked {
                if char != stack_char {
                    // Push stacked token to tokens
                    match stack_char {
                        '+' => tokens.push(Token::new(Increment(stack_count), position)),
                        '-' => tokens.push(Token::new(Decrement(stack_count), position)),
                        '>' => tokens.push(Token::new(Right(stack_count), position)),
                        '<' => tokens.push(Token::new(Left(stack_count), position)),
                        _ => {}
                    }
                    stacked = None;
                }
            }

            match char {
                '+' | '-' | '<' | '>' => {
                    let default = Some((char, 1));
                    stacked = stacked.map_or(default, |(ch, cnt)| Some((ch, cnt + 1)));
                }
                '.' => tokens.push(Token::new(PutChar, position)),
                ',' => tokens.push(Token::new(ReadChar, position)),
                '[' => tokens.push(Token::new(LoopStart, position)),
                ']' => tokens.push(Token::new(LoopEnd, position)),
                '\n' => position.increment_line_number(),
                ' ' | '\t' => {}
                _ => errors.push((position, format!("Unrecognized character: {}", char))),
            }
        }

        // If there's anything stacked left, take it
        if let Some((stack_char, stack_count)) = stacked {
            match stack_char {
                '+' => tokens.push(Token::new(Increment(stack_count), position)),
                '-' => tokens.push(Token::new(Decrement(stack_count), position)),
                '>' => tokens.push(Token::new(Right(stack_count), position)),
                '<' => tokens.push(Token::new(Left(stack_count), position)),
                _ => {}
            }
        }

        tokens.push(Token::new(EOF, position));

        if errors.len() == 0 {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }
}
