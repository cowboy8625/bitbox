pub mod emitter;
pub mod module;
pub mod opcode;
pub mod section;

pub use emitter::Emitter;

trait ToDataType {
    fn to_data_type(&self) -> section::DataType;
}

impl ToDataType for crate::ast::Identifier {
    fn to_data_type(&self) -> section::DataType {
        match self.lexeme.as_str() {
            "i32" => section::DataType::I32,
            "i64" => section::DataType::I64,
            "f32" => section::DataType::F32,
            "f64" => section::DataType::F64,
            "void" => section::DataType::VOID,
            ty => unimplemented!("{ty}"),
        }
    }
}
