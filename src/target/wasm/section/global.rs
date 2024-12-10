use super::DataType;
use super::Instruction;
use anyhow::Result;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Global {
    globals: Vec<GlobalEntry>,
}

impl Global {
    pub const ID: u8 = 0x06;
    pub fn new(globals: Vec<GlobalEntry>) -> Self {
        Self { globals }
    }

    pub fn push(&mut self, global: GlobalEntry) {
        self.globals.push(global);
    }

    pub fn len(&self) -> usize {
        // one for vec length
        let mut length = 1;
        for global in &self.globals {
            length += global.to_bytes().unwrap().len();
        }
        length
    }

    pub fn get(&self, name: &str) -> Option<(usize, &GlobalEntry)> {
        self.globals
            .iter()
            .enumerate()
            .find(|(_, global)| global.name == name)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.push(Global::ID);
        // Add 1 for the count;
        let length = self.len();
        leb128::write::unsigned(&mut bytes, length as u64)?;

        leb128::write::unsigned(&mut bytes, self.globals.len() as u64)?;
        for entry in &self.globals {
            bytes.extend(entry.to_bytes()?);
        }
        Ok(bytes)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Intializer {
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    Get(u32),
}

impl Intializer {
    pub fn as_instruction(&self) -> Instruction {
        match self {
            Self::I32Const(val) => Instruction::I32Const(*val),
            Self::I64Const(val) => Instruction::I64Const(*val),
            Self::F32Const(val) => Instruction::F32Const(*val),
            Self::F64Const(val) => Instruction::F64Const(*val),
            Self::Get(val) => Instruction::GlobalGet(*val),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.as_instruction().to_bytes()?)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEntry {
    pub name: String,
    pub ty: DataType,
    pub mutable: bool,
    pub intializer: Intializer,
}

impl GlobalEntry {
    pub fn new_i32(name: impl Into<String>, mutable: bool, value: i32) -> Self {
        Self {
            name: name.into(),
            ty: DataType::I32,
            mutable,
            intializer: Intializer::I32Const(value),
        }
    }

    pub fn new_i64(name: impl Into<String>, mutable: bool, value: i64) -> Self {
        Self {
            name: name.into(),
            ty: DataType::I64,
            mutable,
            intializer: Intializer::I64Const(value),
        }
    }

    pub fn new_f32(name: impl Into<String>, mutable: bool, value: f32) -> Self {
        Self {
            name: name.into(),
            ty: DataType::F32,
            mutable,
            intializer: Intializer::F32Const(value),
        }
    }

    pub fn new_f64(name: impl Into<String>, mutable: bool, value: f64) -> Self {
        Self {
            name: name.into(),
            ty: DataType::F64,
            mutable,
            intializer: Intializer::F64Const(value),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.push(self.ty as u8);
        bytes.push(self.mutable as u8);
        bytes.extend(self.intializer.to_bytes()?);
        bytes.push(0x0B);
        Ok(bytes)
    }
}

#[test]
fn test_global() {
    let mut globals = Global::default();
    let entry = GlobalEntry {
        name: String::from("test"),
        ty: DataType::I32,
        mutable: false,
        intializer: Intializer::I32Const(100),
    };
    globals.push(entry);
    let bytes = globals.to_bytes();
    assert!(bytes.is_ok());
    assert_eq!(bytes.unwrap(), vec![0x06, 0x04, 0x7F, 0x00, 0x41, 0x64]);
}
