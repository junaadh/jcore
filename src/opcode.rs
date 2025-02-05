use std::str::FromStr;

use crate::{error::Exception, register::Register, vm::OP_LEN};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Op {
    Nop = 0x6f,

    // arithmetic
    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,

    // store load
    Ldr = 0x30,
    Push = 0x33,
    Pop = 0x34,
}

impl TryFrom<u8> for Op {
    type Error = Exception;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use self::Op::*;
        let value = value & 0x7f;
        Ok(match value {
            0x6f => Nop,

            0x10 => Add,
            0x11 => Sub,
            0x12 => Mul,
            0x13 => Div,

            0x30 => Ldr,
            0x33 => Push,
            0x34 => Pop,
            _ => return Err(Exception::InvalidOp(value)),
        })
    }
}

impl FromStr for Op {
    type Err = Exception;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        Ok(match s.as_str() {
            "nop" => Self::Nop,
            "add" => Self::Add,
            "sub" => Self::Sub,
            "mul" => Self::Mul,
            "div" => Self::Div,
            "ldr" => Self::Ldr,
            "push" => Self::Push,
            "pop" => Self::Pop,
            _ => return Err(Exception::UnknownSymbol(s.into_boxed_str(), 0)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operand {
    Reg(Register),
    Imm(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Instruction {
    Nop,

    Add(Register, Register, Operand),
    Sub(Register, Register, Operand),
    Mul(Register, Register, Operand),
    Div(Register, Register, Operand),

    Ldr(Register, Operand),
    Push(Operand),
    Pop(Operand),
}

impl From<&Instruction> for Op {
    fn from(value: &Instruction) -> Self {
        use self::Instruction::*;
        match value {
            Nop => Op::Nop,

            Add(_, _, _) => Op::Add,
            Sub(_, _, _) => Op::Sub,
            Mul(_, _, _) => Op::Mul,
            Div(_, _, _) => Op::Div,

            Ldr(_, _) => Op::Ldr,
            Push(_) => Op::Push,
            Pop(_) => Op::Pop,
        }
    }
}

impl TryFrom<u32> for Instruction {
    type Error = Exception;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let imm_flag = (value >> 31) & 0x1 == 1;

        let opcode = Op::try_from(((value >> 24) & 0xff) as u8)?;

        use self::Op::*;
        Ok(match opcode {
            Nop => Self::Nop,

            Add | Sub | Mul | Div => {
                let r1 = Register::try_from(((value >> 19) & 0x1f) as u8)?;
                let r2 = Register::try_from(((value >> 14) & 0x1f) as u8)?;

                let r3 = if imm_flag {
                    Operand::Imm(value & 0x3fff)
                } else {
                    Operand::Reg(Register::try_from(((value >> 9) & 0x1f) as u8)?)
                };

                match opcode {
                    Add => Self::Add(r1, r2, r3),
                    Sub => Self::Sub(r1, r2, r3),
                    Mul => Self::Mul(r1, r2, r3),
                    Div => Self::Div(r1, r2, r3),
                    _ => unreachable!(),
                }
            }

            Ldr => {
                let r = Register::try_from(((value >> 19) & 0x1f) as u8)?;
                let o = if imm_flag {
                    Operand::Imm(value & 0x7ffff)
                } else {
                    Operand::Reg(Register::try_from(((value >> 14) & 0x1f) as u8)?)
                };
                Self::Ldr(r, o)
            }
            Push => Self::Push(if imm_flag {
                Operand::Imm(value & 0xffffff)
            } else {
                Operand::Reg(Register::try_from(((value >> 19) & 0x1f) as u8)?)
            }),
            Pop => Self::Pop(if imm_flag {
                Operand::Imm(value & 0xffffff)
            } else {
                Operand::Reg(Register::try_from(((value >> 19) & 0x1f) as u8)?)
            }),
        })
    }
}

impl TryFrom<Instruction> for u32 {
    type Error = Exception;

    fn try_from(value: Instruction) -> Result<Self, Self::Error> {
        let op = Op::from(&value);

        Ok(match value {
            Instruction::Nop => (op as u32) << 24,

            Instruction::Add(r1, r2, r3)
            | Instruction::Sub(r1, r2, r3)
            | Instruction::Mul(r1, r2, r3)
            | Instruction::Div(r1, r2, r3) => {
                let mut op_len = OP_LEN * 8;
                op_len -= 8;
                let mut encoded = (op as u32) << op_len;
                op_len -= 5;
                encoded |= (r1 as u32) << op_len;
                op_len -= 5;
                encoded |= (r2 as u32) << op_len;
                match r3 {
                    Operand::Reg(r) => {
                        op_len -= 5;
                        encoded |= (r as u32) << op_len;
                        encoded &= 0x7fff_ffff;
                    }
                    Operand::Imm(i) => {
                        encoded |= i;
                        encoded |= 0x8000_0000;
                    }
                }
                encoded
            }

            Instruction::Ldr(r, o) => {
                let mut op_len = OP_LEN * 8;
                op_len -= 8;
                let mut encoded = (op as u32) << op_len;
                op_len -= 5;
                encoded |= (r as u32) << op_len;
                match o {
                    Operand::Reg(re) => {
                        op_len -= 5;
                        encoded |= (re as u32) << op_len;
                        encoded |= 0x7fff_ffff;
                    }
                    Operand::Imm(i) => {
                        encoded |= i;
                        encoded |= 0x8000_0000;
                    }
                }
                encoded
            }
            Instruction::Push(o) | Instruction::Pop(o) => {
                let mut op_len = OP_LEN * 8;
                op_len -= 8;
                let mut encoded = (op as u32) << op_len;
                match o {
                    Operand::Reg(re) => {
                        op_len -= 5;
                        encoded |= (re as u32) << op_len;
                        encoded &= 0x7fff_ffff;
                    }
                    Operand::Imm(i) => {
                        encoded |= i;
                        encoded |= 0x8000_0000;
                    }
                }
                encoded
            }
        })
    }
}
