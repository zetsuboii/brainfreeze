//! # Parsing
//! Regular language is converted into context free grammar (CFG)
//! It combines tokens into expressions and defines valid expressions
//! There are infinite number of valid strings (expressions) and finite
//! number of rules.
//!
//! You can create strings using these rules. These rules are called
//! **production** and strings created using these productions is called
//! **derivations**.
//!
//! Each production in CFG has a *head* and a *body*
//!
//! - **Head** is the rule's name
//!
//! - **Body** is what rule generates, which is a list of two type of symbols:
//!     - Terminal: Lexemes that don't lead to further lexems
//!     - Nonterminal: Lexemes that reference other rules. One rule name can
//!       refer to multiple bodies, in which case we can follow whichever one we want
//!
//! These rules are specified using Backus-Naur form (BNF).
//! - `->`      Start of the rule
//! - `*`       Zero or more times
//! - `+`       One or more times
//! - `|`       Union
//! - `?`       Optional
//! - `;`       End of rule
//!
//! BNF of Brainf*ck would be:
//! ```plaintext
//! program -> command*;
//! command -> operator | loop;
//! operator -> "+" | "-" | "<" | ">" | "," | ".";
//! loop -> "[" program "]";
//! ```
//!
//! This is a recursive structure so it will be represented with
//! [Abstract Syntax Tree (AST)](https://en.wikipedia.org/wiki/Abstract_syntax_tree)
//! structure.

use crate::{
    interpreter::{Interpret, State},
    lexer::{Position, Token, TokenKind},
};

#[derive(Debug)]
pub struct Program {
    pub commands: Vec<Box<dyn Command>>,
}
impl Interpret for Program {
    fn interpret(&mut self, state: &mut State) {
        for command in self.commands.iter_mut() {
            command.interpret(state);
        }
    }
}

pub type ParseError = (Position, String);

pub trait Command: std::fmt::Debug + Interpret {}

#[derive(Debug)]
pub enum Operator {
    Increment(usize),
    Decrement(usize),
    Right(usize),
    Left(usize),
    PutChar,
    ReadChar,
}
impl Interpret for Operator {
    fn interpret(&mut self, state: &mut State) {
        match self {
            Operator::Increment(v) => state.memory[state.pointer] += *v as u8,
            Operator::Decrement(v) => state.memory[state.pointer] -= *v as u8,
            Operator::Right(v) => state.pointer += *v,
            Operator::Left(v) => state.pointer -= *v,
            Operator::PutChar => {
                let c = state.memory[state.pointer] as char;
                state.output.push(c as u8);
            }
            Operator::ReadChar => {
                let c = state.input.pop().unwrap_or(0);
                state.memory[state.pointer] = c;
            }
        }
    }
}

impl Command for Operator {}

#[derive(Debug)]
pub struct Iteration {
    pub program: Program,
}
impl Interpret for Iteration {
    fn interpret(&mut self, state: &mut State) {
        while state.memory[state.pointer] != 0 {
            self.program.interpret(state);
        }
    }
}
impl Command for Iteration {}

#[derive(Debug)]
pub struct Parser {
    current: usize,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            current: 0,
            tokens,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn at_end(&self) -> bool {
        matches!(self.peek().kind(), &TokenKind::EOF)
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn push_error(&mut self, message: String) {
        self.errors.push((self.peek().position().clone(), message));
    }

    fn program(&mut self) -> Program {
        let mut commands: Vec<Box<dyn Command>> = Vec::new();

        loop {
            if let Some(operator) = self.operator() {
                commands.push(operator);
                continue;
            }

            if let Some(iteration) = self.iteration() {
                commands.push(iteration);
                continue;
            }

            break;
        }

        Program { commands }
    }

    fn operator(&mut self) -> Option<Box<dyn Command>> {
        let operator = match self.peek().kind() {
            TokenKind::Increment(v) => Some(Operator::Increment(*v)),
            TokenKind::Decrement(v) => Some(Operator::Decrement(*v)),
            TokenKind::Right(v) => Some(Operator::Right(*v)),
            TokenKind::Left(v) => Some(Operator::Left(*v)),
            TokenKind::PutChar => Some(Operator::PutChar),
            TokenKind::ReadChar => Some(Operator::ReadChar),
            _ => None,
        };

        if let Some(operator) = operator {
            self.advance();
            let boxed = Box::new(operator);
            Some(boxed)
        } else {
            None
        }
    }

    fn iteration(&mut self) -> Option<Box<dyn Command>> {
        if !matches!(self.peek().kind(), TokenKind::LoopStart) {
            return None;
        }
        self.advance();

        let program = self.program();

        if !matches!(self.peek().kind(), TokenKind::LoopEnd) {
            self.push_error("Expected ']'".to_string());
            return None;
        }
        self.advance();

        Some(Box::new(Iteration { program }))
    }

    pub fn parse(mut self) -> Result<Program, Vec<ParseError>> {
        let program = self.program();
        while !self.at_end() {
            self.push_error(format!("Unexpected token {}", self.peek().kind()));
            self.advance();
        }

        if self.errors.len() == 0 {
            Ok(program)
        } else {
            Err(self.errors)
        }
    }
}
