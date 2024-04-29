use thiserror::Error;

#[derive(Debug, Error)]
pub enum BitBoxError {
    #[error("Invalid instruction {0:#02x}")]
    InvalidInstruction(u8),
}
