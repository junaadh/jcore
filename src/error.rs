#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exception {
    InvalidMemoryAccess(u32),
    StackOverflow,
    StackUnderflow,
    InvalidOp(u8),
    InvalidReg(u8),
    DivisionByZero,
}
