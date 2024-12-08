pub mod emitter;
pub mod module;
pub mod opcode;
pub mod section;

pub use emitter::Emitter;

trait ToDataType {
    fn to_data_type(&self) -> section::DataType;
}

impl ToDataType for crate::ssa::Type {
    fn to_data_type(&self) -> section::DataType {
        match self {
            crate::ssa::Type::Signed(32) => section::DataType::I32,
            crate::ssa::Type::Signed(64) => section::DataType::I64,
            crate::ssa::Type::Float(32) => section::DataType::F32,
            crate::ssa::Type::Float(64) => section::DataType::F64,
            crate::ssa::Type::Void => section::DataType::VOID,
            unknown => unimplemented!("Unknown type: {:?}", unknown),
        }
    }
}
