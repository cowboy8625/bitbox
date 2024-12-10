pub mod emitter;
pub mod module;
pub mod opcode;
pub mod section;

use anyhow::{bail, Result};
pub use emitter::Emitter;

trait ToDataType {
    fn to_data_type(&self) -> Result<section::DataType>;
}

impl ToDataType for crate::ssa::Type {
    fn to_data_type(&self) -> Result<section::DataType> {
        match self {
            crate::ssa::Type::Signed(32) => Ok(section::DataType::I32),
            crate::ssa::Type::Signed(64) => Ok(section::DataType::I64),
            crate::ssa::Type::Float(32) => Ok(section::DataType::F32),
            crate::ssa::Type::Float(64) => Ok(section::DataType::F64),
            unknown => bail!("Unknown type: {:?}", unknown),
        }
    }
}
