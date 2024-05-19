mod lexer;
mod parser;
use crate::asm::{
    lexer::{Token, TokenKind},
    parser::Item,
};
use std::collections::HashMap;

use anyhow::Result;

use self::parser::Label;

type SymbolTable = HashMap<String, u32>;

pub use lexer::Span;

pub fn assemble(src: &str) -> Result<Vec<u8>> {
    let program_instructions = parser::parse(src)?;
    let data_offset = 0;
    let symbol_table = symbol_table(&program_instructions, data_offset);
    let mut program = Vec::new();
    let mut entry_point: Option<u32> = None;
    for item in program_instructions.iter() {
        match item {
            Item::EntryPoint(token) => {
                let Token {
                    kind: TokenKind::Identifier(name),
                    ..
                } = token
                else {
                    unreachable!("expected identifier")
                };
                let index = &0; // symbol_table.get(name).unwrap();
                entry_point = Some(*index + Header::SIZE as u32);
            }
            Item::Text(text) => {
                for instruction in text.iter() {
                    let opcode = instruction.opcode.to_bytes();
                    program.extend_from_slice(&opcode);
                }
            }
        }
    }

    let mut header = Header::default();
    // TODO: set the length of the data section here
    header.set_header_text_section(0);
    let Some(entry_point) = entry_point else {
        return Err(anyhow::anyhow!("no entry point found"));
    };
    header.set_header_entry_point(entry_point);
    let mut bin = header.build().to_vec();
    bin.extend_from_slice(&program);
    Ok(bin)
}

#[derive(Debug)]
pub struct Header([u8; Header::SIZE]);
impl Header {
    pub const MAGIC_NUMBER: [u8; 4] = [0x42, 0x42, 0x56, 0x4d]; // BBVM
    pub const TEXT_OFFSET: usize = 4;
    pub const ENTRY_OFFSET: usize = 8;
    pub const SIZE: usize = 64;

    /// Takes the length of bytes of the data section
    fn set_header_text_section(&mut self, offset: u32) {
        let [a, b, c, d] = offset.to_le_bytes();
        self.0[Self::TEXT_OFFSET + 0] = a;
        self.0[Self::TEXT_OFFSET + 1] = b;
        self.0[Self::TEXT_OFFSET + 2] = c;
        self.0[Self::TEXT_OFFSET + 3] = d;
    }

    fn set_header_entry_point(&mut self, offset: u32) {
        let [a, b, c, d] = offset.to_le_bytes();
        self.0[Self::ENTRY_OFFSET + 0] = a;
        self.0[Self::ENTRY_OFFSET + 1] = b;
        self.0[Self::ENTRY_OFFSET + 2] = c;
        self.0[Self::ENTRY_OFFSET + 3] = d;
    }

    fn build(self) -> [u8; Header::SIZE] {
        self.0
    }
}

impl Default for Header {
    fn default() -> Self {
        let mut bytes = [0; Header::SIZE];
        bytes[0] = Header::MAGIC_NUMBER[0];
        bytes[1] = Header::MAGIC_NUMBER[1];
        bytes[2] = Header::MAGIC_NUMBER[2];
        bytes[3] = Header::MAGIC_NUMBER[3];
        Self(bytes)
    }
}

fn symbol_table(instructions: &[Item], data_offset: u32) -> SymbolTable {
    let mut symbol_table = SymbolTable::new();
    let mut ip = data_offset + Header::SIZE as u32;
    for item in instructions.iter() {
        match item {
            Item::EntryPoint(_) => {}
            Item::Text(text) => {
                for instruction in text.iter() {
                    if let Some(Label {
                        name, def: true, ..
                    }) = &instruction.label
                    {
                        symbol_table.insert(name.clone(), ip);
                        ip += 4;
                    } else {
                        ip += 4;
                    }
                }
            }
        }
    }
    symbol_table
}
