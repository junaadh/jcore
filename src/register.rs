#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    SP,
    PC,
    BP,
    // DONT create any preceding variants as this is used for max register len
    FLAGS,
}

use std::str::FromStr;

pub use Register::*;

use crate::error::Exception;
pub const REGISTER_LEN: usize = Register::FLAGS as usize + 1;

impl Register {}

impl TryFrom<u8> for Register {
    type Error = Exception;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => R0,
            1 => R1,
            2 => R2,
            3 => R3,
            4 => SP,
            5 => PC,
            6 => BP,
            7 => FLAGS,
            _ => return Err(Exception::InvalidReg(value)),
        })
    }
}

impl FromStr for Register {
    type Err = Exception;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "r0" | "R0" => Self::R0,
            "r1" | "R1" => Self::R1,
            "r2" | "R2" => Self::R2,
            "r3" | "R3" => Self::R3,
            "sp" | "SP" => Self::SP,
            "pc" | "PC" => Self::PC,
            "bp" | "BP" => Self::BP,
            "flags" | "FLAGS" => Self::FLAGS,
            _ => return Err(Exception::UnknownSymbol(s.to_string().into_boxed_str(), 0)),
        })
    }
}
