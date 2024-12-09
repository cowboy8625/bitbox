use super::DataType;
use super::Instruction;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Code {
    length: usize,
    blocks: Vec<Block>,
}

impl Code {
    const ID: u8 = 0x0A;

    pub fn push(&mut self, block: Block) {
        self.length += block.len();
        self.blocks.push(block);
    }

    pub fn with(mut self, block: Block) -> Self {
        self.length += block.len();
        self.blocks.push(block);
        self
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.push(Code::ID);
        // Add 1 for the count;
        let length = self.length + 1 + self.blocks.len();
        leb128::write::unsigned(&mut bytes, length as u64)?;
        let count = self.blocks.len();
        leb128::write::unsigned(&mut bytes, count as u64)?;
        for block in &self.blocks {
            bytes.extend(block.to_bytes()?);
        }
        Ok(bytes)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LocalVariable {
    pub name: String,
    pub ty: DataType,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Block {
    instructions: Vec<Instruction>,
    locals: Vec<LocalVariable>,
    local_type_info: HashMap<DataType, usize>,
}

impl Block {
    pub fn new(instructions: Vec<Instruction>, locals: Vec<LocalVariable>) -> Self {
        let mut local_type_info = HashMap::new();

        for var in locals.iter() {
            if let Some(count) = local_type_info.get_mut(&var.ty) {
                *count += 1;
                continue;
            }
            local_type_info.insert(var.ty, 1);
        }

        Self {
            instructions,
            locals,
            local_type_info,
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn push_local(&mut self, name: impl Into<String>, ty: DataType) {
        let name = name.into();
        if let Some(count) = self.local_type_info.get_mut(&ty) {
            *count += 1;
        } else {
            self.local_type_info.insert(ty, 1);
        }
        self.locals.push(LocalVariable { name, ty });
    }

    pub fn get_local(&self, name: impl Into<String>) -> Option<&LocalVariable> {
        let name = name.into();
        self.locals
            .iter()
            .find(|var| var.name.as_str() == name.as_str())
    }

    pub fn get_local_index(&self, name: &str, offset: usize) -> Option<usize> {
        self.locals
            .iter()
            .position(|var| var.name.as_str() == name)
            .map(|i| i + offset)
    }

    fn len(&self) -> usize {
        let mut length = 0;

        for instruction in &self.instructions {
            length += instruction.len();
        }

        // Byte for local param count
        length += 1;
        length += self.local_type_info.len() * 2;

        // Byte for 0x0B end of block
        length += 1;
        length
    }

    pub fn with(mut self, instruction: Instruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let length = self.len();
        leb128::write::unsigned(&mut bytes, length as u64)?;
        let local_variable_count = self.local_type_info.len();
        leb128::write::unsigned(&mut bytes, local_variable_count as u64)?;
        for (ty, count) in self.local_type_info.iter() {
            // NOTE: I think maybe this should be leb128 encode but Im not sure yet.
            bytes.push(*count as u8);
            bytes.push(*ty as u8);
        }
        for instruction in &self.instructions {
            bytes.extend(instruction.to_bytes()?);
        }

        bytes.push(0x0B);
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_code_block() {
        let block = Block::default()
            .with(Instruction::I32Const(1))
            .with(Instruction::I32Const(2))
            .with(Instruction::I32Add);
        eprintln!("{:?}", block);
        let bytes = match block.to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => panic!("ERROR: {}", err),
        };
        assert_eq!(bytes, vec![0x07, 0x00, 0x41, 0x01, 0x41, 0x02, 0x6A, 0x0B]);
    }

    #[test]
    fn test_code_section() {
        let code_section = Code::default().with(
            Block::default()
                .with(Instruction::I32Const(1))
                .with(Instruction::I32Const(2))
                .with(Instruction::I32Add),
        );

        let bytes = code_section.to_bytes().unwrap();
        assert_eq!(
            bytes,
            vec![0x0A, 0x09, 0x01, 0x07, 0x00, 0x41, 0x01, 0x41, 0x02, 0x6A, 0x0B]
        );
    }
}
