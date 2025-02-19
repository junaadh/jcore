use super::symbols::{SymbolId, SymbolTable};
use crate::{
    assembler::symbols::SymbolKind, opcode::Op, register::Register,
};
use std::{str::Chars, sync::MutexGuard};
use TokensKind::*;

#[derive(Debug)]
pub struct Lexer<'src> {
    pub source: &'src str,
    chars: Chars<'src>,
    start: usize,
    syms: MutexGuard<'src, SymbolTable<'static>>,
    line: usize,
}

pub fn tokenize<'src>(
    source: &'src str,
    syms: MutexGuard<'src, SymbolTable<'static>>,
) -> impl Iterator<Item = Token> + use<'src> {
    let mut lexer = Lexer::new(source, syms);
    std::iter::from_fn(move || {
        let token = lexer.next_token();
        if token.kind != TokensKind::Eof {
            Some(token)
        } else {
            None
        }
    })
}

impl<'src> Lexer<'src> {
    pub fn new(
        source: &'src str,
        syms: MutexGuard<'src, SymbolTable<'static>>,
    ) -> Self {
        Self {
            source,
            chars: source.chars(),
            start: 0,
            syms,
            line: 1,
        }
    }

    fn next_token(&mut self) -> Token {
        self.advance_while(|c| matches!(c, '\t' | '\r' | ' '));
        self.start = self.pos();

        let char = self.advance();
        // print!("'{char}' ");

        let kind = match char {
            '\n' => Newline,
            '#' => {
                self.start = self.pos();
                // consume number
                self.advance_while(|c| {
                    c.is_ascii_hexdigit() || c == 'x'
                });
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
                self.advance_while(|x| {
                    x.is_ascii_alphanumeric() || x == '_'
                });
                let content = self.content().to_lowercase();

                match content.parse::<Register>() {
                    Ok(r) => Register(r),
                    Err(_) => match content.parse::<Op>() {
                        Ok(o) => Mnemonic(o),
                        Err(_) => {
                            let s = self.content();
                            Label(self.syms.insert(
                                s,
                                SymbolKind::Label,
                                None,
                                self.line,
                            ))
                        }
                    },
                }
            }
            '.' => {
                self.advance_while(|c| c.is_alphanumeric());
                let s = self.content();
                Directive(self.syms.insert(
                    s,
                    SymbolKind::Directive,
                    None,
                    self.line,
                ))
            }
            '%' => {
                self.advance_while(|c| c.is_alphanumeric());
                Param(
                    self.content()
                        .trim_start_matches("%")
                        .parse::<usize>()
                        .unwrap()
                        .saturating_sub(1),
                )
            }

            ',' => Comma,
            ';' => {
                self.advance_while(|c| c != '\n');
                Comment
            }
            ':' => Semi,
            '\0' => Eof,

            _ => {
                // println!("'{char}' 0x{:02x}", char as u8);
                self.make_error()
            }
        };

        // println!(" {:?}", kind);
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
        let s = self.content();
        Error(self.syms.insert(s, SymbolKind::None, None, self.line))
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default,
)]
pub enum TokensKind {
    Mnemonic(Op),
    Register(Register),
    Imm(i32),
    Label(SymbolId),
    Directive(SymbolId),
    Error(SymbolId),
    Param(usize),

    Comment,
    Comma,
    Semi,
    Newline,
    #[default]
    Eof,
}

impl TokensKind {
    pub fn get_reg(
        &self,
    ) -> Result<Register, Box<dyn std::error::Error>> {
        if let Register(r) = self {
            Ok(*r)
        } else {
            Err("not a register symbol".to_string().into())
        }
    }

    pub fn get_op(&self) -> Result<Op, Box<dyn std::error::Error>> {
        if let Mnemonic(i) = self {
            Ok(*i)
        } else {
            Err("not a operand symbol".to_string().into())
        }
    }

    pub fn get_imm(&self) -> Result<i32, Box<dyn std::error::Error>> {
        if let Imm(i) = self {
            Ok(*i)
        } else {
            Err("not an immediate symbol".to_string().into())
        }
    }

    pub fn get_sym(
        &self,
    ) -> Result<SymbolId, Box<dyn std::error::Error>> {
        match *self {
            Label(s) | Error(s) | Directive(s) => Ok(s),
            _ => Err(format!("unexpected symbol {:?}", self,).into()),
        }
    }

    pub fn is_param(&self) -> bool {
        matches!(self, TokensKind::Param(_))
    }

    pub fn get_param(
        &self,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if let TokensKind::Param(i) = self {
            Ok(*i)
        } else {
            Err("failed to get the parameter".into())
        }
    }
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
