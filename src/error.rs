#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exception {
    InvalidMemoryAccess(u32),
    StackOverflow,
    StackUnderflow,
    InvalidOp(u8),
    InvalidReg(u8),
    DivisionByZero,

    UnknownSymbol(Box<str>, usize),
}

impl Exception {
    pub fn add_line(&mut self, line: usize) {
        if let Exception::UnknownSymbol(_, i) = self {
            *i = line;
        }
    }
}
