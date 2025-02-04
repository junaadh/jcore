use crate::{
    error::Exception,
    memory::{Addressable, MEMORY_LEN},
    opcode::Instruction,
    register::*,
};
use std::ops::{Index, IndexMut};

pub type BIT = u32;
pub const OP_LEN: BIT = std::mem::size_of::<BIT>() as BIT;

#[derive(Debug)]
pub struct Machine {
    register: [BIT; REGISTER_LEN],
    // stack: Stack<BIT, STACK_LEN>,
    pub mem: Box<dyn Addressable>,
}

impl Default for Machine {
    fn default() -> Self {
        Machine::new()
    }
}

impl Machine {
    pub fn new() -> Self {
        let mut vm = Self {
            register: [0; REGISTER_LEN],
            // stack: Stack::new(),
            mem: Box::new([0; MEMORY_LEN]),
        };

        // FIXME: setting the stack pointer
        vm[SP] = 0x400;
        vm
    }

    pub fn state(&self) {
        println!(
            "R0: {}\nR1: {}\nR2: {}\nR3: {}\nSP: {}\nPC: {}\nBP: {}\nFL: {}",
            self[R0], self[R1], self[R2], self[R3], self[SP], self[PC], self[BP], self[FLAGS],
        )
    }

    pub fn step(&mut self) -> Result<(), Exception> {
        let pc = self[PC];
        // print!("DEBUG: pc={} ", pc);

        let instruction = self.mem.read_u32(pc)?;
        self[PC] += OP_LEN;
        let op = Instruction::try_from(instruction)?;
        // println!("instruction: {op:?}");
        match op {
            Instruction::Nop => Ok(()),

            Instruction::Add(r1, r2, r3) => {
                self[r1] = self[r2]
                    + match r3 {
                        crate::opcode::Operand::Reg(r) => self[r],
                        crate::opcode::Operand::Imm(i) => i,
                    };
                Ok(())
            }
            Instruction::Sub(r1, r2, r3) => {
                self[r1] = self[r2]
                    - match r3 {
                        crate::opcode::Operand::Reg(r) => self[r],
                        crate::opcode::Operand::Imm(i) => i,
                    };
                Ok(())
            }
            Instruction::Mul(r1, r2, r3) => {
                self[r1] = self[r2]
                    * match r3 {
                        crate::opcode::Operand::Reg(r) => self[r],
                        crate::opcode::Operand::Imm(i) => i,
                    };
                Ok(())
            }
            Instruction::Div(r1, r2, r3) => {
                let div = match r3 {
                    crate::opcode::Operand::Reg(r) => self[r],
                    crate::opcode::Operand::Imm(i) => i,
                };

                if div == 0 {
                    return Err(Exception::DivisionByZero);
                }

                self[r1] = self[r2] / div;
                Ok(())
            }
            Instruction::Ldr(r, o) => {
                self[r] = match o {
                    crate::opcode::Operand::Reg(r) => self.mem.read_u32(self[r])?,
                    crate::opcode::Operand::Imm(i) => i,
                };
                Ok(())
            }
            Instruction::Push(o) => {
                self[SP] -= OP_LEN;
                self.mem.write_u32(
                    self[SP],
                    match o {
                        crate::opcode::Operand::Reg(r) => self[r],
                        crate::opcode::Operand::Imm(i) => i,
                    },
                )?;
                // println!("DEBUG: pushed to {}", self[SP]);
                Ok(())
            }
            Instruction::Pop(o) => {
                let r = match o {
                    crate::opcode::Operand::Reg(r) => r,
                    crate::opcode::Operand::Imm(_) => unreachable!(),
                };
                // println!("DEBUG: popping from {}", self[SP]);
                let value = self.mem.read_u32(self[SP])?;
                self[SP] += OP_LEN;
                self[r] = value;
                Ok(())
            }
        }
    }
}

impl Index<Register> for Machine {
    type Output = BIT;

    #[inline]
    fn index(&self, index: Register) -> &Self::Output {
        &self.register[index as usize]
    }
}

impl IndexMut<Register> for Machine {
    #[inline]
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.register[index as usize]
    }
}
