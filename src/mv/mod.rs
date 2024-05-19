use crate::asm::{Header, Span};
use crate::instructions::{Data, Execute, Imm, Instruction, Opcode, Register, Type};
use anyhow::Result;

pub struct Mv {
    program: Vec<u8>,
    regesters: Vec<u32>,
    pc: usize,
    pub running: bool,
}

// Public implementation
impl Mv {
    pub fn new(program: Vec<u8>) -> Result<Self> {
        let entry_point = &program[Header::ENTRY_OFFSET..Header::ENTRY_OFFSET + 4];
        let pc = usize::from_le_bytes([
            entry_point[0],
            entry_point[1],
            entry_point[2],
            entry_point[3],
            0,
            0,
            0,
            0,
        ]);

        Ok(Self {
            program,
            regesters: vec![0; 32],
            pc,
            running: true,
        })
    }

    pub fn get_regester(&self, reg: u8) -> &u32 {
        &self.regesters[reg as usize]
    }

    pub fn set_regester(&mut self, reg: u8, value: u32) {
        self.regesters[reg as usize] = value;
    }

    pub fn execute(&mut self) -> Result<()> {
        let opcode: Opcode = self.get_next_byte().try_into()?;
        match opcode {
            Opcode::Load => self.load()?,
            Opcode::Add => self.add()?,
            Opcode::Hult => self.hult()?,
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while self.running {
            self.execute()?;
        }
        println!("{:?}", self.regesters);
        Ok(())
    }
}
// Private implementation
impl Mv {
    fn get_next_byte(&mut self) -> u8 {
        let pc = self.pc;
        self.pc += 1;
        self.program[pc]
    }

    fn load(&mut self) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let reg: Register = (self.get_next_byte(), Span::default()).try_into()?;

        let mut data = Vec::new();
        match r#type {
            Type::U(8) => data.push(self.get_next_byte()),
            Type::U(16) => data.extend_from_slice(&[self.get_next_byte(), self.get_next_byte()]),
            Type::U(32) => data.extend_from_slice(&[
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
            ]),
            Type::U(64) => data.extend_from_slice(&[
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
            ]),
            Type::U(128) => data.extend_from_slice(&[
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
                self.get_next_byte(),
            ]),
            Type::I(8) => todo!(),
            Type::I(16) => todo!(),
            Type::I(32) => todo!(),
            Type::I(64) => todo!(),
            Type::I(128) => todo!(),
            Type::Void => todo!(),
            _ => unreachable!("Unimplemented type: {:?}", r#type),
        }

        Instruction {
            opcode: Opcode::Load,
            r#type,
            data: Data::Imm(reg, Imm(data)),
        }
        .execute(self);
        Ok(())
    }

    fn add(&mut self) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let des: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let lhs: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let rhs: Register = (self.get_next_byte(), Span::default()).try_into()?;

        Instruction {
            opcode: Opcode::Add,
            r#type,
            data: Data::Reg3(des, lhs, rhs),
        }
        .execute(self);
        Ok(())
    }

    fn hult(&mut self) -> Result<()> {
        Instruction {
            opcode: Opcode::Hult,
            r#type: Type::Void,
            data: Data::NoArgs,
        }
        .execute(self);
        Ok(())
    }
}
