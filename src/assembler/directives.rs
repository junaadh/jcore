use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Directives {
    Entry,
    Section,
    MacroStart,
    MacroEnd,
}

impl FromStr for Directives {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            ".entry" => Self::Entry,
            ".section" => Self::Section,
            ".macro" => Self::MacroStart,
            ".endmacro" => Self::MacroEnd,
            _ => return Err(format!("unknown macro {s}").into()),
        })
    }
}
