use super::Instruction;
use anyhow::Result;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Data {
    pub data: Vec<Segment>,
}

impl Data {
    const ID: u8 = 0x0B;

    pub fn push(&mut self, segment: Segment) {
        self.data.push(segment);
    }

    pub fn get_segment_by_name(&self, name: impl Into<String>) -> Option<&Segment> {
        let name = name.into();
        self.data.iter().find(|segment| segment.name == name)
    }

    pub fn with(mut self, segment: Segment) -> Self {
        self.data.push(segment);
        self
    }

    pub fn get_id(&self, name: impl Into<String>) -> Option<u32> {
        let name = name.into();
        self.data
            .iter()
            .position(|segment| segment.name == name)
            .map(|id| id as u32)
    }

    pub fn len(&self) -> usize {
        let mut length = 0;
        for segment in &self.data {
            length += segment.data.len();
        }
        length
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.push(Data::ID);
        // Add 1 for the count;
        let mut length = 1;

        let mut segment_bytes = Vec::new();
        for segment in &self.data {
            segment_bytes.extend(segment.to_bytes()?);
        }
        length += segment_bytes.len();

        leb128::write::unsigned(&mut bytes, length as u64)?;
        leb128::write::unsigned(&mut bytes, self.data.len() as u64)?;
        bytes.extend(segment_bytes);
        Ok(bytes)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Segment {
    pub name: String,
    pub offset: u32,
    pub instructions: Vec<Instruction>,
    pub data: Vec<u8>,
}

impl Segment {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_instruction(mut self, instruction: Instruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data.extend(data);
        self
    }

    pub fn push_data(&mut self, data: Vec<u8>) -> u32 {
        let offset = self.data.len() as u32;
        self.data.extend(data);
        offset
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        leb128::write::unsigned(&mut bytes, self.offset as u64)?;

        for instruction in &self.instructions {
            bytes.extend(instruction.to_bytes()?);
        }

        if !self.instructions.is_empty() {
            bytes.push(0x0B);
        }

        leb128::write::unsigned(&mut bytes, self.data.len() as u64)?;
        bytes.extend(self.data.clone());
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_segment() {
        let segment = Segment::default()
            .with_instruction(Instruction::I32Const(1))
            .with_data("abc".as_bytes().to_vec());
        let bytes = segment.to_bytes().unwrap();
        assert_eq!(bytes, vec![0x00, 0x41, 0x01, 0x0b, 0x03, 0x61, 0x62, 0x63]);
    }

    #[test]
    fn test_data() {
        let segment = Segment::default()
            .with_name("string")
            .with_instruction(Instruction::I32Const(1))
            .with_data("abc".as_bytes().to_vec());
        let data = Data::default().with(segment);
        let bytes = data.to_bytes().unwrap();
        assert_eq!(
            bytes,
            vec![0x0B, 0x09, 0x01, 0x00, 0x41, 0x01, 0x0b, 0x03, 0x61, 0x62, 0x63]
        );
    }
}
