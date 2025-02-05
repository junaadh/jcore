use super::symbols::{SymbolId, SymbolTable};
use crate::{assembler::symbols::SymbolKind, opcode::Op, register::Register};
use std::str::Chars;
use TokensKind::*;

#[derive(Debug)]
pub struct Lexer<'src> {
    pub source: &'src str,
    chars: Chars<'src>,
    start: usize,
    syms: &'src mut SymbolTable<'src>,
    line: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str, syms: &'src mut SymbolTable<'src>) -> Self {
        Self {
            source,
            chars: source.chars(),
            start: 0,
            syms,
            line: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.advance_while(|c| c.is_whitespace());
        self.start = self.pos();

        let char = self.advance();

        let kind = match char {
            '#' => {
                self.start = self.pos();
                // consume number
                self.advance_while(|c| c.is_ascii_hexdigit() || c == 'x');
                let content = self.content().trim_start_matches("0x");
                match content.parse::<i8>() {
                    Ok(imm) => Imm(imm as i32),
                    Err(_) => match content.parse::<i16>() {
                        Ok(imm) => Imm(imm as i32),
                        Err(_) => match content.parse::<i32>() {
                            Ok(imm) => Imm(imm),
                            Err(_) => self.make_error(),
                        },
                    },
                }
            }
            x if x.is_ascii_alphabetic() || x == '_' => {
                self.advance_while(|x| x.is_ascii_alphanumeric() || x == '_');
                let content = self.content().to_lowercase();

                match content.parse::<Register>() {
                    Ok(r) => Register(r),
                    Err(_) => match content.parse::<Op>() {
                        Ok(o) => Mnemonic(o),
                        Err(_) => Label(self.syms.insert(self.content(), SymbolKind::Label, None)),
                    },
                }
            }
            '.' => {
                self.advance_while(|c| c.is_alphanumeric());
                Directive(
                    self.syms
                        .insert(self.content(), SymbolKind::Directive, None),
                )
            }
            '%' => {
                self.advance_while(|c| c.is_alphanumeric());
                Param(
                    self.syms
                        .insert(self.content(), SymbolKind::Parameter, None),
                )
            }

            ',' => Comma,
            ';' => {
                self.advance_while(|c| c != '\n');
                Comment
            }
            '\0' => Eof,

            _ => self.make_error(),
        };

        Token {
            kind,
            line: self.line,
        }
    }

    fn advance(&mut self) -> char {
        self.chars
            .next()
            .inspect(|&x| {
                if x == '\n' {
                    self.line += 1;
                }
            })
            .unwrap_or('\0')
    }

    fn peek(&self) -> char {
        self.chars.clone().next().unwrap_or('\0')
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn advance_while(&mut self, predicate: fn(char) -> bool) {
        while !self.is_eof() && predicate(self.peek()) {
            self.advance();
        }
    }

    fn pos(&self) -> usize {
        self.source.len() - self.chars.as_str().len()
    }

    fn content(&self) -> &'src str {
        &self.source[self.start..self.pos()]
    }

    fn make_error(&mut self) -> TokensKind {
        self.advance_while(|x| !x.is_whitespace() || x != ',');
        Error(self.syms.insert(self.content(), SymbolKind::None, None))
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == Eof {
            None
        } else {
            Some(token)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TokensKind {
    Mnemonic(Op),
    Register(Register),
    Imm(i32),
    Label(SymbolId),
    Directive(SymbolId),
    Error(SymbolId),
    Param(SymbolId),

    Comment,
    Comma,
    #[default]
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Token {
    pub kind: TokensKind,
    pub line: usize,
}

/*

   Directives:

   .entry _main
   .section data
   .macro name %a1 %b2
       Add %a1, %a1, %b2
   .endmacro

*/
