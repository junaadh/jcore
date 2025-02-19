mod directives;
pub mod lexer;
pub mod symbols;

use std::{
    borrow::BorrowMut,
    collections::HashMap,
    io::{stdout, Write},
    process::exit,
    sync::Mutex,
};

use directives::Directives;
use lexer::{tokenize, Token, TokensKind};
use symbols::{SymbolId, SymbolTable};

use crate::opcode::{Instruction, Op, Operand};

pub fn assemble(_filename: &str, source: &str) {
    let symbol_table = Mutex::new(SymbolTable::default());

    // tokenization
    let binding: std::sync::MutexGuard<'_, SymbolTable<'_>> =
        symbol_table.lock().unwrap();
    let raw_tokens = tokenize(source, binding).collect::<Vec<_>>();

    // first pass: resolve macro defs, labels
    let mut binding = symbol_table.lock().unwrap();
    let mut directives = HashMap::new();
    let tokens = first_pass(
        raw_tokens.into_iter(),
        binding.borrow_mut(),
        &mut directives,
    )
    .map_err(|x| {
        eprintln!("error occured: ");
        for e in x {
            let c = if let TokensKind::Error(i) = e.kind {
                binding.get_symbol(&i).unwrap().name.to_string()
            } else {
                " ".to_string()
            };
            eprintln!(
                "unexpected symbol: '{c}' found at line: {}",
                e.line
            );
        }
        exit(1);
    })
    .unwrap();

    // second pass
    let ins =
        second_pass(tokens.into_iter(), &directives, &mut binding)
            .unwrap();

    // println!("entry: {}", ins.entry);
    // for i in ins.instructions {
    // println!("{:?}", i);
    // }
    let encoded = ins
        .instructions
        .iter()
        .map(|x| u32::try_from(*x).unwrap().to_le_bytes())
        .collect::<Vec<_>>()
        .concat();
    let mut stdout = stdout().lock();
    stdout.write_all(&encoded).unwrap();
}

fn second_pass(
    tokens: impl Iterator<Item = Token>,
    directives: &HashMap<SymbolId, Vec<Macros>>,
    symbol_table: &mut SymbolTable,
) -> Result<ResolvedTokens, Box<dyn std::error::Error>> {
    let mut tokens = tokens.peekable();
    let entry = if let DirectiveBody::Generic { body } = &directives
        .get(&symbol_table.get_id(".entry").unwrap())
        .unwrap()[0]
        .body
    {
        if let TokensKind::Label(i) = body[0].kind {
            symbol_table
                .get_symbol(&i)
                .unwrap()
                .value
                .unwrap_or_default()
        } else {
            0
        }
    } else {
        0
    } as i32;
    let mut ins_vec = Vec::new();

    let mut index = 0 - entry;

    while tokens.peek().is_some() {
        let cur = tokens.next().unwrap();

        use TokensKind::*;
        match cur.kind {
            Mnemonic(i) => {
                index += 1;
                let ins = match i {
                    Op::Nop => Instruction::Nop,

                    Op::Add | Op::Sub | Op::Mul | Op::Div => {
                        let o1 =
                            tokens.next().unwrap().kind.get_reg()?;

                        assert_eq!(
                            tokens.next().map(|t| t.kind),
                            Some(TokensKind::Comma),
                            "expected a fucking comma",
                        );

                        let o2 =
                            tokens.next().unwrap().kind.get_reg()?;

                        assert_eq!(
                            tokens.next().map(|t| t.kind),
                            Some(TokensKind::Comma),
                            "expected a fucking comma",
                        );

                        let o3 = {
                            let tok = tokens.next().unwrap();
                            match tok.kind.get_reg() {
                                Ok(r) => Operand::Reg(r),
                                Err(_) => match tok.kind.get_imm() {
                                    Ok(i) => Operand::Imm(i as u32),
                                    Err(i) => return Err(i),
                                },
                            }
                        };
                        match i {
                            Op::Add => Instruction::Add(o1, o2, o3),
                            Op::Sub => Instruction::Sub(o1, o2, o3),
                            Op::Mul => Instruction::Mul(o1, o2, o3),
                            Op::Div => Instruction::Div(o1, o2, o3),
                            _ => unreachable!(),
                        }
                    }

                    Op::Ldr => {
                        let o1 =
                            tokens.next().unwrap().kind.get_reg()?;

                        assert_eq!(
                            tokens.next().map(|t| t.kind),
                            Some(TokensKind::Comma),
                            "expected a fucking comma",
                        );

                        let o2 = {
                            let tok = tokens.next().unwrap();
                            match tok.kind.get_reg() {
                                Ok(r) => Operand::Reg(r),
                                Err(_) => match tok.kind.get_imm() {
                                    Ok(i) => Operand::Imm(i as u32),
                                    Err(i) => return Err(i),
                                },
                            }
                        };
                        Instruction::Ldr(o1, o2)
                    }

                    Op::Push | Op::Pop => {
                        let o = {
                            let tok = tokens.next().unwrap();
                            match tok.kind.get_reg() {
                                Ok(r) => Operand::Reg(r),
                                Err(_) => match tok.kind.get_imm() {
                                    Ok(i) => Operand::Imm(i as u32),
                                    Err(i) => return Err(i),
                                },
                            }
                        };
                        match i {
                            Op::Push => Instruction::Push(o),
                            Op::Pop => Instruction::Pop(o),
                            _ => unreachable!(),
                        }
                    }
                };
                ins_vec.push(ins);
            }
            Label(i) => {
                let is_decl = tokens
                    .peek()
                    .is_some_and(|t| t.kind == TokensKind::Semi);
                if is_decl {
                    symbol_table.update(i, |s| {
                        s.value = Some(index as u32);
                    });

                    // consume semi
                    tokens.next();
                } else {
                    let macro_decl = directives
                        .get(&symbol_table.get_id(".macro").unwrap())
                        .unwrap()
                        .iter()
                        .find(|x| match x.body {
                            DirectiveBody::Macro { name, .. } => {
                                name.kind.get_sym().unwrap() == i
                            }
                            _ => false,
                        })
                        .unwrap();

                    if let DirectiveBody::Macro {
                        parameters,
                        body,
                        ..
                    } = &macro_decl.body
                    {
                        let mut callees = vec![];
                        (0..parameters.len()).for_each(|_| {
                            if tokens.peek().is_some() {
                                callees.push(tokens.next().unwrap());
                            }
                        });
                        let mut part_resolved = Vec::new();

                        for tok in body {
                            if tok.kind.is_param() {
                                let idx = tok.kind.get_param()?;
                                part_resolved.push(callees[idx]);
                            } else {
                                part_resolved.push(*tok);
                            }
                        }

                        let inner_ins = second_pass(
                            part_resolved.into_iter(),
                            directives,
                            symbol_table,
                        )?;
                        ins_vec.extend_from_slice(
                            &inner_ins.instructions,
                        );
                    }
                }
            }
            Directive(_i) => {
                // let macro_ = &directives.get(&i).unwrap()[0];
                // let sym = symbol_table.get_symbol(&i).unwrap();
                // let kind = sym.name.parse::<Directives>()?;

                todo!()
            }
            Newline | Comment | Semi => continue,
            x => todo!("{x:?}"),
        }
    }

    Ok(ResolvedTokens {
        entry,
        instructions: ins_vec,
    })
}

fn first_pass(
    tokens: impl Iterator<Item = Token>,
    symbol_table: &mut SymbolTable,
    directives: &mut HashMap<SymbolId, Vec<Macros>>,
) -> Result<Vec<Token>, Vec<Token>> {
    let mut tokens = tokens.peekable();

    let mut resolved_tokens = Vec::<Token>::new();
    let mut errors = Vec::<Token>::new();

    // index from start of file
    let mut index = 0;

    while tokens.peek().is_some() {
        let cur = tokens.next().unwrap();

        match cur.kind {
            TokensKind::Mnemonic(_) => {
                index += 4;
                resolved_tokens.push(cur);
            }
            TokensKind::Label(i) => {
                symbol_table.update(i, |s| s.value = Some(index));
                resolved_tokens.push(cur);
            }
            TokensKind::Directive(e) => {
                let macro_ = symbol_table
                    .get_symbol(&e)
                    .map(|s| {
                        s.name
                            .parse::<directives::Directives>()
                            .expect("assemble error")
                    })
                    .unwrap();
                match macro_ {
                    Directives::Entry | Directives::Section => {
                        let mut body = Vec::new();
                        while tokens.peek().is_some_and(|t| {
                            t.kind != TokensKind::Newline
                        }) {
                            body.push(tokens.next().unwrap());
                        }
                        body.push(tokens.next().unwrap());
                        let dot_macro = Macros {
                            name: e,
                            body: DirectiveBody::Generic { body },
                        };
                        if let std::collections::hash_map::Entry::Vacant(e) = directives.entry(e) {
                            e.insert(vec![dot_macro]);
                        } else {
                            directives
                                .get_mut(&e)
                                .unwrap()
                                .push(dot_macro);
                        }
                    }
                    Directives::MacroStart => {
                        let name = tokens.next().unwrap();
                        let mut params = Vec::new();
                        let mut body = Vec::new();
                        while tokens.peek().is_some_and(|t| {
                            t.kind != TokensKind::Newline
                        }) {
                            params.push(tokens.next().unwrap());
                        }
                        // skip newline
                        assert_eq!(
                            tokens.next().map(|t| t.kind),
                            Some(TokensKind::Newline)
                        );
                        while tokens.peek().is_some_and(|t| {
                            if let TokensKind::Directive(i) = t.kind {
                                symbol_table.get_symbol(&i).map(|x| {
                                    x.name.parse::<Directives>().map(
                                        |x| x != Directives::MacroEnd,
                                    ).unwrap()
                                }).unwrap()
                            } else {
                                true
                            }
                        }) {
                            body.push(tokens.next().unwrap());
                        }
                        tokens.next();
                        while tokens.peek().is_some_and(|x| {
                            x.kind == TokensKind::Newline
                        }) {
                            tokens.next();
                        }
                        let dot_macro = Macros {
                            name: e,
                            body: DirectiveBody::Macro {
                                name,
                                parameters: params,
                                body,
                            },
                        };
                        if let std::collections::hash_map::Entry::Vacant(e) = directives.entry(e) {
                            e.insert(vec![dot_macro]);
                        } else {
                            directives
                                .get_mut(&e)
                                .unwrap()
                                .push(dot_macro);
                        }
                    }
                    _ => unreachable!(),
                }
            }
            TokensKind::Error(_) => {
                index += 4;
                errors.push(cur);
            }
            _ => resolved_tokens.push(cur),
        }
    }

    if errors.is_empty() {
        Ok(resolved_tokens)
    } else {
        Err(errors)
    }
}

#[derive(Debug)]
pub struct Macros {
    pub name: SymbolId,
    pub body: DirectiveBody,
}

#[derive(Debug)]
pub enum DirectiveBody {
    Macro {
        name: Token,
        parameters: Vec<Token>,
        body: Vec<Token>,
    },
    Generic {
        body: Vec<Token>,
    },
}

#[derive(Debug, Clone)]
pub struct ResolvedTokens {
    pub entry: i32,
    pub instructions: Vec<Instruction>,
}
