use crate::asm::Span;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BitBoxError {
    #[error("Invalid instruction {0:#02x}")]
    InvalidInstruction(u8),
    #[error("Invalid opcode {0:#02x}")]
    InvalidOpcode(u8),
    #[error("Register out of bounds at {}..{}", .1.col_start, .1.col_end)]
    RegisterOutOfBounds(u8, Span),
    #[error("Unknown label {}..{}", .1.col_start, .1.col_end)]
    UnknownLabel(String, Span),
}
