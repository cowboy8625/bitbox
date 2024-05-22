use crate::asm::{Header, Span};
use crate::instructions::{Data, Either, Execute, Imm, Instruction, Opcode, Register, Type};
use anyhow::Result;

pub struct Mv {
    program: Vec<u8>,
    regesters: Vec<u32>,
    stack: Vec<u32>,
    pub pc: usize,
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
            stack: Vec::new(),
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

    pub fn push_to_stack(&mut self, value: u32) {
        self.stack.push(value);
    }

    pub fn pop_from_stack(&mut self) -> u32 {
        self.stack.pop().unwrap()
    }

    pub fn execute(&mut self) -> Result<()> {
        let opcode: Opcode = self.get_next_byte().try_into()?;
        match opcode {
            Opcode::Load => self.opcode_1reg_imm(Opcode::Load)?,
            Opcode::Push => self.opcode_1reg(Opcode::Push)?,
            Opcode::Pop => self.opcode_1reg(Opcode::Pop)?,
            Opcode::Add => self.opcode_3reg(Opcode::Add)?,
            Opcode::Inc => self.opcode_1reg(Opcode::Inc)?,
            Opcode::Eq => self.opcode_3reg(Opcode::Eq)?,
            Opcode::Jne => self.opcode_1reg_label(Opcode::Jne)?,
            Opcode::Hult => self.opcode_noargs(Opcode::Hult)?,
            Opcode::PrintReg => self.opcode_1reg(Opcode::PrintReg)?,
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while self.running {
            self.execute()?;
        }
        println!("stack: {:?}", self.stack);
        println!("regesters: {:?}", self.regesters);
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

    fn opcode_noargs(&mut self, opcode: Opcode) -> Result<()> {
        Instruction {
            opcode,
            r#type: Type::Void,
            data: Data::NoArgs,
        }
        .execute(self);
        Ok(())
    }

    fn opcode_1reg(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let reg: Register = (self.get_next_byte(), Span::default()).try_into()?;
        Instruction {
            opcode,
            r#type,
            data: Data::Reg1(reg),
        }
        .execute(self);
        Ok(())
    }

    fn opcode_3reg(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let reg1: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let reg2: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let reg3: Register = (self.get_next_byte(), Span::default()).try_into()?;
        Instruction {
            opcode,
            r#type,
            data: Data::Reg3(reg1, reg2, reg3),
        }
        .execute(self);
        Ok(())
    }

    fn opcode_1reg_imm(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let reg: Register = (self.get_next_byte(), Span::default()).try_into()?;

        let mut data = Vec::new();
        // TODO: CLEAN THIS UP
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
            opcode,
            r#type,
            data: Data::Imm(reg, Imm(data)),
        }
        .execute(self);
        Ok(())
    }
    fn opcode_1reg_label(&mut self, opcode: Opcode) -> Result<()> {
        let r#type: Type = self.get_next_byte().try_into()?;
        let lhs: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let rhs: Register = (self.get_next_byte(), Span::default()).try_into()?;
        let label = u32::from_le_bytes([
            self.get_next_byte(),
            self.get_next_byte(),
            self.get_next_byte(),
            self.get_next_byte(),
        ]);
        Instruction {
            opcode,
            r#type,
            data: Data::RegLabel(lhs, rhs, Either::Right(label)),
        }
        .execute(self);
        Ok(())
    }
}
