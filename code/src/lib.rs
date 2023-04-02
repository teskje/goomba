mod decode;
mod display;
mod inst;

use std::fmt;

pub use decode::decode;
pub use inst::{Cnd, Dst, DstW, Inst, Ref, Reg, RegW, Src, SrcW};

#[derive(Debug)]
pub enum Error {
    InvalidOpcode(u8),
    TooFewBytes,
    TooManyBytes,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            InvalidOpcode(op) => write!(f, "invalid opcode: {op:#x}"),
            TooFewBytes => f.write_str("too few bytes"),
            TooManyBytes => f.write_str("too many bytes"),
        }
    }
}

impl std::error::Error for Error {}
